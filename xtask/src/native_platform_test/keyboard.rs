//! Refocus an observed owned window before a key, without retrying input.
use anyhow::{ensure, Result};
use serde_json::{json, Value};

pub(super) fn focus_target(
    mut observe: impl FnMut() -> Result<Value>,
    mut send: impl FnMut(&Value, Value) -> Result<Value>,
) -> Result<(Value, Value)> {
    let observed = observe()?;
    activate_target(&observed, &mut observe, &mut send)
}

fn activate_target(
    observed: &Value,
    observe: &mut impl FnMut() -> Result<Value>,
    send: &mut impl FnMut(&Value, Value) -> Result<Value>,
) -> Result<(Value, Value)> {
    let receipt = send(observed, json!({"action":"focus"}))?;
    ensure!(
        receipt["status"] == "focused" && receipt["owner"] == observed["owner"],
        "Owned CAD activation did not complete; no key was sent: {receipt}"
    );
    let activated = observe()?;
    require_same_target(observed, &activated)?;
    ensure!(
        activated["foreground"] == true,
        "CAD lost foreground after activation; no key was sent"
    );
    Ok((activated, receipt))
}

pub(super) fn send_key(
    key: &str,
    mut observe: impl FnMut() -> Result<Value>,
    mut send: impl FnMut(&Value, Value) -> Result<Value>,
) -> Result<Value> {
    let mut observed = observe()?;
    let foreground = observed["foreground"]
        .as_bool()
        .ok_or_else(|| anyhow::anyhow!("Owned observation omitted foreground state"))?;
    if !foreground {
        // Nothing has been typed. Activation consumes its own observation;
        // the key must use a new observation of the same document and window.
        observed = activate_target(&observed, &mut observe, &mut send)?.0;
    }
    // Any denial or partial receipt remains fatal. Repeating a key could edit
    // twice or act on a different control, even if the next observation looks OK.
    let receipt = send(&observed, json!({"action":"key","key":key}))?;
    ensure!(
        receipt["status"] == "input_sent",
        "Key did not complete; do not retry blindly: {receipt}"
    );
    Ok(receipt)
}

pub(super) fn require_same_target(expected: &Value, actual: &Value) -> Result<()> {
    ensure!(
        actual["owner"] == expected["owner"]
            && actual["window_handle"] == expected["window_handle"]
            && actual["target_kind"] == expected["target_kind"],
        "CAD document or window changed during foreground preparation; no key was sent"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::VecDeque};

    fn observation(foreground: bool, token: &str) -> Value {
        json!({"owner":{"pid":91,"process_instance_id":"owned-process",
            "session_id":"owned-session","document_id":"owned-document","generation":7},
            "window_handle":32,"target_kind":"bevy_window","foreground":foreground,
            "observation":token})
    }

    fn exercise(
        observations: Vec<Value>,
        receipts: Vec<Result<Value>>,
    ) -> (Result<Value>, Vec<Value>) {
        let mut observations = VecDeque::from(observations);
        let mut receipts = VecDeque::from(receipts);
        let calls = RefCell::new(Vec::new());
        let result = send_key(
            "Ctrl+V",
            || {
                calls.borrow_mut().push(json!({"action":"observe"}));
                Ok(observations
                    .pop_front()
                    .expect("Unexpected extra observation"))
            },
            |observed, mut request| {
                request["observation"] = observed["observation"].clone();
                calls.borrow_mut().push(request);
                receipts.pop_front().expect("Unexpected input retry")
            },
        );
        (result, calls.into_inner())
    }

    fn focused() -> Result<Value> {
        Ok(json!({"status":"focused","owner":observation(false, "old")["owner"]}))
    }

    #[test]
    fn explicit_field_preparation_reobserves_the_same_target_without_sending_keys() {
        let mut observations =
            VecDeque::from([observation(false, "old"), observation(true, "prepared")]);
        let calls = RefCell::new(Vec::new());
        let (prepared, receipt) = focus_target(
            || {
                calls.borrow_mut().push("observe");
                Ok(observations.pop_front().expect("Unexpected observation"))
            },
            |observed, request| {
                assert_eq!(observed["observation"], "old");
                assert_eq!(request, json!({"action":"focus"}));
                calls.borrow_mut().push("focus");
                focused()
            },
        )
        .unwrap();
        assert_eq!(prepared["observation"], "prepared");
        assert_eq!(receipt["status"], "focused");
        assert_eq!(calls.into_inner(), ["observe", "focus", "observe"]);
    }

    #[test]
    fn explicit_field_preparation_rejects_denial_and_changed_owner_or_foreground() {
        for change in ["denied", "document", "foreground"] {
            let mut prepared = observation(true, "prepared");
            match change {
                "document" => prepared["owner"]["document_id"] = json!("other-document"),
                "foreground" => prepared["foreground"] = json!(false),
                _ => (),
            }
            let mut observations = VecDeque::from([observation(false, "old"), prepared]);
            let mut activations = 0;
            let result = focus_target(
                || Ok(observations.pop_front().expect("Unexpected observation")),
                |_, request| {
                    assert_eq!(request["action"], "focus");
                    activations += 1;
                    if change == "denied" {
                        Ok(json!({"status":"failed"}))
                    } else {
                        focused()
                    }
                },
            );
            assert!(result.is_err(), "{change}");
            assert_eq!(activations, 1, "{change}");
        }
    }

    #[test]
    fn already_foreground_sends_the_key_once_without_activation() {
        let (result, calls) = exercise(
            vec![observation(true, "ready")],
            vec![Ok(json!({"status":"input_sent"}))],
        );
        assert!(result.is_ok());
        assert_eq!(
            calls,
            vec![
                json!({"action":"observe"}),
                json!({"action":"key","key":"Ctrl+V","observation":"ready"})
            ]
        );
    }

    #[test]
    fn lost_foreground_requires_activation_and_a_new_observation_before_one_key() {
        let (result, calls) = exercise(
            vec![observation(false, "old"), observation(true, "new")],
            vec![focused(), Ok(json!({"status":"input_sent"}))],
        );
        assert!(result.is_ok());
        assert_eq!(
            calls,
            vec![
                json!({"action":"observe"}),
                json!({"action":"focus","observation":"old"}),
                json!({"action":"observe"}),
                json!({"action":"key","key":"Ctrl+V","observation":"new"})
            ]
        );
    }

    #[test]
    fn denied_activation_never_sends_or_retries_a_key() {
        for receipt in [
            Err(anyhow::anyhow!("Windows denied activation")),
            Ok(json!({"status":"failed"})),
            Ok(json!({"status":"focused","owner":{"pid":92}})),
        ] {
            let (result, calls) = exercise(vec![observation(false, "old")], vec![receipt]);
            assert!(result.is_err());
            assert_eq!(calls.len(), 2);
            assert_eq!(calls[1]["action"], "focus");
        }
    }

    #[test]
    fn replaced_owner_document_or_window_after_activation_never_receives_a_key() {
        for (pointer, replacement) in [
            ("/owner/pid", json!(92)),
            ("/owner/process_instance_id", json!("replaced")),
            ("/owner/session_id", json!("replaced")),
            ("/owner/document_id", json!("another-tab")),
            ("/owner/generation", json!(8)),
            ("/window_handle", json!(33)),
            ("/target_kind", json!("native_dialog")),
            ("/foreground", json!(false)),
        ] {
            let mut activated = observation(true, "new");
            *activated.pointer_mut(pointer).unwrap() = replacement;
            let (result, calls) =
                exercise(vec![observation(false, "old"), activated], vec![focused()]);
            assert!(result.is_err(), "{pointer}");
            assert_eq!(calls.len(), 3, "{pointer}");
            assert_eq!(calls[2]["action"], "observe");
        }
    }

    #[test]
    fn failed_or_partial_key_is_never_repeated() {
        for receipt in [
            Err(anyhow::anyhow!("Foreground changed before input")),
            Ok(json!({"status":"partial_input","completed":1})),
            Ok(json!({"status":"failed"})),
        ] {
            let (result, calls) = exercise(vec![observation(true, "ready")], vec![receipt]);
            assert!(result.is_err());
            assert_eq!(calls.len(), 2);
            assert_eq!(calls[1]["action"], "key");
        }
    }

    #[test]
    fn missing_foreground_state_never_sends_anything() {
        let mut observed = observation(true, "ready");
        observed.as_object_mut().unwrap().remove("foreground");
        let (result, calls) = exercise(vec![observed], vec![]);
        assert!(result.is_err());
        assert_eq!(calls, vec![json!({"action":"observe"})]);
    }
}
