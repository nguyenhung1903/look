//
//  look_appApp.swift
//  look-app
//
//  Created by kunkka07xx on 2026/04/04.
//

import Darwin
import SwiftUI

@main
struct look_appApp: App {
    private static let author = "kunkka07xx"

    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var appUIState = AppUIState()
    @StateObject private var themeStore = ThemeStore()
    private let hotKeyManager = GlobalHotKeyManager()

    init() {
        if let exitCode = handleCLIFlags() {
            fflush(stdout)
            exit(exitCode)
        }

        hotKeyManager.registerToggleHotKey()
    }

    private func handleCLIFlags() -> Int32? {
        if CommandLine.arguments.contains("-h") || CommandLine.arguments.contains("--help") {
            print("look - lightweight macOS launcher")
            print("Author: \(Self.author)")
            print("")
            print("Usage:")
            print("  look                Launch app UI")
            print("  look -v, --version  Print version")
            print("  look -h, --help     Print this help")
            return 0
        }

        if CommandLine.arguments.contains("-v") || CommandLine.arguments.contains("--version") {
            let version = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
            let build = Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String
            if let version {
                if let build, build != version {
                    print("look \(version) (\(build))")
                } else {
                    print("look \(version)")
                }
            } else {
                print("look unknown")
            }
            return 0
        }

        return nil
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .frame(minWidth: 620, minHeight: 600)
                .background(WindowConfigurator())
                .environmentObject(appUIState)
                .environmentObject(themeStore)
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(replacing: .appTermination) {
                Button("Hide Look") {
                    NotificationCenter.default.post(name: .lookHideLauncherRequested, object: nil)
                }
                .keyboardShortcut("q", modifiers: [.command])

                Button("Quit Look") {
                    NSApplication.shared.terminate(nil)
                }
                .keyboardShortcut("q", modifiers: [.command, .option])
            }

            CommandGroup(after: .appSettings) {
                Button("Theme Settings") {
                    appUIState.showsThemeSettings.toggle()
                }
                .keyboardShortcut(",", modifiers: [.command, .shift])

                Button("Reload Config") {
                    NotificationCenter.default.post(name: .lookReloadConfigRequested, object: nil)
                    NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
                }
                .keyboardShortcut(";", modifiers: [.command, .shift])

                Divider()

                Button("Zoom In") {
                    themeStore.zoomIn()
                    NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
                }
                .keyboardShortcut("=", modifiers: [.command])

                Button("Zoom Out") {
                    themeStore.zoomOut()
                    NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
                }
                .keyboardShortcut("-", modifiers: [.command])

                Button("Actual Size") {
                    themeStore.resetZoom()
                    NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
                }
                .keyboardShortcut("0", modifiers: [.command])
            }
        }
    }
}
