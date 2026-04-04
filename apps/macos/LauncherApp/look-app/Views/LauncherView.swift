import AppKit
import SwiftUI

struct LauncherView: View {
    @EnvironmentObject private var appUIState: AppUIState
    @EnvironmentObject private var themeStore: ThemeStore

    @State private var query = ""
    @State private var commandInput = ""
    @State private var isCommandMode = false
    @State private var backendResults: [LauncherResult] = []
    @State private var selectedResultID: String?
    @State private var selectedCommandID: String?
    @State private var activeCommandID: String?
    @State private var commandFeedback = ""
    @State private var keyboardMonitor = KeyboardSelectionMonitor()
    @State private var searchTask: Task<Void, Never>?
    @State private var bannerMessage: String?
    @State private var bannerTask: Task<Void, Never>?
    @State private var selectedKillSuggestionIndex: Int?
    @State private var pendingKillApp: (NSRunningApplication, Int)?
    @FocusState private var isQueryFocused: Bool

    private let bridge = EngineBridge.shared

    private let commandCatalog: [AppCommand] = AppConstants.Launcher.commandCatalog

    private var filteredResults: [LauncherResult] {
        var seenTitles = Set<String>()
        var unique: [LauncherResult] = []
        for item in backendResults {
            let key = item.title.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            if key.isEmpty {
                unique.append(item)
                continue
            }
            if seenTitles.insert(key).inserted {
                unique.append(item)
            }
        }
        return unique
    }

    private var commandNamePart: String {
        guard activeCommandID == nil else { return "" }
        let normalized = commandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalized.isEmpty else { return "" }
        return normalized.split(maxSplits: 1, whereSeparator: { $0.isWhitespace }).first.map(String.init) ?? ""
    }

    private var commandArgsPart: String {
        if activeCommandID != nil {
            return commandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        }

        let normalized = commandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let splitPoint = normalized.firstIndex(where: { $0.isWhitespace }) else { return "" }
        return String(normalized[splitPoint...]).trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var activeCommand: AppCommand? {
        guard let activeCommandID else { return nil }
        return commandCatalog.first(where: { $0.id == activeCommandID })
    }

    private var liveCommandPreview: String? {
        guard isCommandMode else { return nil }

        if hasSudoWarning {
            return "Warning: sudo command detected"
        }

        if activeCommandID == "calc" {
            let expr = commandInput.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !expr.isEmpty else { return nil }
            guard CalcCommand.isReadyForEvaluation(expr) else { return nil }

            switch CalcCommand.evaluate(expr) {
            case .value(let value):
                return "Result: \(value)"
            case .error(let message):
                return message
            }
        }

        return nil
    }

    private var hasSudoWarning: Bool {
        guard isCommandMode, activeCommandID == "shell" else { return false }
        return ShellCommand.hasSudoWarning(commandInput)
    }

    private var filteredCommands: [AppCommand] {
        let prefix = commandNamePart.lowercased()
        if prefix.isEmpty {
            return commandCatalog
        }
        return commandCatalog.filter { $0.id.hasPrefix(prefix) }
    }

    private var killSuggestions: [(NSRunningApplication, Int)] {
        let apps = KillCommand.getRunningApps()
        let searchTerm = commandArgsPart.trimmingCharacters(in: .whitespacesAndNewlines)
        return KillCommand.filterApps(searchTerm: searchTerm, from: apps)
    }

    private func setInitialSelection() {
        if isCommandMode {
            if let activeCommandID {
                selectedCommandID = activeCommandID
            } else {
                selectedCommandID = filteredCommands.first?.id
            }
        } else {
            selectedResultID = filteredResults.first?.id
        }
    }

    private func moveSelection(_ direction: MoveCommandDirection, shouldAutocompleteCommand: Bool = false) {
        guard !appUIState.showsThemeSettings else { return }

        if isCommandMode && activeCommandID == "kill" {
            let suggestions = killSuggestions.prefix(20)
            guard !suggestions.isEmpty else { return }

            let currentNum = selectedKillSuggestionIndex
            let currentIndex = suggestions.firstIndex { $0.1 == currentNum }

            let nextIndex: Int
            switch direction {
            case .down:
                if let currentIndex {
                    nextIndex = min(currentIndex + 1, suggestions.count - 1)
                } else {
                    nextIndex = 0
                }
            case .up:
                if let currentIndex {
                    nextIndex = max(currentIndex - 1, 0)
                } else {
                    nextIndex = suggestions.count - 1
                }
            default:
                return
            }

            selectedKillSuggestionIndex = suggestions[nextIndex].1
            return
        }

        if isCommandMode {
            guard !filteredCommands.isEmpty else {
                selectedCommandID = nil
                return
            }

            guard let currentID = selectedCommandID,
                let currentIndex = filteredCommands.firstIndex(where: { $0.id == currentID })
            else {
                selectedCommandID = filteredCommands.first?.id
                if shouldAutocompleteCommand {
                    autocompleteSelectedCommand()
                }
                return
            }

            let nextIndex: Int
            switch direction {
            case .down:
                nextIndex = (currentIndex + 1) % filteredCommands.count
            case .up:
                nextIndex = (currentIndex - 1 + filteredCommands.count) % filteredCommands.count
            default:
                return
            }

            selectedCommandID = filteredCommands[nextIndex].id
            if shouldAutocompleteCommand {
                autocompleteSelectedCommand()
            }
            return
        }

        guard !filteredResults.isEmpty else {
            selectedResultID = nil
            return
        }

        guard let currentID = selectedResultID,
            let currentIndex = filteredResults.firstIndex(where: { $0.id == currentID })
        else {
            selectedResultID = filteredResults.first?.id
            return
        }

        let nextIndex: Int
        switch direction {
        case .down:
            nextIndex = (currentIndex + 1) % filteredResults.count
        case .up:
            nextIndex = (currentIndex - 1 + filteredResults.count) % filteredResults.count
        default:
            return
        }

        selectedResultID = filteredResults[nextIndex].id
    }

    private func autocompleteSelectedCommand() {
        guard isCommandMode,
            let commandID = selectedCommandID,
            filteredCommands.contains(where: { $0.id == commandID })
        else { return }

        activeCommandID = commandID
        commandFeedback = "Selected /\(commandID)"
    }

    private func enterCommandMode() {
        isCommandMode = true
        commandInput = ""
        commandFeedback = ""
        activeCommandID = "calc"
        selectedCommandID = "calc"
        DispatchQueue.main.async {
            isQueryFocused = true
        }
    }

    private func exitCommandMode() {
        guard isCommandMode else { return }
        isCommandMode = false
        commandInput = ""
        commandFeedback = ""
        activeCommandID = nil
        selectedCommandID = nil
        refreshSearchResults()
        DispatchQueue.main.async {
            isQueryFocused = true
        }
    }

    private func handleSubmit() {
        if isCommandMode {
            if activeCommandID == "kill", let selectedNum = selectedKillSuggestionIndex {
                if let (app, _) = killSuggestions.first(where: { $0.1 == selectedNum }) {
                    pendingKillApp = (app, selectedNum)
                }
            } else {
                runCommandModeAction()
            }
        } else {
            let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
            if let translationTarget = extractTranslationQuery(from: trimmed) {
                let result = bridge.translate(text: translationTarget)
                if let translated = result?.translated, !translated.isEmpty {
                    showBanner("\(translationTarget) -> \(translated)")
                } else {
                    showBanner("Translation failed")
                }
                isQueryFocused = true
            } else {
                openSelectedApp()
            }
        }

        DispatchQueue.main.async {
            isQueryFocused = true
        }
    }

    private func extractTranslationQuery(from input: String) -> String? {
        if input.hasPrefix("t\"") {
            let text = String(input.dropFirst(2)).trimmingCharacters(in: .whitespacesAndNewlines)
            return text.isEmpty ? nil : text
        }

        if input.lowercased().hasPrefix("tr ") {
            let text = String(input.dropFirst(3)).trimmingCharacters(in: .whitespacesAndNewlines)
            return text.isEmpty ? nil : text
        }

        return nil
    }

    private func runCommandModeAction() {
        let resolvedCommand = commandCatalog.first(where: { $0.id == (activeCommandID ?? "") })
            ?? commandCatalog.first(where: { $0.id == commandNamePart.lowercased() })
            ?? commandCatalog.first(where: { $0.id == selectedCommandID })

        guard let resolvedCommand else {
            commandFeedback = "Unknown command. Try /shell, /calc, or /kill"
            return
        }

        switch resolvedCommand.id {
        case "shell":
            guard !commandArgsPart.isEmpty else {
                commandFeedback = "Usage: /shell <command>"
                return
            }
            commandFeedback = "Running..."
            ShellCommand.run(commandArgsPart) { [self] message in
                commandFeedback = message
                isQueryFocused = true
            }
        case "calc":
            guard !commandArgsPart.isEmpty else {
                commandFeedback = "Usage: /calc <expression>"
                return
            }
            let result = CalcCommand.evaluate(commandArgsPart)
            switch result {
            case .value(let value):
                commandFeedback = "Result: \(value)"
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(value, forType: .string)
            case .error(let message):
                commandFeedback = message
            }
        case "kill":
            let apps = KillCommand.getRunningApps()
            let searchTerm = commandArgsPart.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            let matched = KillCommand.filterApps(searchTerm: searchTerm, from: apps)

            if apps.isEmpty {
                commandFeedback = "No apps running"
            } else if searchTerm.isEmpty {
                let appList = apps.enumerated().map { idx, app in
                    "\(idx + 1). \(app.localizedName ?? "Unknown") (PID: \(app.processIdentifier))"
                }
                commandFeedback = "Running apps:\n" + appList.joined(separator: "\n") + "\n\n/kill <name or number>"
            } else if matched.isEmpty {
                commandFeedback = "No matching apps. /kill to list all."
            } else if matched.count > 1 {
                let list = matched.map { app, num in "\(num). \(app.localizedName ?? "Unknown")" }
                commandFeedback = "Multiple matches:\n" + list.joined(separator: "\n") + "\n\nBe more specific."
            } else {
                let (app, _) = matched[0]
                KillCommand.kill(pid: app.processIdentifier, name: app.localizedName ?? "Unknown") { [self] message in
                    commandFeedback = message
                }
            }
        default:
            commandFeedback = "Unsupported command"
        }
    }

    private func runKillCommand(num: Int) {
        let apps = KillCommand.getRunningApps()
        guard num > 0 && num <= apps.count else {
            commandFeedback = "Invalid app number"
            return
        }
        let app = apps[num - 1]
        KillCommand.kill(pid: app.processIdentifier, name: app.localizedName ?? "Unknown") { [self] message in
            commandFeedback = message
        }
    }

    private func openSelectedApp() {
        guard let selectedResultID,
            let selected = filteredResults.first(where: { $0.id == selectedResultID })
        else { return }

        switch selected.kind {
        case .app:
            openTarget(selected.path)
            bridge.recordUsage(candidateID: selected.id, action: "open_app")
        case .file:
            openTarget(selected.path)
            bridge.recordUsage(candidateID: selected.id, action: "open_file")
        case .folder:
            openTarget(selected.path)
            bridge.recordUsage(candidateID: selected.id, action: "open_folder")
        }
    }

    private func openTarget(_ target: String) {
        if target.contains(":") && !target.hasPrefix("/") {
            if let url = URL(string: target) {
                NSWorkspace.shared.open(url)
                return
            }
        }

        NSWorkspace.shared.open(URL(fileURLWithPath: target))
    }

    private func refreshSearchResults() {
        guard !isCommandMode else { return }

        let currentQuery = query
        let searchLimit = AppConstants.Launcher.defaultSearchLimit
        searchTask?.cancel()
        searchTask = Task {
            try? await Task.sleep(nanoseconds: 70_000_000)
            guard !Task.isCancelled else { return }

            let results = await Task.detached(priority: .userInitiated) {
                bridge.search(query: currentQuery, limit: searchLimit)
            }.value

            await MainActor.run {
                guard !isCommandMode, query == currentQuery else { return }
                backendResults = results
                setInitialSelection()
            }
        }
    }

    private func performWebSearchFromQuery() {
        guard !isCommandMode else { return }
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        if let translationTarget = extractTranslationQuery(from: trimmed) {
            let result = bridge.translate(text: translationTarget)
            if let translated = result?.translated, !translated.isEmpty {
                showBanner("\(translationTarget) -> \(translated)")
            } else {
                showBanner("Translation failed")
            }
            isQueryFocused = true
            return
        }

        var components = URLComponents(string: "https://www.google.com/search")
        components?.queryItems = [URLQueryItem(name: "q", value: trimmed)]
        guard let url = components?.url else { return }
        NSWorkspace.shared.open(url)
    }

    private func reloadConfig() {
        themeStore.reloadFromConfig()
        let backendReloaded = bridge.reloadConfig()
        showBanner(backendReloaded ? "Config reloaded" : "Config reload failed")
        if isCommandMode {
            commandFeedback = backendReloaded ? "Config reloaded" : "Config reload failed"
        }
        refreshSearchResults()
        focusActiveInput()
    }

    private func focusActiveInput() {
        if appUIState.showsThemeSettings {
            NotificationCenter.default.post(name: .lookFocusSettingsInputRequested, object: nil)
            return
        }

        DispatchQueue.main.async {
            isQueryFocused = true
        }
    }

    private func showBanner(_ message: String) {
        bannerTask?.cancel()
        withAnimation(.easeOut(duration: 0.15)) {
            bannerMessage = message
        }

        bannerTask = Task {
            try? await Task.sleep(nanoseconds: 1_800_000_000)
            guard !Task.isCancelled else { return }
            await MainActor.run {
                withAnimation(.easeIn(duration: 0.15)) {
                    bannerMessage = nil
                }
            }
        }
    }

    private func selectCommand(_ commandID: String) {
        activeCommandID = commandID
        selectedCommandID = commandID
        commandFeedback = "Selected /\(commandID)"
    }

    var body: some View {
        ZStack {
            themedBackground

            VStack(alignment: .leading, spacing: 12) {
                if appUIState.showsThemeSettings {
                    ThemeSettingsView(settings: $themeStore.settings)
                } else {
                    SearchInputBar(
                        text: isCommandMode ? $commandInput : $query,
                        isCommandMode: $isCommandMode,
                        isQueryFocused: $isQueryFocused,
                        activeCommand: activeCommand,
                        themeStore: themeStore,
                        onSubmit: handleSubmit,
                        onExitCommandMode: exitCommandMode
                    )

                    if let bannerMessage {
                        Text(bannerMessage)
                            .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .semibold))
                            .foregroundStyle(themeStore.fontColor())
                            .padding(.horizontal, 10)
                            .padding(.vertical, 6)
                            .background(.green.opacity(0.42), in: Capsule())
                            .transition(.move(edge: .top).combined(with: .opacity))
                    }

                    if isCommandMode {
                        CommandFeedbackView(
                            message: liveCommandPreview ?? (commandFeedback.isEmpty ? AppConstants.Launcher.commandEmptyMessage : commandFeedback),
                            themeStore: themeStore
                        )
                    }

                    if isCommandMode {
                        Spacer(minLength: 8)

                        if activeCommandID == "kill" {
                            KillCommandView(
                                suggestions: Array(killSuggestions),
                                selectedIndex: selectedKillSuggestionIndex,
                                pendingApp: pendingKillApp,
                                themeStore: themeStore,
                                onSelect: { app, num in
                                    pendingKillApp = (app, num)
                                    selectedKillSuggestionIndex = num
                                },
                                onConfirm: { pid in
                                    runKillCommand(num: pid)
                                    pendingKillApp = nil
                                },
                                onCancel: { pendingKillApp = nil }
                            )
                            .onAppear {
                                if selectedKillSuggestionIndex == nil {
                                    selectedKillSuggestionIndex = killSuggestions.first?.1
                                }
                            }
                        } else {
                            CommandListView(
                                commands: filteredCommands,
                                selectedID: selectedCommandID,
                                activeID: activeCommandID,
                                themeStore: themeStore,
                                onSelect: selectCommand
                            )
                        }
                    } else {
                        ResultsListView(
                            results: filteredResults,
                            selectedID: selectedResultID,
                            themeStore: themeStore,
                            onSelect: { selectedResultID = $0 },
                            onOpen: { _ in openSelectedApp() }
                        )
                    }

                    HintBar(isCommandMode: isCommandMode, themeStore: themeStore)
                }
            }
            .padding(14)
            .font(themeStore.uiFont())
            .foregroundStyle(themeStore.fontColor())
            .background(.black.opacity(0.16), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        }
        .background(Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(hasSudoWarning ? Color.orange.opacity(0.95) : themeStore.borderColor(), lineWidth: themeStore.borderLineWidth())
        )
        .ignoresSafeArea()
        .onAppear {
            refreshSearchResults()
            startKeyboardNavigationIfNeeded()
            DispatchQueue.main.async {
                isQueryFocused = true
            }
        }
        .onDisappear {
            searchTask?.cancel()
            bannerTask?.cancel()
            keyboardMonitor.stop()
        }
        .onChange(of: query) { _, _ in
            if !isCommandMode && query.hasPrefix("/") {
                query = ""
                enterCommandMode()
                return
            }

            if !isCommandMode {
                refreshSearchResults()
            }
        }
        .onChange(of: commandInput) { _, _ in
            if isCommandMode {
                setInitialSelection()
            }
        }
        .onChange(of: appUIState.showsThemeSettings) { _, showsSettings in
            if showsSettings {
                keyboardMonitor.stop()
                NotificationCenter.default.post(name: .lookFocusSettingsInputRequested, object: nil)
            } else {
                startKeyboardNavigationIfNeeded()
                DispatchQueue.main.async {
                    isQueryFocused = true
                }
            }
        }
        .onMoveCommand { direction in
            moveSelection(direction)
        }
        .onReceive(
            NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)
        ) { _ in
            DispatchQueue.main.async {
                isQueryFocused = true
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .lookReloadConfigRequested)) { _ in
            reloadConfig()
        }
        .onReceive(NotificationCenter.default.publisher(for: .lookRefocusInputRequested)) { _ in
            focusActiveInput()
        }
    }

    @ViewBuilder
    private var themedBackground: some View {
        if let url = themeStore.backgroundImageURL,
            let image = NSImage(contentsOf: url)
        {
            backgroundImageView(image: image)
                .blur(radius: themeStore.settings.backgroundImageBlur)
                .opacity(themeStore.settings.backgroundImageOpacity)
        }

        VisualEffectBlur(material: themeStore.settings.blurMaterial.material)
            .opacity(
                min(
                    1,
                    max(
                        0,
                        themeStore.settings.blurOpacity * themeStore.settings.blurMaterial.blurOpacityScale
                    )
                )
            )

        Color(
            .sRGB,
            red: themeStore.settings.tintRed,
            green: themeStore.settings.tintGreen,
            blue: themeStore.settings.tintBlue,
            opacity: min(
                1,
                max(
                    0,
                    themeStore.settings.tintOpacity * themeStore.settings.blurMaterial.tintOpacityScale
                )
            )
        )
    }

    @ViewBuilder
    private func backgroundImageView(image: NSImage) -> some View {
        GeometryReader { proxy in
            let size = proxy.size

            Group {
                switch themeStore.settings.backgroundImageMode {
                case .fit:
                    Image(nsImage: image)
                        .resizable()
                        .scaledToFit()
                        .frame(width: size.width, height: size.height)
                case .fill:
                    Image(nsImage: image)
                        .resizable()
                        .scaledToFill()
                        .frame(width: size.width, height: size.height)
                        .clipped()
                case .stretch:
                    Image(nsImage: image)
                        .resizable()
                        .frame(width: size.width, height: size.height)
                case .tile:
                    Rectangle()
                        .fill(ImagePaint(image: Image(nsImage: image), scale: 0.3))
                        .frame(width: size.width, height: size.height)
                }
            }
        }
        .ignoresSafeArea()
        .allowsHitTesting(false)
    }

    private func startKeyboardNavigationIfNeeded() {
        guard !appUIState.showsThemeSettings else { return }
        keyboardMonitor.start(
            onNext: { moveSelection(.down, shouldAutocompleteCommand: true) },
            onPrevious: { moveSelection(.up, shouldAutocompleteCommand: true) },
            onExitCommandMode: {
                exitCommandMode()
            },
            onWebSearch: {
                performWebSearchFromQuery()
            },
            onConfirmKill: { [self] in
                if let (_, num) = pendingKillApp {
                    runKillCommand(num: num)
                    pendingKillApp = nil
                }
            },
            onCancelKill: { [self] in
                pendingKillApp = nil
            },
            killConfirmationActive: { [self] in
                pendingKillApp != nil
            }
        )
    }
}
