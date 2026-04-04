import Foundation
import Combine

final class AppUIState: ObservableObject {
    @Published var showsThemeSettings = false
}

extension Notification.Name {
    static let lookReloadConfigRequested = Notification.Name("look.reloadConfigRequested")
    static let lookRefocusInputRequested = Notification.Name("look.refocusInputRequested")
    static let lookFocusSettingsInputRequested = Notification.Name("look.focusSettingsInputRequested")
}
