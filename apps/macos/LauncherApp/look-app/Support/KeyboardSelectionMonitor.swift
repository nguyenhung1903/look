import AppKit
import Foundation

final class KeyboardSelectionMonitor {
    private var monitor: Any?

    func start(
        onNext: @escaping () -> Void,
        onPrevious: @escaping () -> Void,
        onExitCommandMode: @escaping () -> Void,
        onWebSearch: @escaping () -> Void
    ) {
        guard monitor == nil else { return }

        monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            if event.keyCode == 36 && event.modifierFlags.contains(.command) {
                onWebSearch()
                return nil
            }

            if event.modifierFlags.contains(.command)
                || event.modifierFlags.contains(.option)
                || event.modifierFlags.contains(.control)
            {
                return event
            }

            if event.keyCode == 53 && event.modifierFlags.contains(.shift) {
                onExitCommandMode()
                return nil
            }

            if event.keyCode == 126 {
                onPrevious()
                return nil
            }

            if event.keyCode == 125 {
                onNext()
                return nil
            }

            guard event.keyCode == 48 else {
                return event
            }

            if event.modifierFlags.contains(.shift) {
                onPrevious()
            } else {
                onNext()
            }
            return nil
        }
    }

    func stop() {
        guard let monitor else { return }
        NSEvent.removeMonitor(monitor)
        self.monitor = nil
    }
}
