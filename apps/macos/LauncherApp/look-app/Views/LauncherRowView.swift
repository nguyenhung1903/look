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

    private var pathInfo: String {
        let parentPath = URL(fileURLWithPath: result.path).deletingLastPathComponent().path
        let components = parentPath
            .split(separator: "/")
            .map(String.init)
        let tail = components.suffix(3).joined(separator: "/")

        if tail.isEmpty {
            return "/"
        }
        if components.count > 3 {
            return ".../\(tail)"
        }
        return "/\(tail)"
    }

    private var kindLabel: String {
        switch result.kind {
        case .app:
            return "App"
        case .file:
            return "File"
        case .folder:
            return "Folder"
        }
    }

    private var metaLabel: String {
        if result.kind == .app {
            return kindLabel
        }
        return "\(kindLabel)  •  \(pathInfo)"
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 10) {
                Image(nsImage: rowIcon)
                    .resizable()
                    .frame(width: 22, height: 22)
                VStack(alignment: .leading, spacing: 2) {
                    Text(result.title)
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .medium))
                        .foregroundStyle(themeStore.fontColor())
                    Text(metaLabel)
                        .font(themeStore.uiFont(size: CGFloat(max(10, themeStore.settings.fontSize - 3)), weight: .regular))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.65))
                        .lineLimit(1)
                }
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
