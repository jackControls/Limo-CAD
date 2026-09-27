# Disposable macOS stock-control IME probe

Use the registered workflow on the reviewed feature branch:

```sh
gh workflow run native-host-tests.yml --repo jackControls/noBS-CAD --ref feat/bevy-interface -f ime-probe-only=true -f ime-probe-macos=true
```

This defaults to inventory of installed TIS input sources, enabled/selected
state, image/session, and TCC preflights. It compiles a small AppKit helper on
`macos-15`; it builds no CAD code. All existing native/package jobs are skipped.
`ime-probe-macos` has no effect without `ime-probe-only`. macOS, Windows probe,
and ordinary native runs use different concurrency groups.

To additionally enable the installed Apple Japanese Romaji/Hiragana source and
exercise it on the disposable runner:

```sh
gh workflow run native-host-tests.yml --repo jackControls/noBS-CAD --ref feat/bevy-interface -f ime-probe-only=true -f ime-probe-macos=true -f ime-provision-japanese=true -f ime-exercise=true
```

Unlike the Windows capability step, the macOS enable flag only enables the
installed `com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese` source. Missing or
ambiguous source identity fails with its inventory; it never installs an IME or
guesses another language. The selected source and exact enabled-state set are
restored, including a parent source enabled as a side effect. Restoration errors
fail the probe. Input-source changes affect the disposable user/session and are
not claimed to be process-private.

Both script and executable require the expected GitHub-hosted macOS repository
and numeric run ID, with evidence inside canonical `RUNNER_TEMP`. The wrapper
requires a fresh directory. Do not run this on a personal Mac by spoofing those
guards. No TCC reset, permission prompt, preference-file write, or paste is used.

The owned `NSTextView` logs real `keyDown`, `setMarkedText`, `insertText`,
`unmarkText`, and first-rectangle callbacks while forwarding normal AppKit
behavior. Real CoreGraphics virtual keys type `haru`; Control-J normalizes the
provisional conversion to `はる`. The test requires marked text with no committed
content, one Return commit, a second composition, and Escape with no extra commit
or remaining marked text. NSTextView storage includes provisional text, so the
committed value is computed by excluding its marked range. Empty callback
notifications are retained but do not count as text commits.

The field's own input context must acknowledge the selected Japanese source;
each key checks frontmost PID/key window/first responder. The input state machine
has a 30-second deadline. Unexpected Escape/live-conversion behavior is retained
as a failure, never repaired by manually clearing the field. All callback/key
sequences and cleanup status are retained in `report.json`; the wrapper retains
compile/run logs and hashes even on failure. The job is bounded at ten minutes.

**`stock-control-ime-feasible` proves only this stock control and IME session.**
It does not validate Bevy, candidate-popup ownership/placement/pixels, physical
keyboard hardware, or DPI transitions. First-rect callbacks are anchor evidence,
not a capture of the real candidate window. No screenshots are taken by this
prerequisite probe.

The Swift/AppKit source cannot be compiled on the Windows development host;
the short CI job must establish SDK compilation and runtime feasibility.

References: [Apple text-input protocol](https://developer.apple.com/documentation/appkit/nstextinputclient),
[input-source enable/disable APIs](https://developer.apple.com/library/archive/qa/qa1810/_index.html),
[input context](https://developer.apple.com/documentation/appkit/nstextinputcontext),
[Japanese conversion keys](https://support.apple.com/en-gb/guide/japanese-input-method/jpim10263/mac).
