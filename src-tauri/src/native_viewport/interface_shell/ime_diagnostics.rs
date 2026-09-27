//! Bounded, opt-in evidence of real native IME delivery. This observer does not
//! manufacture events or participate in editing, focus, or input-source choice.
use super::{InterfaceControl, NativeInterfaceAction, NativeInterfaceHandle};
use bevy::{
    prelude::*,
    text::EditableText,
    window::{Ime, WindowEvent},
};
use serde_json::{json, Value};

const MAX_EVENTS: usize = 256;
const MAX_VALUE_BYTES: usize = 4096;

pub(super) fn owner_snapshot(owner: &nbcad_interface::DocumentContext) -> Value {
    json!({"window_id":owner.window_id, "document_id":owner.document_id, "epoch":owner.epoch})
}

pub(super) struct Trace {
    events: Vec<Value>,
    overflow: bool,
}

impl Trace {
    #[cfg(test)]
    pub(super) fn for_test() -> Self {
        Self {
            events: Vec::new(),
            overflow: false,
        }
    }

    pub(super) fn opt_in() -> Option<Self> {
        let matches = |key, expected| std::env::var(key).as_deref() == Ok(expected);
        (matches("NBCAD_NATIVE_IME_TEST", "macos-japanese")
            && matches("GITHUB_ACTIONS", "true")
            && matches("RUNNER_OS", "macOS")
            && matches("RUNNER_ENVIRONMENT", "github-hosted")
            && matches("GITHUB_REPOSITORY", "jackControls/noBS-CAD")
            && cfg!(target_os = "macos"))
        .then(|| Self {
            events: Vec::new(),
            overflow: false,
        })
    }

    fn push(&mut self, mut event: Value) {
        if self.events.len() >= MAX_EVENTS
            || event["value"]
                .as_str()
                .is_some_and(|value| value.len() > MAX_VALUE_BYTES)
        {
            self.overflow = true;
            return;
        }
        event["sequence"] = json!(self.events.len() + 1);
        self.events.push(event);
    }

    pub(super) fn snapshot(&self) -> Value {
        json!({"source":"received Bevy WindowEvent::Ime", "overflow":self.overflow,
            "event_limit":MAX_EVENTS, "events":self.events, "current":self.events.last()})
    }
}

pub(super) fn received(
    world: &World,
    handle: &NativeInterfaceHandle,
    action: &NativeInterfaceAction,
    event: &WindowEvent,
) {
    let WindowEvent::Ime(ime) = event else { return };
    let Ok(mut shared) = handle.shared.lock() else {
        return;
    };
    let Some(trace) = &mut shared.ime_diagnostics else {
        return;
    };
    let (kind, window, value, cursor) = match ime {
        Ime::Preedit {
            window,
            value,
            cursor,
        } => ("preedit", *window, value.as_str(), *cursor),
        Ime::Commit { window, value } => ("commit", *window, value.as_str(), None),
        Ime::Enabled { window } => ("enabled", *window, "", None),
        Ime::Disabled { window } => ("disabled", *window, "", None),
    };
    let entity = Entity::from_bits(action.control.key.0);
    let Some(editor) = world.get::<EditableText>(entity) else {
        return;
    };
    trace.push(
        json!({"kind":kind, "window_entity":window.to_bits(), "value":value,
        "cursor":cursor, "context":owner_snapshot(&action.context),
        "control_key":action.control.key.0, "binding":action.control.binding(),
        "control_label":world.get::<InterfaceControl>(entity).map(|control| &control.label),
        "composing":editor.is_composing(), "appkit":appkit_input_context(window)}),
    );
}

#[cfg(not(target_os = "macos"))]
fn appkit_input_context(_entity: Entity) -> Value {
    Value::Null
}

#[cfg(target_os = "macos")]
fn appkit_input_context(entity: Entity) -> Value {
    use objc2::{msg_send, runtime::AnyObject, MainThreadMarker};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    if MainThreadMarker::new().is_none() {
        return json!({"error":"not on AppKit main thread"});
    }
    bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        let Some(window) = windows.get_window(entity) else {
            return Value::Null;
        };
        let Ok(raw) = window.window_handle() else {
            return Value::Null;
        };
        let RawWindowHandle::AppKit(raw) = raw.as_raw() else {
            return Value::Null;
        };
        // The Winit-owned NSView and returned autoreleased objects remain alive
        // throughout this main-thread read. No AppKit state is changed.
        unsafe {
            let view = raw.ns_view.as_ptr().cast::<AnyObject>();
            let context: *mut AnyObject = msg_send![view, inputContext];
            let native_window: *mut AnyObject = msg_send![view, window];
            if context.is_null() || native_window.is_null() {
                return Value::Null;
            }
            let source: *mut AnyObject = msg_send![context, selectedKeyboardInputSource];
            let source_id = if source.is_null() {
                None
            } else {
                let utf8: *const std::ffi::c_char = msg_send![source, UTF8String];
                (!utf8.is_null()).then(|| {
                    std::ffi::CStr::from_ptr(utf8)
                        .to_string_lossy()
                        .into_owned()
                })
            };
            let first: *mut AnyObject = msg_send![native_window, firstResponder];
            let key: bool = msg_send![native_window, isKeyWindow];
            let number: isize = msg_send![native_window, windowNumber];
            let scale: f64 = msg_send![native_window, backingScaleFactor];
            json!({"source_id":source_id, "window_number":number, "key_window":key,
                "first_responder_is_view":first == view, "backing_scale":scale})
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn received_ime_trace_is_bounded_and_keeps_order_without_truncating_text() {
        let mut trace = Trace {
            events: Vec::new(),
            overflow: false,
        };
        trace.push(json!({"kind":"preedit", "value":"はる", "composing":true}));
        trace.push(json!({"kind":"commit", "value":"はる", "composing":false}));
        assert_eq!(trace.snapshot()["events"][1]["sequence"], 2);
        trace.push(json!({"value":"x".repeat(MAX_VALUE_BYTES + 1)}));
        assert!(trace.overflow);
        assert_eq!(trace.events.len(), 2);
        for _ in 0..MAX_EVENTS {
            trace.push(json!({"value":""}));
        }
        assert_eq!(trace.events.len(), MAX_EVENTS);
        assert_eq!(trace.events[0]["value"], "はる");
    }
}
