use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use enigo::{Axis, Button, Direction, Enigo, Key, Keyboard, Mouse, Settings};
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
use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
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
            Err("Observed CAD process has exited or is unavailable".into())
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
pub(crate) struct ComputerControl {
    observation: Option<Observation>,
}

impl ComputerControl {
    pub(crate) fn call(
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
        guard_foreground(hwnd, false)?;
        guard_held_input()?;
        let plan = plan(&request, &observed, hwnd)?;
        guard_foreground(hwnd, false)?;
        if Some(client_bounds(hwnd)?) != observed.bounds
            || session::computer_control_owner(session_id)? != observed.owner
            || session::now_ms() > observed.expires_ms
        {
            return Err("Desktop moved or changed immediately before input; observe again".into());
        }
        observed.process.verify()?;
        let mut driver = InputDriver::new()?;
        let mut completed = 0;
        let mut pointer = None;
        for step in &plan {
            let guard = guard_action(&observed, hwnd, driver.holds_button(), completed == 0)
                .and_then(|()| match *step {
                    Step::Move(point) => guard_pointer(point, hwnd),
                    Step::Button(_, _) | Step::Scroll(_) => guard_cursor(
                        pointer.ok_or("Pointer input has no planned position")?,
                        hwnd,
                    ),
                    _ => Ok(()),
                });
            if let Err(error) = guard {
                if completed == 0 {
                    return Err(error);
                }
                let cleanup_errors = driver.release_all();
                return Ok(json!({"status":"input_incomplete","action":request.action,
                    "backend":"enigo","completed_primitives":completed,"planned_primitives":plan.len(),
                    "failed_primitive":step.kind(),"error":error,"cleanup_errors":cleanup_errors,
                    "input_may_have_been_inserted":true,"owner":observed.owner,
                    "observation_consumed":true,"hint":"Input stopped when an ownership, focus or visibility guard changed. Observe and capture the result; do not blindly retry."}));
            }
            if let Err(error) = driver.apply(*step) {
                let cleanup_errors = driver.release_all();
                return Ok(json!({"status":"input_incomplete","action":request.action,
                    "backend":"enigo","completed_primitives":completed,"planned_primitives":plan.len(),
                    "failed_primitive":step.kind(),"error":error,"cleanup_errors":cleanup_errors,
                    "input_may_have_been_inserted":true,"owner":observed.owner,
                    "observation_consumed":true,"hint":"The input backend cannot report how many events a failed primitive inserted. Owned held keys/buttons received one release attempt. Observe and capture the result; do not blindly retry."}));
            }
            if let Step::Move(point) = *step {
                pointer = Some(point);
            }
            completed += 1;
        }
        Ok(
            json!({"status":"input_sent","action":request.action,"backend":"enigo",
            "completed_primitives":completed,"owner":observed.owner,
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

fn guard_foreground(hwnd: HWND, owns_capture: bool) -> Result<(), String> {
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
        || (!info.hwndCapture.is_null()
            && (!owns_capture || unsafe { GetAncestor(info.hwndCapture, GA_ROOT) } != hwnd))
        || !info.hwndMenuOwner.is_null()
        || (!info.hwndFocus.is_null() && unsafe { GetAncestor(info.hwndFocus, GA_ROOT) } != hwnd)
    {
        return Err("CAD input is captured by an existing gesture or native menu".into());
    }
    Ok(())
}

fn guard_action(
    observed: &Observation,
    hwnd: HWND,
    owns_capture: bool,
    first: bool,
) -> Result<(), String> {
    observed.process.verify()?;
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid);
    }
    if unsafe { IsWindow(hwnd) } == 0
        || unsafe { IsWindowEnabled(hwnd) } == 0
        || unsafe { IsWindowVisible(hwnd) } == 0
        || unsafe { IsIconic(hwnd) } != 0
        || observed.owner["pid"].as_u64() != Some(pid as u64)
        || session::now_ms() > observed.expires_ms
        || Some(client_bounds(hwnd)?) != observed.bounds
    {
        return Err("Observed CAD window is no longer available at its qualified bounds".into());
    }
    let session_id = observed.owner["session_id"]
        .as_str()
        .ok_or("Observation has no session")?;
    let current = session::computer_control_owner(session_id)?;
    if (first && current != observed.owner)
        || [
            "session_id",
            "window_id",
            "document_id",
            "process_instance_id",
            "pid",
        ]
        .iter()
        .any(|field| current[*field] != observed.owner[*field])
    {
        return Err("Active CAD document or window owner changed during input".into());
    }
    guard_foreground(hwnd, owns_capture)
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

fn screen_point(point: [i32; 2], observed: &Observation, hwnd: HWND) -> Result<[i32; 2], String> {
    let bounds = observed
        .bounds
        .ok_or("This observation has no qualified pointer coordinates")?;
    if point[0] < 0 || point[1] < 0 || point[0] >= bounds[2] || point[1] >= bounds[3] {
        return Err("Pointer point is outside the observed CAD client rectangle".into());
    }
    let screen = [
        bounds[0]
            .checked_add(point[0])
            .ok_or("Pointer X overflow")?,
        bounds[1]
            .checked_add(point[1])
            .ok_or("Pointer Y overflow")?,
    ];
    guard_pointer(screen, hwnd)?;
    Ok(screen)
}

fn guard_pointer(screen: [i32; 2], hwnd: HWND) -> Result<(), String> {
    let _dpi = DpiGuard::enter()?;
    let target = unsafe {
        WindowFromPoint(POINT {
            x: screen[0],
            y: screen[1],
        })
    };
    if target.is_null() || unsafe { GetAncestor(target, GA_ROOT) } != hwnd {
        return Err("Pointer target is occluded by another window".into());
    }
    Ok(())
}

fn guard_cursor(expected: [i32; 2], hwnd: HWND) -> Result<(), String> {
    let _dpi = DpiGuard::enter()?;
    let mut actual = POINT::default();
    if unsafe { GetCursorPos(&mut actual) } == 0 || [actual.x, actual.y] != expected {
        return Err("Cursor moved away from its qualified CAD target".into());
    }
    guard_pointer(expected, hwnd)
}

/// Enigo 0.6.1 absolute movement normalizes against the primary monitor only.
/// Native physical positioning preserves negative and mixed-DPI monitor coordinates.
fn position_cursor(screen: [i32; 2]) -> Result<(), String> {
    let _dpi = DpiGuard::enter()?;
    let mut actual = POINT::default();
    if unsafe { SetCursorPos(screen[0], screen[1]) } == 0
        || unsafe { GetCursorPos(&mut actual) } == 0
        || [actual.x, actual.y] != screen
    {
        return Err("Windows did not position the cursor at the qualified CAD point".into());
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Step<'a> {
    Move([i32; 2]),
    Button(Button, Direction),
    Key(Key, Direction),
    Scroll(i32),
    Text(&'a str),
}

impl Step<'_> {
    fn kind(&self) -> &'static str {
        match self {
            Self::Move(_) => "pointer_move",
            Self::Button(_, Direction::Press) => "button_press",
            Self::Button(_, _) => "button_release",
            Self::Key(_, Direction::Press) => "key_press",
            Self::Key(_, _) => "key_release",
            Self::Scroll(_) => "wheel",
            Self::Text(_) => "text",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Held {
    Button(Button),
    Key(Key),
}

/// Track attempted presses too: a failed Enigo call may already have inserted input.
struct InputDriver {
    enigo: Enigo,
    held: Vec<Held>,
}

impl InputDriver {
    fn new() -> Result<Self, String> {
        let settings = Settings {
            release_keys_when_dropped: false,
            ..Default::default()
        };
        Ok(Self {
            enigo: Enigo::new(&settings).map_err(|error| error.to_string())?,
            held: Vec::new(),
        })
    }

    fn holds_button(&self) -> bool {
        self.held.iter().any(|held| matches!(held, Held::Button(_)))
    }

    fn apply(&mut self, step: Step<'_>) -> Result<(), String> {
        let held = match step {
            Step::Button(button, direction) => Some((Held::Button(button), direction)),
            Step::Key(key, direction) => Some((Held::Key(key), direction)),
            _ => None,
        };
        if let Some((held, Direction::Press)) = held {
            self.held.push(held);
        }
        match step {
            Step::Move(point) => position_cursor(point)?,
            Step::Button(button, direction) => self
                .enigo
                .button(button, direction)
                .map_err(|error| error.to_string())?,
            Step::Key(key, direction) => self
                .enigo
                .key(key, direction)
                .map_err(|error| error.to_string())?,
            Step::Scroll(notches) => self
                .enigo
                .scroll(notches, Axis::Vertical)
                .map_err(|error| error.to_string())?,
            Step::Text(text) => self.enigo.text(text).map_err(|error| error.to_string())?,
        }
        if let Some((held, Direction::Release)) = held {
            self.held.retain(|candidate| *candidate != held);
        }
        Ok(())
    }

    fn release_all(&mut self) -> Vec<String> {
        let mut errors = Vec::new();
        for held in self.held.drain(..).rev() {
            let result = match held {
                Held::Button(button) => self.enigo.button(button, Direction::Release),
                Held::Key(key) => self.enigo.key(key, Direction::Release),
            };
            if let Err(error) = result {
                errors.push(error.to_string());
            }
        }
        errors
    }
}

impl Drop for InputDriver {
    fn drop(&mut self) {
        let _ = self.release_all();
    }
}

fn plan<'a>(
    request: &'a Request,
    observed: &Observation,
    hwnd: HWND,
) -> Result<Vec<Step<'a>>, String> {
    let mut steps = Vec::new();
    match request.action.as_str() {
        "click" | "double_click" | "drag" | "wheel" => {
            let point = request
                .point
                .ok_or("Pointer input needs point in physical client pixels")?;
            steps.push(Step::Move(screen_point(point, observed, hwnd)?));
            if request.action == "wheel" {
                let delta = request
                    .delta
                    .filter(|delta| {
                        *delta != 0 && (-1200..=1200).contains(delta) && *delta % 120 == 0
                    })
                    .ok_or("Wheel delta must be a nonzero multiple of 120 from -1200 to 1200")?;
                steps.push(Step::Scroll(-delta / 120));
            } else {
                let button = match request.button.as_deref().unwrap_or("left") {
                    "left" => Button::Left,
                    "middle" => Button::Middle,
                    "right" => Button::Right,
                    _ => return Err("Mouse button must be left, middle or right".into()),
                };
                steps.push(Step::Button(button, Direction::Press));
                if request.action == "drag" {
                    let to = request.to.ok_or("Drag requires an endpoint to")?;
                    screen_point(to, observed, hwnd)?;
                    for step in 1..=16 {
                        let at = [
                            point[0] + ((to[0] as i64 - point[0] as i64) * step / 16) as i32,
                            point[1] + ((to[1] as i64 - point[1] as i64) * step / 16) as i32,
                        ];
                        steps.push(Step::Move(screen_point(at, observed, hwnd)?));
                    }
                }
                steps.push(Step::Button(button, Direction::Release));
                if request.action == "double_click" {
                    steps.push(Step::Button(button, Direction::Press));
                    steps.push(Step::Button(button, Direction::Release));
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
                    "Ctrl" => Key::Control,
                    "Shift" => Key::Shift,
                    _ => return Err("Only Ctrl and Shift key modifiers are supported".into()),
                };
                if modifiers.contains(&modifier) {
                    return Err("Duplicate key modifier".into());
                }
                modifiers.push(modifier);
                steps.push(Step::Key(modifier, Direction::Press));
            }
            steps.push(Step::Key(key, Direction::Press));
            steps.push(Step::Key(key, Direction::Release));
            for modifier in modifiers.into_iter().rev() {
                steps.push(Step::Key(modifier, Direction::Release));
            }
        }
        "text" => {
            if !observed.editable_focus {
                return Err("Observe and focus an editable CAD text control before typing".into());
            }
            let text = request.text.as_deref().ok_or("Text input requires text")?;
            if text.is_empty() || text.chars().count() > 512 || text.chars().any(char::is_control) {
                return Err("Text must contain 1-512 printable Unicode characters".into());
            }
            steps.push(Step::Text(text));
        }
        _ => return Err("Unknown computer control action".into()),
    }
    Ok(steps)
}

fn key_code(key: &str) -> Result<Key, String> {
    let code = match key {
        "Enter" => Key::Return,
        "Escape" => Key::Escape,
        "Tab" => Key::Tab,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Insert" => Key::Insert,
        "ArrowLeft" => Key::LeftArrow,
        "ArrowRight" => Key::RightArrow,
        "ArrowUp" => Key::UpArrow,
        "ArrowDown" => Key::DownArrow,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Space" => Key::Space,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        _ if key.len() == 1 && key.as_bytes()[0].is_ascii_alphanumeric() => {
            Key::Other(key.as_bytes()[0].to_ascii_uppercase() as u32)
        }
        _ => {
            return Err(
                "Unsupported key; use a named navigation key, F1-F12, letter or digit".into(),
            )
        }
    };
    Ok(code)
}
