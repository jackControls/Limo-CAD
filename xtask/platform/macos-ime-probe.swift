// QA-only stock NSTextView. Observes real AppKit input; never injects text/IME callbacks.
import AppKit
import Carbon
import CoreGraphics
import Foundation

struct ProbeError: Error, CustomStringConvertible { let description: String }
func require(_ condition: Bool, _ message: String) throws {
    if !condition { throw ProbeError(description: message) }
}
func property(_ source: TISInputSource, _ key: CFString) -> AnyObject? {
    guard let pointer = TISGetInputSourceProperty(source, key) else { return nil }
    return Unmanaged<AnyObject>.fromOpaque(pointer).takeUnretainedValue()
}
func sourceID(_ source: TISInputSource) -> String {
    property(source, kTISPropertyInputSourceID) as? String ?? ""
}
func sources() -> [TISInputSource] {
    TISCreateInputSourceList(nil, true).takeRetainedValue() as NSArray as! [TISInputSource]
}
func describe(_ source: TISInputSource) -> [String: Any] {
    ["id": sourceID(source), "bundle": property(source, kTISPropertyBundleID) as? String ?? "",
     "name": property(source, kTISPropertyLocalizedName) as? String ?? "",
     "type": property(source, kTISPropertyInputSourceType) as? String ?? "",
     "mode": property(source, kTISPropertyInputModeID) as? String ?? "",
     "languages": property(source, kTISPropertyInputSourceLanguages) as? [String] ?? [],
     "enabled": property(source, kTISPropertyInputSourceIsEnabled) as? Bool ?? false,
     "selectable": property(source, kTISPropertyInputSourceIsSelectCapable) as? Bool ?? false]
}
func rangeJSON(_ range: NSRange) -> [String: Int] { ["location": range.location, "length": range.length] }
func stringValue(_ value: Any) -> String {
    (value as? NSAttributedString)?.string ?? (value as? String) ?? String(describing: value)
}

final class ObservedTextView: NSTextView {
    var received: [[String: Any]] = []
    var inserted: [String] = []
    var markedCount = 0
    var save: (() -> Void)?
    func state() -> [String: Any] {
        let value = string as NSString, marked = markedRange()
        let valid = marked.location != NSNotFound && marked.location <= value.length &&
            marked.length <= value.length - marked.location
        return ["storage": string, "has_marked_text": hasMarkedText(),
                "marked_range": rangeJSON(marked), "selection": rangeJSON(selectedRange()),
                "marked_text": valid ? value.substring(with: marked) : "",
                "committed": valid ? value.replacingCharacters(in: marked, with: "") : string,
                "input_source": inputContext?.selectedKeyboardInputSource ?? ""]
    }
    func record(_ method: String, _ fields: [String: Any] = [:]) {
        guard received.count < 1024 else { return }
        var entry = fields
        entry["method"] = method; entry["uptime"] = ProcessInfo.processInfo.systemUptime
        entry["state"] = state(); received.append(entry); save?()
    }
    override func keyDown(with event: NSEvent) {
        record("keyDown", ["key_code": event.keyCode, "flags": event.modifierFlags.rawValue])
        super.keyDown(with: event)
    }
    override func setMarkedText(_ value: Any, selectedRange: NSRange, replacementRange: NSRange) {
        markedCount += 1
        record("setMarkedText:before", ["text": stringValue(value), "selected": rangeJSON(selectedRange),
                                       "replacement": rangeJSON(replacementRange)])
        super.setMarkedText(value, selectedRange: selectedRange, replacementRange: replacementRange)
        record("setMarkedText:after")
    }
    override func insertText(_ value: Any, replacementRange: NSRange) {
        let text = stringValue(value)
        if !text.isEmpty { inserted.append(text) }
        record("insertText:before", ["text": text, "replacement": rangeJSON(replacementRange)])
        super.insertText(value, replacementRange: replacementRange)
        record("insertText:after")
    }
    override func unmarkText() {
        record("unmarkText:before"); super.unmarkText(); record("unmarkText:after")
    }
    override func firstRect(forCharacterRange range: NSRange, actualRange: NSRangePointer?) -> NSRect {
        let rect = super.firstRect(forCharacterRange: range, actualRange: actualRange)
        record("firstRect", ["requested": rangeJSON(range),
                             "screen_rect": ["x": rect.minX, "y": rect.minY, "width": rect.width, "height": rect.height]])
        return rect
    }
}

final class Probe {
    let out: URL
    var report: [String: Any] = ["schema_version": 1, "status": "started", "native_bevy_validated": false,
                                "candidate_placement": "not tested", "popup_pixels": "not captured"]
    var failure: String?
    init(_ out: URL) {
        self.out = out; report["started_utc"] = ISO8601DateFormatter().string(from: Date())
    }
    func save() {
        do {
            let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
            try data.write(to: out.appendingPathComponent("report.json"), options: .atomic)
        } catch { failure = "Cannot retain evidence: \(error)" }
    }
    func run(enable: Bool, exercise: Bool) throws {
        let environment = ProcessInfo.processInfo.environment
        report["requested"] = ["enable_japanese": enable, "exercise": exercise]
        report["environment"] = ["os": ProcessInfo.processInfo.operatingSystemVersionString,
            "image_os": environment["ImageOS"] ?? "", "image_version": environment["ImageVersion"] ?? "",
            "run_id": environment["GITHUB_RUN_ID"] ?? "", "sha": environment["GITHUB_SHA"] ?? "",
            "uid": getuid(), "pid": getpid(), "session": String(describing: CGSessionCopyCurrentDictionary())]
        let before = sources(), prior = TISCopyCurrentKeyboardInputSource().takeRetainedValue()
        report["before"] = before.map(describe); report["prior_source"] = describe(prior)
        report["event_posting_allowed"] = CGPreflightPostEventAccess()
        report["screen_capture_allowed"] = CGPreflightScreenCaptureAccess()
        report["accessibility_trusted"] = AXIsProcessTrusted()
        report["status"] = "inventory-complete"; save()
        if let error = failure { throw ProbeError(description: error) }
        guard enable || exercise else { return }
        // Exact installed Apple Romaji-typing/Hiragana mode; fail with inventory if this SDK/OS differs.
        let matches = before.filter { sourceID($0) == "com.apple.inputmethod.Kotoeri.RomajiTyping.Japanese" }
        try require(matches.count == 1, "Expected one installed Apple Japanese Romaji-typing source; inspect inventory")
        let target = matches[0], targetID = sourceID(target)
        try require(property(target, kTISPropertyInputSourceIsSelectCapable) as? Bool == true, "Japanese mode is not selectable")
        let initiallyEnabled = property(target, kTISPropertyInputSourceIsEnabled) as? Bool == true
        let initiallyEnabledIDs = Set(before.filter { property($0, kTISPropertyInputSourceIsEnabled) as? Bool == true }.map(sourceID))
        var newlyEnabled: [TISInputSource] = [], selectedByProbe = false
        defer {
            var cleanup: [String: Any] = [:]
            if selectedByProbe {
                let result = TISSelectInputSource(prior)
                let restored = sourceID(TISCopyCurrentKeyboardInputSource().takeRetainedValue())
                cleanup["restore_status"] = result; cleanup["restored_source"] = restored
                if result != noErr || restored != sourceID(prior) { failure = "Could not restore prior input source" }
            }
            // Enabling a mode can also enable its parent input method. Restore
            // the observed delta, never disable a source enabled before this probe.
            var disabled: [[String: Any]] = []
            for source in newlyEnabled.sorted(by: { sourceID($0).count > sourceID($1).count }) {
                let result = TISDisableInputSource(source)
                disabled.append(["id": sourceID(source), "status": result])
                if result != noErr { failure = "Could not disable a source enabled by the probe" }
            }
            cleanup["disabled"] = disabled
            let finalEnabled = Set(sources().filter { property($0, kTISPropertyInputSourceIsEnabled) as? Bool == true }.map(sourceID))
            cleanup["enabled_set_restored"] = finalEnabled == initiallyEnabledIDs
            if finalEnabled != initiallyEnabledIDs { failure = "Enabled input-source set was not restored exactly" }
            report["cleanup"] = cleanup; report["after"] = sources().map(describe); save()
        }
        if !initiallyEnabled {
            try require(enable, "Japanese source is disabled; explicitly enable it on this disposable runner")
            let result = TISEnableInputSource(target)
            newlyEnabled = sources().filter { !initiallyEnabledIDs.contains(sourceID($0)) && property($0, kTISPropertyInputSourceIsEnabled) as? Bool == true }
            report["enable_status"] = result; save()
            try require(result == noErr, "TISEnableInputSource failed with OSStatus \(result)")
            try require(sources().contains { sourceID($0) == targetID && property($0, kTISPropertyInputSourceIsEnabled) as? Bool == true }, "Source did not become enabled")
        }
        guard exercise else { report["status"] = "source-enable-feasible"; return }
        try require(CGPreflightPostEventAccess(), "TCC denies event posting; no input sent")
        let app = NSApplication.shared
        app.setActivationPolicy(.regular)
        let window = NSWindow(contentRect: NSRect(x: 160, y: 180, width: 640, height: 220),
                              styleMask: [.titled, .closable], backing: .buffered, defer: false)
        window.isReleasedWhenClosed = false; window.title = "noBS CAD disposable macOS IME probe"
        let field = ObservedTextView(frame: NSRect(x: 24, y: 40, width: 590, height: 140))
        field.font = NSFont.systemFont(ofSize: 24); field.isRichText = false
        window.contentView?.addSubview(field)
        window.makeKeyAndOrderFront(nil); window.makeFirstResponder(field)
        report["status"] = "exercise-in-progress"; report["window_number"] = window.windowNumber
        report["selected_source"] = describe(target); save()
        field.save = { [weak self, weak field] in
            guard let self = self, let field = field else { return }
            self.report["received"] = field.received; self.report["field"] = field.state(); self.save()
        }
        let started = ProcessInfo.processInfo.systemUptime
        var stage = 0, secondMarkedCount = 0, sent: [[String: Any]] = []
        var activationRequested = false
        guard let eventSource = CGEventSource(stateID: .combinedSessionState) else {
            throw ProbeError(description: "Cannot create OS event source")
        }
        func focus() throws {
            try require(NSWorkspace.shared.frontmostApplication?.processIdentifier == getpid() &&
                        window.isKeyWindow && window.firstResponder === field, "Owned stock field lost foreground/focus")
        }
        func key(_ code: CGKeyCode, _ down: Bool, _ flags: CGEventFlags = []) throws {
            try focus()
            guard let event = CGEvent(keyboardEventSource: eventSource, virtualKey: code, keyDown: down) else {
                throw ProbeError(description: "Cannot construct virtual-key event")
            }
            event.flags = flags; event.postToPid(getpid())
            sent.append(["key": code, "down": down, "flags": flags.rawValue,
                         "elapsed": ProcessInfo.processInfo.systemUptime - started])
            Thread.sleep(forTimeInterval: 0.025)
        }
        func tap(_ code: CGKeyCode) throws { try key(code, true); try key(code, false) }
        func haru() throws { for code in [CGKeyCode(4), 0, 15, 32] { try tap(code) } }
        func hiragana() throws {
            try key(59, true, .maskControl)
            defer { try? key(59, false) }
            try key(38, true, .maskControl); try key(38, false, .maskControl)
        }
        func stop(_ timer: Timer) {
            timer.invalidate(); app.stop(nil)
            // Wake NSApplication's event wait after stop from a run-loop timer.
            // This lifecycle event is never sent to the text-input client.
            if let wake = NSEvent.otherEvent(with: .applicationDefined, location: .zero,
                modifierFlags: [], timestamp: 0, windowNumber: window.windowNumber,
                context: nil, subtype: 0, data1: 0, data2: 0) {
                app.postEvent(wake, atStart: false)
            }
        }
        let timer = Timer(timeInterval: 0.25, repeats: true) { timer in
            do {
                try require(ProcessInfo.processInfo.systemUptime - started < 30, "IME deadline exceeded at stage \(stage)")
                try require(field.received.count < 1024, "Input callback trace exceeded its bound")
                if let error = self.failure { throw ProbeError(description: error) }
                if stage == 0 {
                    // Activation requested before NSApplication.run() can be lost
                    // while a command-line AppKit application finishes launching.
                    // Ask once from its running event loop, then require real focus.
                    self.report["activation"] = ["finished_launching": NSRunningApplication.current.isFinishedLaunching,
                        "running": app.isRunning, "active": app.isActive, "key_window": window.isKeyWindow,
                        "field_is_first_responder": window.firstResponder === field,
                        "frontmost_pid": NSWorkspace.shared.frontmostApplication?.processIdentifier ?? -1,
                        "requested": activationRequested]
                    if !activationRequested && NSRunningApplication.current.isFinishedLaunching {
                        if #available(macOS 14.0, *) { app.activate() }
                        else { app.activate(ignoringOtherApps: true) }
                        window.makeKeyAndOrderFront(nil); window.makeFirstResponder(field)
                        activationRequested = true
                    }
                    self.save()
                    if NSWorkspace.shared.frontmostApplication?.processIdentifier != getpid() || !window.isKeyWindow { return }
                    try focus(); selectedByProbe = true
                    let result = TISSelectInputSource(target)
                    self.report["select_status"] = result
                    try require(result == noErr, "TISSelectInputSource failed: \(result)")
                    stage = 1
                } else {
                    try focus()
                    let state = field.state()
                    if stage == 1 {
                        if field.inputContext?.selectedKeyboardInputSource != targetID { return }
                        try haru(); stage = 2
                    } else if stage == 2 && field.hasMarkedText() {
                        try hiragana(); stage = 3
                    } else if stage == 3 && state["marked_text"] as? String == "はる" {
                        try require(field.markedCount > 0 && state["committed"] as? String == "" && field.inserted.isEmpty,
                                    "Provisional keys changed committed text")
                        self.report["preedit"] = state; try tap(36); stage = 4
                    } else if stage == 4 && !field.hasMarkedText() && field.string == "はる" {
                        try require(field.inserted == ["はる"], "Return did not commit exactly once")
                        self.report["committed"] = state; secondMarkedCount = field.markedCount
                        try haru(); stage = 5
                    } else if stage == 5 && field.hasMarkedText() && field.markedCount > secondMarkedCount {
                        try hiragana(); stage = 6
                    } else if stage == 6 && state["marked_text"] as? String == "はる" {
                        try require(state["committed"] as? String == "はる" && field.inserted == ["はる"], "Second preedit changed committed text")
                        self.report["second_preedit"] = state; try tap(53); stage = 7
                    } else if stage == 7 && !field.hasMarkedText() {
                        try require(field.string == "はる" && field.inserted == ["はる"], "Escape changed committed text")
                        self.report["cancelled"] = state; self.report["status"] = "stock-control-ime-feasible"
                        stop(timer)
                    }
                }
                self.report["stage"] = stage; self.report["sent_keys"] = sent; self.save()
            } catch {
                self.failure = String(describing: error); stop(timer)
            }
        }
        RunLoop.main.add(timer, forMode: .common); app.run(); timer.invalidate()
        report["received"] = field.received; report["sent_keys"] = sent; report["final_field"] = field.state()
        report["elapsed_seconds"] = ProcessInfo.processInfo.systemUptime - started
        field.save = nil; window.close()
        if let error = failure { throw ProbeError(description: error) }
        try require(report["status"] as? String == "stock-control-ime-feasible", "Owned probe closed before completion")
    }
}

var probe: Probe?
do {
    let env = ProcessInfo.processInfo.environment, args = Array(CommandLine.arguments.dropFirst())
    try require(env["GITHUB_ACTIONS"] == "true" && env["RUNNER_OS"] == "macOS" &&
                env["RUNNER_ENVIRONMENT"] == "github-hosted" && env["GITHUB_REPOSITORY"] == "jackControls/noBS-CAD" &&
                env["GITHUB_RUN_ID"]?.range(of: "^[0-9]+$", options: .regularExpression) != nil, "Disposable GitHub macOS runner required")
    try require(args.count >= 2 && args[0] == "--out", "Expected --out directory")
    try require(args.dropFirst(2).allSatisfy { ["--enable-japanese", "--exercise"].contains($0) }, "Unknown probe option")
    guard let temp = env["RUNNER_TEMP"] else { throw ProbeError(description: "RUNNER_TEMP is absent") }
    let root = URL(fileURLWithPath: temp).resolvingSymlinksInPath().standardizedFileURL.path + "/"
    let out = URL(fileURLWithPath: args[1]).resolvingSymlinksInPath().standardizedFileURL
    try require(args[1].hasPrefix("/") && out.path.hasPrefix(root), "Output must be beneath RUNNER_TEMP")
    let current = Probe(out); probe = current; current.save()
    try current.run(enable: args.contains("--enable-japanese"), exercise: args.contains("--exercise"))
    if let error = current.failure { throw ProbeError(description: error) }
    current.report["finished_utc"] = ISO8601DateFormatter().string(from: Date()); current.save()
    if let error = current.failure { throw ProbeError(description: error) }
    print("macOS IME prerequisite: \(current.report["status"]!); Bevy/candidate pixels remain unvalidated")
} catch {
    probe?.report["status"] = "failed"; probe?.report["error"] = String(describing: error)
    probe?.report["finished_utc"] = ISO8601DateFormatter().string(from: Date()); probe?.save()
    FileHandle.standardError.write(Data((String(describing: error) + "\n").utf8)); exit(1)
}
