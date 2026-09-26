# Native platform input checks

`cargo xtask test-mcp native-platform --desktop-input --server ABSOLUTE_NATIVE_BINARY --out EMPTY_ABSOLUTE_DIRECTORY`

Build the binary with `--features dev-bevy-host`. Use a disposable desktop: the
fixture focuses its own newly spawned window and uses the system clipboard. It
restores prior text clipboard contents in memory, but not other clipboard formats.
No existing document or window is selected. The fresh process owns its private
session directory and is stopped when the fixture exits.

MCP opens Rename and focuses the existing text field. Shortcut keys then go
through Windows `SendInput`, macOS CoreGraphics, or Linux X11 XTEST (`xdotool`),
into the real Winit event loop. Assertions cover select-all without a literal
letter, selection collapse, OS copy/paste, and Unicode round trips. Captures use
the product's Bevy window capture endpoint. `report.json` describes the event
source and results; captures still require visual review for caret/selection and
layout correctness.

The opt-in `Native sketch visual regressions` workflow runs this fixture on all
three OSes, plus the separate GPU sketch-boundary test. Linux needs `xclip`,
`xdotool`, and a working X11 display; CI runs Xvfb with Mesa at scale factors 1 and
2. macOS requires Accessibility permission to post CoreGraphics events; a TCC
denial is a failing check with a specific error, not a skipped input test.
Windows needs an interactive desktop that permits focusing the owned window.

These tests do not prove IME preedit/candidate/commit behavior, physical keyboard
layouts, Wayland input, moving between monitors with different DPI, or visual
correctness without reviewing the captured pixels. Unicode paste is not IME.
