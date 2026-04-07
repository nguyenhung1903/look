import Foundation

enum HintText {
    enum Launcher {
        static let normal = "Enter open  •  Cmd+F reveal  •  Cmd+H help  •  Cmd+/ command mode"
        static let command = "Tab select  •  Cmd+1/2/3  •  Enter run  •  Esc back  •  Cmd+H help"
        static let kill = "Up/Down navigate  •  Cmd+1/2/3 switch  •  Y confirm  •  N cancel  •  Cmd+Esc back to list"
        static let sys = "Sys info view  •  Cmd+Esc back to command list (calc)  •  Esc back"
    }

    enum Settings {
        static let advancedApply = "Save Config, then press Cmd+Shift+; to apply backend indexing changes."
        static let shortcutsTips = "Tips: t\"word for network translate | tw\"word to translate EN↔VI/EN↔JA | /kill to force quit apps"
    }
}
