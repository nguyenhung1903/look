import Foundation
import CoreGraphics

struct AppCommand: Identifiable {
    let id: String
    let title: String
    let detail: String
    let placeholder: String
}

enum AppConstants {
    enum Launcher {
        static let defaultSearchLimit = 40
        static let commandListMaxHeight: CGFloat = 180
        static let commandResultFontSize: CGFloat = 18
        static let calcMaxMagnitude = 1_000_000_000_000.0

        static let commandCatalog: [AppCommand] = [
            AppCommand(id: "shell", title: "shell", detail: "Run a shell command", placeholder: "Type shell command"),
            AppCommand(id: "calc", title: "calc", detail: "Evaluate math expression", placeholder: "Type math expression"),
            AppCommand(id: "kill", title: "kill", detail: "Force kill a running app", placeholder: "Type app name to kill"),
        ]

        static let normalHint = "Tab/Shift+Tab move  •  Enter open  •  t\"text translate  •  Cmd+Enter web  •  / command mode"
        static let commandHint = "Tab/Shift+Tab select command  •  Enter run  •  Shift+Esc exit"
        static let commandEmptyMessage = "Type expression and press Enter"
    }

    enum ThemeUI {
        static let labelWidth: CGFloat = 110
        static let pickerWidth: CGFloat = 140
    }
}
