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
            guard isExpressionReadyForEvaluation(expr) else { return nil }

            switch evaluateMathExpression(expr) {
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
        let trimmed = commandInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return false }
        let pattern = "(^|\\s)sudo(\\s|$)"
        guard let regex = try? NSRegularExpression(pattern: pattern, options: [.caseInsensitive]) else {
            return false
        }
        let range = NSRange(trimmed.startIndex..<trimmed.endIndex, in: trimmed)
        return regex.firstMatch(in: trimmed, range: range) != nil
    }

    private var filteredCommands: [AppCommand] {
        let prefix = commandNamePart.lowercased()
        if prefix.isEmpty {
            return commandCatalog
        }
        return commandCatalog.filter { $0.id.hasPrefix(prefix) }
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
            runCommandModeAction()
        } else {
            openSelectedApp()
        }

        DispatchQueue.main.async {
            isQueryFocused = true
        }
    }

    private func runCommandModeAction() {
        let resolvedCommand = commandCatalog.first(where: { $0.id == (activeCommandID ?? "") })
            ?? commandCatalog.first(where: { $0.id == commandNamePart.lowercased() })
            ?? commandCatalog.first(where: { $0.id == selectedCommandID })

        guard let resolvedCommand else {
            commandFeedback = "Unknown command. Try /shell or /calc"
            return
        }

        switch resolvedCommand.id {
        case "shell":
            guard !commandArgsPart.isEmpty else {
                commandFeedback = "Usage: /shell <command>"
                return
            }
            let shellCommand = commandArgsPart
            commandFeedback = "Running..."
            Task.detached(priority: .userInitiated) {
                let process = Process()
                let outputPipe = Pipe()
                process.executableURL = URL(fileURLWithPath: "/bin/zsh")
                process.arguments = ["-lc", shellCommand]
                process.standardOutput = outputPipe
                process.standardError = outputPipe

                let message: String
                do {
                    try process.run()
                    process.waitUntilExit()
                    let data = outputPipe.fileHandleForReading.readDataToEndOfFile()
                    let raw = String(data: data, encoding: .utf8)?
                        .trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
                    if raw.isEmpty {
                        message = process.terminationStatus == 0 ? "Done" : "Error: command failed"
                    } else {
                        let prefix = process.terminationStatus == 0 ? "" : "Error: "
                        message = String((prefix + raw).prefix(180))
                    }
                } catch {
                    message = "Error: failed to execute command"
                }

                await MainActor.run {
                    commandFeedback = message
                    isQueryFocused = true
                }
            }
        case "calc":
            guard !commandArgsPart.isEmpty else {
                commandFeedback = "Usage: /calc <expression>"
                return
            }
            let calcResult = evaluateMathExpression(commandArgsPart)
            switch calcResult {
            case .value(let result):
                commandFeedback = "Result: \(result)"
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(result, forType: .string)
            case .error(let message):
                commandFeedback = message
            }
        default:
            commandFeedback = "Unsupported command"
        }
    }

    private enum CalcResult {
        case value(String)
        case error(String)
    }

    private func evaluateMathExpression(_ expression: String) -> CalcResult {
        guard isExpressionReadyForEvaluation(expression) else {
            return .error("Invalid expression")
        }

        let normalized = decimalizeIntegerTokens(in: normalizeMathExpression(expression))

        if containsDivisionByZero(in: normalized) {
            return .error("Error: division by zero")
        }

        let parsed = NSExpression(format: normalized)
        guard let value = parsed.expressionValue(with: nil, context: nil) else {
            return .error("Invalid expression")
        }
        if let number = value as? NSNumber {
            let evaluated = number.doubleValue
            if abs(evaluated) > AppConstants.Launcher.calcMaxMagnitude {
                return .error("Error: result out of range (±1,000,000,000,000)")
            }
            return .value(formatFloat(evaluated))
        }
        return .error("Invalid expression")
    }

    private func isExpressionReadyForEvaluation(_ expression: String) -> Bool {
        let trimmed = expression.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            return false
        }

        var balance = 0
        for ch in trimmed {
            if ch == "(" { balance += 1 }
            if ch == ")" {
                balance -= 1
                if balance < 0 { return false }
            }
        }
        if balance != 0 {
            return false
        }

        if let last = trimmed.last,
            "+-*/.(".contains(last)
        {
            return false
        }

        let allowedPattern = "^[0-9A-Za-z_+\\-*/().:xXvV\\s]+$"
        guard let regex = try? NSRegularExpression(pattern: allowedPattern) else { return false }
        let full = NSRange(trimmed.startIndex..<trimmed.endIndex, in: trimmed)
        guard let match = regex.firstMatch(in: trimmed, range: full), match.range == full else {
            return false
        }

        return true
    }

    private func formatFloat(_ value: Double) -> String {
        if value.isNaN || value.isInfinite {
            return "nan"
        }

        let formatter = NumberFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.numberStyle = .decimal
        formatter.groupingSeparator = ","
        formatter.usesGroupingSeparator = true
        formatter.minimumFractionDigits = 4
        formatter.maximumFractionDigits = 4
        formatter.minimumIntegerDigits = 1
        return formatter.string(from: NSNumber(value: value)) ?? String(format: "%.4f", value)
    }

    private func decimalizeIntegerTokens(in expression: String) -> String {
        let pattern = "(?<![A-Za-z0-9_\\.])(\\d+)(?![A-Za-z0-9_\\.])"
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return expression }

        let range = NSRange(expression.startIndex..<expression.endIndex, in: expression)
        let matches = regex.matches(in: expression, range: range)
        var output = expression
        for match in matches.reversed() {
            guard let tokenRange = Range(match.range(at: 1), in: output) else { continue }
            output.replaceSubrange(tokenRange, with: output[tokenRange] + ".0")
        }
        return output
    }

    private func containsDivisionByZero(in expression: String) -> Bool {
        let pattern = "/\\s*0+(?:\\.0+)?(?:\\b|(?=\\)))"
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return false }
        let range = NSRange(expression.startIndex..<expression.endIndex, in: expression)
        return regex.firstMatch(in: expression, range: range) != nil
    }

    private func normalizeMathExpression(_ expression: String) -> String {
        var normalized = expression
            .replacingOccurrences(of: "x", with: "*")
            .replacingOccurrences(of: "X", with: "*")
            .replacingOccurrences(of: ":", with: "/")

        normalized = replacePrefixSqrt(in: normalized)
        return normalized
    }

    private func replacePrefixSqrt(in expression: String) -> String {
        var output = ""
        var index = expression.startIndex

        while index < expression.endIndex {
            let char = expression[index]
            if char == "v" || char == "V" {
                let prev = index > expression.startIndex ? expression[expression.index(before: index)] : " "
                let nextIndex = expression.index(after: index)
                let next = nextIndex < expression.endIndex ? expression[nextIndex] : " "
                let prevIsWord = prev.isLetter || prev.isNumber || prev == "_"
                let nextIsStart = next.isNumber || next == "." || next == "(" || next == " "
                if !prevIsWord && nextIsStart {
                    output.append("sqrt")
                    index = nextIndex
                    continue
                }
            }

            output.append(char)
            index = expression.index(after: index)
        }

        return output
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
                    HStack(spacing: 8) {
                        Image(systemName: isCommandMode ? "terminal" : "magnifyingglass")
                            .foregroundStyle(isCommandMode ? .green : .secondary)
                        TextField(
                            isCommandMode
                                    ? (activeCommand?.placeholder ?? "Choose a command with Tab")
                                    : "Search apps",
                            text: isCommandMode ? $commandInput : $query
                        )
                            .textFieldStyle(.plain)
                            .focused($isQueryFocused)
                            .onTapGesture {
                                DispatchQueue.main.async {
                                    isQueryFocused = true
                                }
                            }
                            .onSubmit(handleSubmit)

                        if isCommandMode {
                            if let activeCommand {
                                Text("/\(activeCommand.title)")
                                    .font(
                                        themeStore.uiFont(
                                            size: CGFloat(themeStore.settings.fontSize - 1),
                                            weight: .regular
                                        )
                                    )
                                    .foregroundStyle(themeStore.fontColor())
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 3)
                                    .background(.green.opacity(0.18), in: Capsule())
                            }
                            Button("Exit") {
                                exitCommandMode()
                            }
                            .keyboardShortcut(.escape, modifiers: [.shift])
                            .font(
                                themeStore.uiFont(
                                    size: CGFloat(themeStore.settings.fontSize - 1),
                                    weight: .regular
                                )
                            )
                            .buttonStyle(.plain)
                            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 10)
                    .background(
                        .white.opacity(0.14),
                        in: RoundedRectangle(cornerRadius: 10, style: .continuous)
                    )

                    if let bannerMessage {
                        Text(bannerMessage)
                            .font(
                                themeStore.uiFont(
                                    size: CGFloat(themeStore.settings.fontSize - 1),
                                    weight: .semibold
                                )
                            )
                            .foregroundStyle(themeStore.fontColor())
                            .padding(.horizontal, 10)
                            .padding(.vertical, 6)
                            .background(.green.opacity(0.42), in: Capsule())
                            .transition(.move(edge: .top).combined(with: .opacity))
                    }

                    if isCommandMode {
                        Text(
                            liveCommandPreview
                                ?? (commandFeedback.isEmpty
                                    ? AppConstants.Launcher.commandEmptyMessage
                                    : commandFeedback)
                        )
                            .font(
                                themeStore.uiFont(
                                    size: CGFloat(themeStore.settings.fontSize + 4),
                                    weight: .semibold
                                )
                            )
                            .foregroundStyle(themeStore.fontColor())
                            .lineLimit(1)
                    }

                    if isCommandMode {
                        Spacer(minLength: 8)

                        ScrollView {
                            LazyVStack(spacing: 4) {
                                ForEach(filteredCommands) { command in
                                    HStack(spacing: 10) {
                                        Image(systemName: "terminal")
                                            .frame(width: 22, height: 22)
                                            .foregroundStyle(.green)
                                        VStack(alignment: .leading, spacing: 2) {
                                            Text("/\(command.title)")
                                                .font(
                                                    themeStore.uiFont(
                                                        size: CGFloat(themeStore.settings.fontSize),
                                                        weight: .semibold
                                                    )
                                                )
                                            Text(command.detail)
                                                .font(
                                                    themeStore.uiFont(
                                                        size: CGFloat(themeStore.settings.fontSize - 1),
                                                        weight: .regular
                                                    )
                                                )
                                                .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
                                        }
                                        Spacer(minLength: 0)
                                    }
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 8)
                                    .background(
                                        (selectedCommandID == command.id || activeCommandID == command.id)
                                            ? .green.opacity(0.20) : .white.opacity(0.08),
                                        in: RoundedRectangle(cornerRadius: 8, style: .continuous)
                                    )
                                    .onTapGesture {
                                        selectCommand(command.id)
                                    }
                                }
                            }
                            .padding(2)
                        }
                        .frame(maxHeight: AppConstants.Launcher.commandListMaxHeight)
                    } else {
                        ScrollViewReader { proxy in
                            ScrollView {
                                LazyVStack(spacing: 4) {
                                    ForEach(filteredResults) { result in
                                        LauncherRowView(
                                            result: result,
                                            isSelected: selectedResultID == result.id,
                                            onOpen: {
                                                selectedResultID = result.id
                                                openSelectedApp()
                                            }
                                        )
                                        .id(result.id)
                                    }
                                }
                                .padding(2)
                            }
                            .onChange(of: selectedResultID) { _, newID in
                                guard let newID else { return }
                                withAnimation(.easeOut(duration: 0.12)) {
                                    proxy.scrollTo(newID, anchor: .center)
                                }
                            }
                        }
                    }

                    Text(
                        isCommandMode
                            ? AppConstants.Launcher.commandHint
                            : AppConstants.Launcher.normalHint
                    )
                        .font(
                            themeStore.uiFont(
                                size: CGFloat(themeStore.settings.fontSize - 1),
                                weight: .regular
                            )
                        )
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
                }
            }
            .padding(14)
            .font(themeStore.uiFont())
            .foregroundStyle(themeStore.fontColor())
            .background(
                .black.opacity(0.16),
                in: RoundedRectangle(cornerRadius: 14, style: .continuous)
            )
        }
        .background(Color.clear)
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(
                    hasSudoWarning ? Color.orange.opacity(0.95) : themeStore.borderColor(),
                    lineWidth: themeStore.borderLineWidth()
                )
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
            }
        )
    }
}
