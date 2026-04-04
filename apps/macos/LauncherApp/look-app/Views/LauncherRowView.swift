import AppKit
import SwiftUI

struct LauncherRowView: View {
    @EnvironmentObject private var themeStore: ThemeStore

    let result: LauncherResult
    let isSelected: Bool
    let onOpen: () -> Void

    private var rowIcon: NSImage {
        if result.id.hasPrefix("setting:") {
            let settingsPath = "/System/Applications/System Settings.app"
            if FileManager.default.fileExists(atPath: settingsPath) {
                return NSWorkspace.shared.icon(forFile: settingsPath)
            }
            let legacyPath = "/System/Applications/System Preferences.app"
            return NSWorkspace.shared.icon(forFile: legacyPath)
        }
        return NSWorkspace.shared.icon(forFile: result.path)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Image(nsImage: rowIcon)
                    .resizable()
                    .frame(width: 22, height: 22)
                Text(result.title)
                    .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .medium))
                    .foregroundStyle(themeStore.fontColor())
                Spacer(minLength: 0)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(
                isSelected ? .white.opacity(0.12) : .clear,
                in: RoundedRectangle(cornerRadius: 8, style: .continuous)
            )
            .overlay {
                if isSelected {
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(.white.opacity(0.18), lineWidth: 1)
                }
            }
            .contentShape(Rectangle())
            .onTapGesture {
                onOpen()
            }

            Rectangle()
                .fill(.white.opacity(0.06))
                .frame(height: 1)
                .padding(.horizontal, 6)
        }
    }
}
