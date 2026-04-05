//
//  look_appApp.swift
//  look-app
//
//  Created by kunkka07xx on 2026/04/04.
//

import SwiftUI

@main
struct look_appApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var appUIState = AppUIState()
    @StateObject private var themeStore = ThemeStore()
    private let hotKeyManager = GlobalHotKeyManager()

    init() {
        hotKeyManager.registerToggleHotKey()
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
