//! Computer input is a separate OS surface, never an engine or script operation.
use serde_json::{json, Value};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub(super) use windows::ComputerControl;

#[cfg(not(windows))]
#[derive(Default)]
pub(super) struct ComputerControl;

#[cfg(not(windows))]
impl ComputerControl {
    pub(super) fn call(&mut self, _: &Value, _: Option<&str>) -> Result<Value, String> {
        Err("Native computer control is currently supported on Windows only".into())
    }
}

pub(super) fn schema() -> Value {
    let point = json!({"type":"array","items":{"type":"integer"},"minItems":2,"maxItems":2});
    json!({"type":"object","additionalProperties":false,"required":["action"],"properties":{
        "action":{"type":"string","enum":["observe","focus","click","double_click","drag","wheel","key","text"]},
        "session_id":{"type":"string","description":"Explicit current active desktop session, otherwise use the attached session."},
        "observation":{"type":"string","description":"One-shot token from observe, valid for 60 seconds; observe again after every action."},
        "point":point,"to":point,
        "button":{"type":"string","enum":["left","middle","right"],"default":"left"},
        "delta":{"type":"integer","minimum":-1200,"maximum":1200,"description":"Wheel delta in Windows units (120 per notch), positive scrolls up."},
        "key":{"type":"string","description":"Enter, Escape, Tab, Backspace, Delete, arrows, Home/End/PageUp/PageDown, F1–F12 or an ASCII letter/digit, optionally prefixed Ctrl+ or Shift+."},
        "text":{"type":"string","maxLength":512,"description":"Unicode text for the observed focused editable text control; no control characters."}
    }})
}
