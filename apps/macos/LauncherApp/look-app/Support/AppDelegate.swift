import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if let window = sender.windows.first {
            sender.activate(ignoringOtherApps: true)
            window.makeKeyAndOrderFront(nil)
        }
        NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
        return true
    }
}
