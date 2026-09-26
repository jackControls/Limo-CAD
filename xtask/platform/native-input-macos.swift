import AppKit
import CoreGraphics
import Foundation

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}
guard CommandLine.arguments.count == 3, let ownedPID = Int32(CommandLine.arguments[1]) else {
    fail("Expected an owned PID and input operation")
}
let operation = CommandLine.arguments[2]
let pasteboard = NSPasteboard.general
if operation == "clipboard-read" {
    FileHandle.standardOutput.write(Data((pasteboard.string(forType: .string) ?? "").utf8))
    exit(0)
}
if operation == "clipboard-write" {
    guard let value = String(data: FileHandle.standardInput.readDataToEndOfFile(), encoding: .utf8) else { fail("Clipboard input is not UTF-8") }
    pasteboard.clearContents()
    guard pasteboard.setString(value, forType: .string) else { fail("Cannot set the OS clipboard") }
    exit(0)
}
guard CGPreflightPostEventAccess() else {
    fail("macOS does not grant this helper Accessibility event-posting permission (TCC). No keyboard events were injected; this is not a platform input pass.")
}
guard let application = NSRunningApplication(processIdentifier: ownedPID) else { fail("Owned native process is no longer running") }
application.activate(options: [.activateIgnoringOtherApps])
let deadline = Date().addingTimeInterval(5)
while NSWorkspace.shared.frontmostApplication?.processIdentifier != ownedPID && Date() < deadline {
    Thread.sleep(forTimeInterval: 0.05)
}
guard NSWorkspace.shared.frontmostApplication?.processIdentifier == ownedPID else { fail("Cannot focus the owned native process") }
if operation == "focus" { exit(0) }
let key: CGKeyCode
let command: Bool
switch operation {
case "select-all": key = 0; command = true
case "copy": key = 8; command = true
case "paste": key = 9; command = true
case "right": key = 124; command = false
default: fail("Unknown input operation \(operation)")
}
guard let source = CGEventSource(stateID: .combinedSessionState) else { fail("Cannot create OS event source") }
func send(_ key: CGKeyCode, _ down: Bool, _ flags: CGEventFlags) {
    guard let event = CGEvent(keyboardEventSource: source, virtualKey: key, keyDown: down) else { fail("Cannot create OS key event") }
    event.flags = flags
    event.postToPid(ownedPID)
    Thread.sleep(forTimeInterval: 0.025)
}
if command { send(55, true, .maskCommand) }
send(key, true, command ? .maskCommand : [])
send(key, false, command ? .maskCommand : [])
if command { send(55, false, []) }
