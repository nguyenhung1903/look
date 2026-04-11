import AppKit
import SwiftUI

struct KillCommand {
    static func getRunningApps() -> [NSRunningApplication] {
        NSWorkspace.shared.runningApplications
            .filter { $0.activationPolicy == .regular }
            .sorted { ($0.localizedName ?? "") < ($1.localizedName ?? "") }
    }

    static func filterApps(searchTerm: String, from apps: [NSRunningApplication]) -> [(NSRunningApplication, Int)] {
        if searchTerm.isEmpty {
            return apps.enumerated().map { ($0.element, $0.offset + 1) }
        } else if let num = Int(searchTerm), num > 0 && num <= apps.count {
            return [(apps[num - 1], num)]
        } else {
            return apps.enumerated()
                .filter { ($0.element.localizedName ?? "").lowercased().contains(searchTerm.lowercased()) }
                .map { ($0.element, $0.offset + 1) }
        }
    }

    static func kill(pid: Int32, name: String, completion: @escaping (String) -> Void) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/kill")
        process.arguments = ["-9", "\(pid)"]

        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus == 0 {
                completion("Killed: \(name) (PID: \(pid))")
            } else {
                completion("Failed to kill \(name): permission denied")
            }
        } catch {
            completion("Error: \(error.localizedDescription)")
        }
    }
}

struct KillCommandView: View {
    let suggestions: [(NSRunningApplication, Int)]
    let selectedIndex: Int?
    let pendingApp: (NSRunningApplication, Int)?
    let themeStore: ThemeStore

    let onSelect: (NSRunningApplication, Int) -> Void
    let onConfirm: (Int) -> Void
    let onCancel: () -> Void

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 2) {
                ForEach(suggestions.prefix(20), id: \.1) { app, num in
                    Button {
                        onSelect(app, num)
                    } label: {
                        HStack(spacing: 10) {
                            Image(nsImage: app.icon ?? NSImage())
                                .resizable()
                                .frame(width: 20, height: 20)
                            Text(app.localizedName ?? "Unknown")
                                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .medium))
                                .foregroundStyle(themeStore.fontColor())
                            Spacer()
                            Text("PID: \(app.processIdentifier)")
                                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 2), weight: .regular))
                                .foregroundStyle(themeStore.mutedTextColor())
                            if selectedIndex == num {
                                Text("→ Enter")
                                    .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 2), weight: .regular))
                                    .foregroundStyle(themeStore.accentColor())
                            }
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background(
                            selectedIndex == num
                                ? themeStore.selectionFillColor() : .clear,
                            in: RoundedRectangle(cornerRadius: 6, style: .continuous)
                        )
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(2)
        }

        if let (app, _) = pendingApp {
            Spacer(minLength: 8)
            HStack(spacing: 12) {
                Image(nsImage: app.icon ?? NSImage())
                    .resizable()
                    .frame(width: 24, height: 24)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Kill \(app.localizedName ?? "Unknown")?")
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .semibold))
                        .foregroundStyle(themeStore.fontColor())
                    Text("PID: \(app.processIdentifier)")
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 2), weight: .regular))
                        .foregroundStyle(themeStore.mutedTextColor())
                }
                Spacer()
                Button {
                    onConfirm(Int(app.processIdentifier))
                } label: {
                    Text("Y / Yes")
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .medium))
                        .foregroundStyle(themeStore.onDangerColor())
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(themeStore.dangerColor(), in: Capsule())
                }
                .buttonStyle(.plain)
                Button {
                    onCancel()
                } label: {
                    Text("N / No")
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .medium))
                        .foregroundStyle(themeStore.fontColor())
                        .padding(.horizontal, 12)
                        .padding(.vertical, 6)
                        .background(themeStore.controlFillColor(), in: Capsule())
                }
                .buttonStyle(.plain)
            }
            .padding(10)
            .background(themeStore.controlFillColor().opacity(0.8), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
        }
    }
}
