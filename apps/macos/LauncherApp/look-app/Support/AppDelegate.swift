import AppKit

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        if shouldTerminateDuplicateInstance() {
            NSApp.terminate(nil)
            return
        }

        NSApp.setActivationPolicy(.accessory)
    }

    private func shouldTerminateDuplicateInstance() -> Bool {
        guard let bundleIdentifier = Bundle.main.bundleIdentifier else {
            return false
        }

        let currentPID = ProcessInfo.processInfo.processIdentifier
        let runningApps = NSRunningApplication.runningApplications(withBundleIdentifier: bundleIdentifier)
        guard runningApps.count > 1 else {
            return false
        }

        guard let primaryApp = runningApps.min(by: { $0.processIdentifier < $1.processIdentifier }) else {
            return false
        }

        guard primaryApp.processIdentifier != currentPID else {
            return false
        }

        primaryApp.activate(options: [.activateAllWindows])
        return true
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        if let window = sender.windows.first {
            sender.activate(ignoringOtherApps: true)
            window.makeKeyAndOrderFront(nil)
        }
        NotificationCenter.default.post(name: .lookActivateLauncherRequested, object: nil)
        return true
    }

    func applicationDidBecomeActive(_ notification: Notification) {
        DispatchQueue.main.async {
            if let app = notification.object as? NSApplication,
                let window = app.windows.first
            {
                app.activate(ignoringOtherApps: true)
                window.makeKeyAndOrderFront(nil)
            }
            NotificationCenter.default.post(name: .lookActivateLauncherRequested, object: nil)
        }
    }

    func applicationWillResignActive(_ notification: Notification) {
        NotificationCenter.default.post(name: .lookHideLauncherRequested, object: nil)
    }
}
