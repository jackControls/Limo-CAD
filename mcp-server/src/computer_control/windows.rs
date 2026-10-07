use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Deserialize;
use serde_json::{json, Value};
use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, LPARAM, POINT, RECT, WAIT_TIMEOUT,
};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, WaitForSingleObject,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
};
use windows_sys::Win32::UI::HiDpi::{
    GetDpiForWindow, SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::{build_pair, session};

const OBSERVATION_MS: u64 = 60_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    action: String,
    session_id: Option<String>,
    observation: Option<String>,
    point: Option<[i32; 2]>,
    to: Option<[i32; 2]>,
    button: Option<String>,
    delta: Option<i32>,
    key: Option<String>,
    text: Option<String>,
}

struct Observation {
    token: String,
    expires_ms: u64,
    owner: Value,
    hwnd: usize,
    bounds: Option<[i32; 4]>,
    layout: Option<Value>,
    editable_focus: bool,
    process: DesktopProcess,
}

/// Holding the process object keeps a recycled PID from inheriting a token.
struct DesktopProcess(usize);
impl DesktopProcess {
    fn open(pid: u32) -> Result<Self, String> {
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                pid,
            )
        };
        if handle.is_null() {
            return Err(format!(
                "Cannot retain CAD process ownership: {}",
                std::io::Error::last_os_error()
            ));
        }
        let process = Self(handle as usize);
        process.verify()?;
        Ok(process)
    }

    fn verify(&self) -> Result<(), String> {
        if unsafe { WaitForSingleObject(self.0 as HANDLE, 0) } == WAIT_TIMEOUT {
            Ok(())
        } else {
            Err("Observed CAD process has exited or is unavailable; no input was sent".into())
        }
    }
}
impl Drop for DesktopProcess {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0 as HANDLE);
        }
    }
}

#[derive(Default)]
pub(super) struct ComputerControl {
    observation: Option<Observation>,
}

impl ComputerControl {
    pub(super) fn call(
        &mut self,
        arguments: &Value,
        attached: Option<&str>,
    ) -> Result<Value, String> {
        let request: Request = serde_json::from_value(arguments.clone())
            .map_err(|error| format!("Invalid computer control request: {error}"))?;
        if request.action == "observe" {
            self.observation = None;
            let session_id =
                request.session_id.as_deref().or(attached).ok_or(
                    "Computer control needs an explicit or attached active desktop session",
                )?;
            let owner = session::computer_control_owner(session_id)?;
            let process = DesktopProcess::open(
                owner["pid"]
                    .as_u64()
                    .and_then(|pid| u32::try_from(pid).ok())
                    .ok_or("Owner has no PID")?,
            )?;
            let hwnd = native_window(&owner)?;
            let inspect = inspect(session_id, false)?;
            process.verify()?;
            if session::computer_control_owner(session_id)? != owner {
                return Err("Desktop owner changed during observation; observe again".into());
            }
            let presented = inspect["presented"] == true && unsafe { IsIconic(hwnd) } == 0;
            let bounds = if presented {
                Some(client_bounds(hwnd)?)
            } else {
                None
            };
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let token = format!(
                "computer-{}-{}-{}",
                std::process::id(),
                session::now_ms(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let expires_ms = session::now_ms().saturating_add(OBSERVATION_MS);
            let focused = &inspect["ui"]["focused_control"];
            let editable_focus = presented
                && inspect["ui"]["surfaces"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .flat_map(|surface| surface["controls"].as_array().into_iter().flatten())
                    .any(|control| {
                        control["id"] == *focused
                            && matches!(
                                control["role"].as_str(),
                                Some("textbox" | "multiline_textbox")
                            )
                            && control["read_only"] != true
                            && control["disabled"] != true
                    });
            self.observation = Some(Observation {
                token: token.clone(),
                expires_ms,
                owner: owner.clone(),
                hwnd: hwnd as usize,
                bounds,
                layout: presented.then(|| layout(&inspect)),
                editable_focus,
                process,
            });
            let screen_bounds = bounds.map(
                |bounds| json!({"x":bounds[0],"y":bounds[1],"width":bounds[2],"height":bounds[3]}),
            );
            let scale = bounds.map(|bounds| {
                [
                    bounds[2] as f64
                        / inspect["ui"]["client"]["width"]
                            .as_f64()
                            .unwrap_or(bounds[2] as f64),
                    bounds[3] as f64
                        / inspect["ui"]["client"]["height"]
                            .as_f64()
                            .unwrap_or(bounds[3] as f64),
                ]
            });
            let inspection = if presented {
                inspect
            } else {
                json!({"status":inspect["status"],"presented":false,"build_pair":inspect["build_pair"],
                    "render_status":inspect["render_status"],"hint":"Owner-only focus observation; no rendered controls or pointer coordinates are qualified."})
            };
            return Ok(
                json!({"status":"observed","observation":token,"expires_ms":expires_ms,
                "owner":owner,"window_handle":hwnd as usize,"executable":std::env::current_exe().map_err(|e|e.to_string())?,
                "presented":presented,"focus_only":!presented,"minimized":unsafe { IsIconic(hwnd) } != 0,
                "client_screen_bounds":screen_bounds,
                "coordinate_space":"physical_client_pixels","dpi":unsafe { GetDpiForWindow(hwnd) },
                "interface_to_physical_scale":scale,
                "foreground":unsafe { GetForegroundWindow() == hwnd },"editable_focus":editable_focus,"inspection":inspection,
                "hint":"Use cad_interface capture for the actual rendered image. Focus if needed, observe again, then send one input and observe its visible result."}),
            );
        }
        let observed = self
            .observation
            .take()
            .ok_or("Observe the CAD window before sending input")?;
        if request.observation.as_deref() != Some(&observed.token)
            || session::now_ms() > observed.expires_ms
        {
            return Err(
                "Computer observation is missing, expired or already consumed; observe again"
                    .into(),
            );
        }
        let session_id = observed.owner["session_id"]
            .as_str()
            .ok_or("Observation has no session")?;
        observed.process.verify()?;
        if request
            .session_id
            .as_deref()
            .is_some_and(|id| id != session_id)
            || session::computer_control_owner(session_id)? != observed.owner
        {
            return Err("Active desktop document changed; observe again before input".into());
        }
        let hwnd = native_window(&observed.owner)?;
        if hwnd as usize != observed.hwnd {
            return Err("CAD window was replaced, moved or resized; observe again".into());
        }
        let focus = request.action == "focus";
        if !focus && observed.bounds.is_none() {
            return Err("This observation qualifies focus only; restore/focus CAD and observe a presented frame before input".into());
        }
        if let Some(bounds) = observed.bounds {
            if client_bounds(hwnd)? != bounds {
                return Err("CAD window moved or resized; observe again".into());
            }
        }
        let current = inspect(session_id, !focus)?;
        if let Some(expected) = &observed.layout {
            if layout(&current) != *expected {
                return Err(
                    "Rendered controls or camera changed; observe again before input".into(),
                );
            }
        }
        if session::computer_control_owner(session_id)? != observed.owner {
            return Err("Desktop changed while checking input guards; observe again".into());
        }
        if focus {
            unsafe {
                if IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, SW_RESTORE);
                }
                SetForegroundWindow(hwnd);
            }
            if unsafe { GetForegroundWindow() } != hwnd || unsafe { IsIconic(hwnd) } != 0 {
                return Err("Windows denied CAD foreground activation; no input was sent".into());
            }
            return Ok(
                json!({"status":"focused","owner":observed.owner,"observation_consumed":true,
                "hint":"Observe again before sending mouse or keyboard input."}),
            );
        }
        guard_foreground(hwnd)?;
        guard_held_input()?;
        let inputs = inputs(&request, &observed, hwnd)?;
        guard_foreground(hwnd)?;
        if Some(client_bounds(hwnd)?) != observed.bounds
            || session::computer_control_owner(session_id)? != observed.owner
            || session::now_ms() > observed.expires_ms
        {
            return Err("Desktop moved or changed immediately before input; observe again".into());
        }
        observed.process.verify()?;
        let sent = unsafe {
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        };
        if sent as usize != inputs.len() {
            let error = std::io::Error::last_os_error();
            let released = release_inserted(&inputs[..sent as usize]);
            return Err(format!("Windows inserted {sent}/{} input events ({error}); {released} release events inserted to avoid held input. Do not repeat the action without observing the result", inputs.len()));
        }
        Ok(
            json!({"status":"input_sent","action":request.action,"event_count":sent,"owner":observed.owner,
            "observation_consumed":true,"hint":"OS insertion does not confirm product behavior. Observe and capture before the next action; do not blindly retry."}),
        )
    }
}

fn inspect(session_id: &str, require_presented: bool) -> Result<Value, String> {
    let mut result =
        session::request_ui(&json!({"action":"inspect","session_id":session_id}), None)?;
    build_pair::decorate(&mut result);
    if result["status"] != "applied"
        || (require_presented && result["presented"] != true)
        || result["build_pair"]["status"] != "matched"
    {
        return Err(json!({"code":"computer_control_not_ready","inspection":result,
            "hint":"The current CAD window must have the same clean build as this MCP process. Pointer and keyboard input additionally require a presented frame; focus can restore a retained window."}).to_string());
    }
    Ok(result)
}

fn layout(inspect: &Value) -> Value {
    let mut ui = inspect["ui"].clone();
    let focused = ui["focused_control"].clone();
    if let Some(surfaces) = ui["surfaces"].as_array_mut() {
        for surface in surfaces {
            if let Some(controls) = surface["controls"].as_array_mut() {
                for control in controls {
                    let is_focused = control["id"] == focused;
                    if let Some(map) = control.as_object_mut() {
                        map.remove("id");
                        map.insert("focused".into(), json!(is_focused));
                    }
                }
            }
        }
    }
    if let Some(map) = ui.as_object_mut() {
        map.remove("focused_control");
        map.remove("unlabeled_controls");
        map.remove("ime_diagnostics");
    }
    json!({"ui":ui,"view_state":inspect["view_state"]})
}

fn native_window(owner: &Value) -> Result<HWND, String> {
    struct Search {
        pid: u32,
        windows: Vec<usize>,
    }
    unsafe extern "system" fn visit(hwnd: HWND, pointer: LPARAM) -> i32 {
        let search = &mut *(pointer as *mut Search);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == search.pid && IsWindowVisible(hwnd) != 0 && GetWindow(hwnd, GW_OWNER).is_null() {
            search.windows.push(hwnd as usize);
        }
        1
    }
    let pid = owner["pid"]
        .as_u64()
        .and_then(|pid| u32::try_from(pid).ok())
        .ok_or("Owner has no PID")?;
    let mut search = Search {
        pid,
        windows: Vec::new(),
    };
    if unsafe { EnumWindows(Some(visit), &mut search as *mut Search as LPARAM) } == 0 {
        return Err(format!(
            "Could not enumerate CAD windows: {}",
            std::io::Error::last_os_error()
        ));
    }
    let [window] = search.windows.as_slice() else {
        return Err(
            "CAD process must own exactly one visible main window; no input was sent".into(),
        );
    };
    let hwnd = *window as HWND;
    if unsafe { IsWindowEnabled(hwnd) } == 0 {
        return Err("CAD window is blocked by a native modal dialog; no input was sent".into());
    }
    let current = std::env::current_exe().map_err(|e| e.to_string())?;
    if !same_path(&process_image(pid)?, &current)? {
        return Err(
            "GUI and MCP executable paths differ; restart both from the canonical installed binary"
                .into(),
        );
    }
    Ok(hwnd)
}

fn process_image(pid: u32) -> Result<PathBuf, String> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Err(format!(
            "Cannot verify CAD process {pid}: {}",
            std::io::Error::last_os_error()
        ));
    }
    let mut buffer = vec![0u16; 32768];
    let mut length = buffer.len() as u32;
    let success =
        unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) };
    let error = std::io::Error::last_os_error();
    unsafe {
        CloseHandle(process);
    }
    if success == 0 {
        return Err(format!("Cannot read CAD executable: {error}"));
    }
    Ok(PathBuf::from(
        String::from_utf16(&buffer[..length as usize]).map_err(|e| e.to_string())?,
    ))
}

fn same_path(left: &Path, right: &Path) -> Result<bool, String> {
    let canonical = |path: &Path| {
        std::fs::canonicalize(path)
            .map(|path| path.to_string_lossy().to_lowercase())
            .map_err(|e| e.to_string())
    };
    Ok(canonical(left)? == canonical(right)?)
}

struct DpiGuard(DPI_AWARENESS_CONTEXT);
impl DpiGuard {
    fn enter() -> Result<Self, String> {
        let previous =
            unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        if previous.is_null() {
            return Err("Cannot establish physical screen coordinates".into());
        }
        Ok(Self(previous))
    }
}
impl Drop for DpiGuard {
    fn drop(&mut self) {
        unsafe {
            SetThreadDpiAwarenessContext(self.0);
        }
    }
}

fn client_bounds(hwnd: HWND) -> Result<[i32; 4], String> {
    let _dpi = DpiGuard::enter()?;
    let mut rect = RECT::default();
    let mut origin = POINT::default();
    if unsafe { GetClientRect(hwnd, &mut rect) } == 0
        || unsafe { ClientToScreen(hwnd, &mut origin) } == 0
    {
        return Err("Cannot read the CAD client rectangle".into());
    }
    let bounds = [
        origin.x,
        origin.y,
        rect.right - rect.left,
        rect.bottom - rect.top,
    ];
    if bounds[2] <= 0 || bounds[3] <= 0 {
        return Err("CAD client has no drawable area".into());
    }
    Ok(bounds)
}

fn guard_foreground(hwnd: HWND) -> Result<(), String> {
    if unsafe { GetForegroundWindow() } != hwnd {
        return Err("CAD is not the foreground window; focus it and observe again".into());
    }
    let mut pid = 0;
    let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    let mut info = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetGUIThreadInfo(thread, &mut info) } == 0
        || !info.hwndCapture.is_null()
        || !info.hwndMenuOwner.is_null()
    {
        return Err(
            "CAD input is captured by an existing gesture or native menu; no input was sent".into(),
        );
    }
    Ok(())
}

fn guard_held_input() -> Result<(), String> {
    for key in 1..=254 {
        if unsafe { GetAsyncKeyState(key) } < 0 {
            return Err(
                "A physical key or mouse button is held; release it before computer control".into(),
            );
        }
    }
    Ok(())
}

fn mouse(flags: u32, data: u32, dx: i32, dy: i32) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

fn move_to(point: [i32; 2], observed: &Observation, hwnd: HWND) -> Result<INPUT, String> {
    let _dpi = DpiGuard::enter()?;
    let bounds = observed
        .bounds
        .ok_or("This observation has no qualified pointer coordinates")?;
    if point[0] < 0 || point[1] < 0 || point[0] >= bounds[2] || point[1] >= bounds[3] {
        return Err("Pointer point is outside the observed CAD client rectangle".into());
    }
    let screen = POINT {
        x: bounds[0] + point[0],
        y: bounds[1] + point[1],
    };
    let target = unsafe { WindowFromPoint(screen) };
    if target.is_null() || unsafe { GetAncestor(target, GA_ROOT) } != hwnd {
        return Err("Pointer target is occluded by another window; no input was sent".into());
    }
    let origin_x = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let origin_y = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
    if width <= 1 || height <= 1 {
        return Err("Windows virtual screen has no drawable bounds".into());
    }
    let x = ((screen.x - origin_x) as i64 * 65535 / (width - 1) as i64) as i32;
    let y = ((screen.y - origin_y) as i64 * 65535 / (height - 1) as i64) as i32;
    Ok(mouse(
        MOUSEEVENTF_MOVE
            | MOUSEEVENTF_ABSOLUTE
            | MOUSEEVENTF_VIRTUALDESK
            | MOUSEEVENTF_MOVE_NOCOALESCE,
        0,
        x,
        y,
    ))
}

fn keyboard(key: u16, scan: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: scan,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

/// On a partial insertion only release keys/buttons this request pressed.
fn release_inserted(inserted: &[INPUT]) -> u32 {
    let releases: Vec<_> = inserted
        .iter()
        .filter_map(|input| unsafe {
            match input.r#type {
                INPUT_KEYBOARD => {
                    let key = input.Anonymous.ki;
                    (key.dwFlags & KEYEVENTF_KEYUP == 0)
                        .then(|| keyboard(key.wVk, key.wScan, key.dwFlags | KEYEVENTF_KEYUP))
                }
                INPUT_MOUSE => {
                    let flags = input.Anonymous.mi.dwFlags;
                    let up = if flags & MOUSEEVENTF_LEFTDOWN != 0 {
                        MOUSEEVENTF_LEFTUP
                    } else if flags & MOUSEEVENTF_MIDDLEDOWN != 0 {
                        MOUSEEVENTF_MIDDLEUP
                    } else if flags & MOUSEEVENTF_RIGHTDOWN != 0 {
                        MOUSEEVENTF_RIGHTUP
                    } else {
                        return None;
                    };
                    Some(mouse(up, 0, 0, 0))
                }
                _ => None,
            }
        })
        .collect();
    if releases.is_empty() {
        0
    } else {
        unsafe {
            SendInput(
                releases.len() as u32,
                releases.as_ptr(),
                size_of::<INPUT>() as i32,
            )
        }
    }
}

fn inputs(request: &Request, observed: &Observation, hwnd: HWND) -> Result<Vec<INPUT>, String> {
    let mut inputs = Vec::new();
    match request.action.as_str() {
        "click" | "double_click" | "drag" | "wheel" => {
            let point = request
                .point
                .ok_or("Pointer input needs point in physical client pixels")?;
            inputs.push(move_to(point, observed, hwnd)?);
            if request.action == "wheel" {
                let delta = request
                    .delta
                    .filter(|delta| *delta != 0 && (-1200..=1200).contains(delta))
                    .ok_or("Wheel delta must be nonzero and from -1200 to 1200")?;
                inputs.push(mouse(MOUSEEVENTF_WHEEL, delta as u32, 0, 0));
            } else {
                let (down, up) = match request.button.as_deref().unwrap_or("left") {
                    "left" => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
                    "middle" => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
                    "right" => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
                    _ => return Err("Mouse button must be left, middle or right".into()),
                };
                inputs.push(mouse(down, 0, 0, 0));
                if request.action == "drag" {
                    let to = request.to.ok_or("Drag requires an endpoint to")?;
                    move_to(to, observed, hwnd)?;
                    for step in 1..=16 {
                        let at = [
                            point[0] + ((to[0] as i64 - point[0] as i64) * step / 16) as i32,
                            point[1] + ((to[1] as i64 - point[1] as i64) * step / 16) as i32,
                        ];
                        inputs.push(move_to(at, observed, hwnd)?);
                    }
                }
                inputs.push(mouse(up, 0, 0, 0));
                if request.action == "double_click" {
                    inputs.push(mouse(down, 0, 0, 0));
                    inputs.push(mouse(up, 0, 0, 0));
                }
            }
        }
        "key" => {
            let chord = request.key.as_deref().ok_or("Key input requires key")?;
            let mut parts: Vec<_> = chord.split('+').collect();
            let key = key_code(parts.pop().unwrap_or_default())?;
            let mut modifiers = Vec::new();
            for part in parts {
                let modifier = match part {
                    "Ctrl" => VK_CONTROL,
                    "Shift" => VK_SHIFT,
                    _ => return Err("Only Ctrl and Shift key modifiers are supported".into()),
                };
                if modifiers.contains(&modifier) {
                    return Err("Duplicate key modifier".into());
                }
                modifiers.push(modifier);
                inputs.push(keyboard(modifier, 0, 0));
            }
            let extended = if matches!(
                key,
                VK_LEFT
                    | VK_RIGHT
                    | VK_UP
                    | VK_DOWN
                    | VK_HOME
                    | VK_END
                    | VK_PRIOR
                    | VK_NEXT
                    | VK_INSERT
                    | VK_DELETE
            ) {
                KEYEVENTF_EXTENDEDKEY
            } else {
                0
            };
            inputs.push(keyboard(key, 0, extended));
            inputs.push(keyboard(key, 0, extended | KEYEVENTF_KEYUP));
            for modifier in modifiers.into_iter().rev() {
                inputs.push(keyboard(modifier, 0, KEYEVENTF_KEYUP));
            }
        }
        "text" => {
            if !observed.editable_focus {
                return Err("Observe and focus an editable CAD text control before typing".into());
            }
            let text = request.text.as_deref().ok_or("Text input requires text")?;
            if text.is_empty() || text.chars().count() > 512 || text.chars().any(char::is_control) {
                return Err("Text must contain 1–512 printable Unicode characters".into());
            }
            for unit in text.encode_utf16() {
                inputs.push(keyboard(0, unit, KEYEVENTF_UNICODE));
                inputs.push(keyboard(0, unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP));
            }
        }
        _ => return Err("Unknown computer control action".into()),
    }
    Ok(inputs)
}

fn key_code(key: &str) -> Result<u16, String> {
    let code = match key {
        "Enter" => VK_RETURN,
        "Escape" => VK_ESCAPE,
        "Tab" => VK_TAB,
        "Backspace" => VK_BACK,
        "Delete" => VK_DELETE,
        "Insert" => VK_INSERT,
        "ArrowLeft" => VK_LEFT,
        "ArrowRight" => VK_RIGHT,
        "ArrowUp" => VK_UP,
        "ArrowDown" => VK_DOWN,
        "Home" => VK_HOME,
        "End" => VK_END,
        "PageUp" => VK_PRIOR,
        "PageDown" => VK_NEXT,
        "Space" => VK_SPACE,
        "F1" => VK_F1,
        "F2" => VK_F2,
        "F3" => VK_F3,
        "F4" => VK_F4,
        "F5" => VK_F5,
        "F6" => VK_F6,
        "F7" => VK_F7,
        "F8" => VK_F8,
        "F9" => VK_F9,
        "F10" => VK_F10,
        "F11" => VK_F11,
        "F12" => VK_F12,
        _ if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
            key.as_bytes()[0].to_ascii_uppercase() as u16
        }
        _ => {
            return Err(
                "Unsupported key; use a named navigation key, F1–F12, letter or digit".into(),
            )
        }
    };
    Ok(code)
}
