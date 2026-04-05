import Foundation

enum HintText {
    enum Launcher {
        static let normal = "Tab/Shift+Tab move  •  Enter open  •  t\"text translate  •  Cmd+Enter web  •  Cmd+/ command mode"
        static let command = "Tab select  •  Cmd+1/2/3  •  Enter run  •  Esc hide"
        static let kill = "Up/Down navigate  •  Cmd+1/2/3 switch  •  Y confirm  •  N cancel  •  Cmd+Esc back to list"
        static let sys = "Sys info view  •  Cmd+Esc back to command list (calc)  •  Esc exit"
    }

    enum Settings {
        static let advancedApply = "Save Config, then press Cmd+Shift+; to apply backend indexing changes."
        static let shortcutsTips = "Tips: t\"word to translate | /kill to force quit apps | app_exclude_* to hide from search"
    }
}
