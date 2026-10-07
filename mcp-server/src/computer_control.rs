//! Computer input is a separate OS surface, never an engine or script operation.
use serde_json::{json, Value};

#[cfg(all(windows, feature = "native-computer-control"))]
mod windows;
#[cfg(all(windows, feature = "native-computer-control"))]
pub(crate) use windows::ComputerControl;

#[cfg(not(all(windows, feature = "native-computer-control")))]
#[derive(Default)]
pub(crate) struct ComputerControl {}

#[cfg(not(all(windows, feature = "native-computer-control")))]
impl ComputerControl {
    pub(crate) fn call(&mut self, _: &Value, _: Option<&str>) -> Result<Value, String> {
        Err(json!({
            "code": if cfg!(feature = "native-computer-control") {
                "computer_control_unsupported_platform"
            } else {
                "computer_control_disabled"
            },
            "message": description(),
            "available": false,
            "required_feature": "native-computer-control",
            "supported_platform": "windows",
        })
        .to_string())
    }
}

pub(super) fn description() -> &'static str {
    if !cfg!(feature = "native-computer-control") {
        return "Native computer control is disabled in this build. Enable the native-computer-control Cargo feature on Windows to compile OS mouse and keyboard input. No actions are available while disabled.";
    }
    if !cfg!(windows) {
        return "Native computer control is unavailable on this platform. The native-computer-control Cargo feature currently supports Windows only; no actions are available here.";
    }
    "Windows computer control implemented in Rust with real OS mouse and keyboard input. action=observe returns the presented interface, exact active desktop owner, physical client bounds and a short-lived one-shot observation token. A minimized or unpresented window returns focus_only=true without qualified controls/coordinates. All other actions require that token; focus restores/activates only that window, then observe again before input. click/double_click/drag/wheel take physical client-pixel points from capture; key accepts Ctrl/Shift chords and named keys; text types Unicode into the currently focused editable text control. Input rejects changed documents, geometry, layouts, replaced processes, held keys, foreign foreground windows and occluded pointer targets. GUI and MCP must run the same clean build and executable path. An input_sent receipt confirms OS insertion only: observe/capture afterward to verify the visible result. No external helper, scripts, arbitrary applications or direct model commands."
}

pub(super) fn schema() -> Value {
    let point = json!({"type":"array","items":{"type":"integer"},"minItems":2,"maxItems":2});
    json!({"type":"object","description":description(),
        "x-limo-cad-availability":{
            "feature":"native-computer-control",
            "feature_enabled":cfg!(feature = "native-computer-control"),
            "platform_supported":cfg!(windows),
            "available":cfg!(all(windows, feature = "native-computer-control")),
        },
        "additionalProperties":false,"required":["action"],"properties":{
        "action":{"type":"string","enum":["observe","focus","click","double_click","drag","wheel","key","text"]},
        "session_id":{"type":"string","description":"Explicit current active desktop session, otherwise use the attached session."},
        "observation":{"type":"string","description":"One-shot token from observe, valid for 60 seconds; observe again after every action."},
        "point":point,"to":point,
        "button":{"type":"string","enum":["left","middle","right"],"default":"left"},
        "delta":{"type":"integer","minimum":-1200,"maximum":1200,"multipleOf":120,"description":"Wheel delta in whole Windows mouse notches: multiples of 120, positive scrolls up."},
        "key":{"type":"string","description":"Enter, Escape, Tab, Backspace, Delete, arrows, Home/End/PageUp/PageDown, F1–F12 or an ASCII letter/digit, optionally prefixed Ctrl+ or Shift+."},
        "text":{"type":"string","maxLength":512,"description":"Unicode text for the observed focused editable text control; no control characters."}
    }})
}
