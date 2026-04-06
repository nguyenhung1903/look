import AppKit
import SwiftUI
import UniformTypeIdentifiers

struct ThemeSettingsView: View {
    private enum Field {
        case fontName
    }

    @EnvironmentObject private var appUIState: AppUIState
    @EnvironmentObject private var themeStore: ThemeStore
    @Binding var settings: ThemeSettings
    @State private var selectedTab = 0
    @State private var saveMessage: String?
    @State private var fontSuggestions: [String] = []
    @State private var showsFontSuggestions = false
    @State private var isPickingFontSuggestion = false
    @FocusState private var focusedField: Field?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Settings")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize + 2), weight: .semibold))
                Spacer()

                if let saveMessage {
                    Text(saveMessage)
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                        .foregroundStyle(.white)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 5)
                        .background(.green.opacity(0.42), in: Capsule())
                }

                Button("Save Config") {
                    let ok = themeStore.saveCurrentConfigToFile()
                    saveMessage = ok ? "Saved" : "Save failed"
                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.6) {
                        saveMessage = nil
                    }
                    NotificationCenter.default.post(name: .lookFocusSettingsInputRequested, object: nil)
                }
                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))

                Button("Back to Launcher") {
                    appUIState.showsThemeSettings = false
                    NotificationCenter.default.post(name: .lookRefocusInputRequested, object: nil)
                }
                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                Text("Cmd+Shift+, to close")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 8) {
                tabButton(title: "Appearance", index: 0)
                tabButton(title: "Advanced", index: 1)
                tabButton(title: "Shortcuts", index: 2)
            }

            Group {
                if selectedTab == 0 {
                    appearanceTab
                } else if selectedTab == 1 {
                    backgroundTab
                } else {
                    shortcutsTab
                }
            }
            .frame(maxHeight: .infinity, alignment: .top)

        }
    }

    private var appearanceTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 10) {
                Text("Tint Color")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                    .foregroundStyle(.secondary)

                LabeledSlider(title: "Red", value: $settings.tintRed, range: 0...1)
                LabeledSlider(title: "Green", value: $settings.tintGreen, range: 0...1)
                LabeledSlider(title: "Blue", value: $settings.tintBlue, range: 0...1)
                LabeledSlider(title: "Tint Opacity", value: $settings.tintOpacity, range: 0...1)

                LabeledSlider(title: "Blur Opacity", value: $settings.blurOpacity, range: 0...1)

                HStack(spacing: 10) {
                    Text("Font Name")
                        .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                    TextField("SF Pro Text", text: $settings.fontName)
                        .textFieldStyle(.roundedBorder)
                        .focused($focusedField, equals: .fontName)
                        .onTapGesture {
                            focusedField = .fontName
                            showsFontSuggestions = true
                        }
                        .onChange(of: settings.fontName) { _, newValue in
                            if isPickingFontSuggestion {
                                return
                            }
                            fontSuggestions = themeStore.fontNameSuggestions(for: newValue, limit: 120)
                            showsFontSuggestions = focusedField == .fontName
                        }
                        .onSubmit {
                            if let first = fontSuggestions.first {
                                isPickingFontSuggestion = true
                                settings.fontName = first
                                DispatchQueue.main.async {
                                    placeCaretAtEndOfFontField()
                                    isPickingFontSuggestion = false
                                }
                            }
                            showsFontSuggestions = false
                        }
                        .frame(width: 220, alignment: .leading)

                    Text("Installed font name")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                        .lineLimit(1)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .overlay(alignment: .topLeading) {
                    if showsFontSuggestions && !fontSuggestions.isEmpty {
                        fontSuggestionsDropdown
                            .offset(x: AppConstants.ThemeUI.labelWidth + 10, y: 30)
                    }
                }
                .zIndex(showsFontSuggestions ? 100 : 1)

                LabeledSlider(title: "Font Size", value: $settings.fontSize, range: 10...28)

                Text("Font Color")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                    .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                LabeledSlider(title: "Text Red", value: $settings.fontRed, range: 0...1)
                LabeledSlider(title: "Text Green", value: $settings.fontGreen, range: 0...1)
                LabeledSlider(title: "Text Blue", value: $settings.fontBlue, range: 0...1)
                LabeledSlider(title: "Text Opacity", value: $settings.fontOpacity, range: 0...1)

                Text("Border")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                    .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                LabeledSlider(title: "Border Thick", value: $settings.borderThickness, range: 0...6)
                LabeledSlider(title: "Border Red", value: $settings.borderRed, range: 0...1)
                LabeledSlider(title: "Border Green", value: $settings.borderGreen, range: 0...1)
                LabeledSlider(title: "Border Blue", value: $settings.borderBlue, range: 0...1)
                LabeledSlider(title: "Border Opacity", value: $settings.borderOpacity, range: 0...1)

                HStack(spacing: 10) {
                    Text("Blur Style")
                        .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                        .foregroundStyle(.secondary)

                    Picker("Blur Style", selection: $settings.blurMaterial) {
                        ForEach(LauncherBlurMaterial.allCases) { item in
                            Text(item.title).tag(item)
                        }
                    }
                    .pickerStyle(.menu)
                    .labelsHidden()
                    .frame(width: AppConstants.ThemeUI.pickerWidth)

                    Text(settings.blurMaterial.detail)
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                        .lineLimit(1)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .onAppear {
                fontSuggestions = themeStore.fontNameSuggestions(for: settings.fontName, limit: 120)
                DispatchQueue.main.async {
                    focusedField = .fontName
                }
            }
            .onChange(of: focusedField) { _, focused in
                if focused != .fontName {
                    showsFontSuggestions = false
                }
            }
            .onReceive(NotificationCenter.default.publisher(for: .lookFocusSettingsInputRequested)) { _ in
                DispatchQueue.main.async {
                    focusedField = .fontName
                    showsFontSuggestions = false
                }
            }
        }
        .scrollIndicators(.hidden)
    }

    private func tabButton(title: String, index: Int) -> some View {
        let isActive = selectedTab == index
        return Button {
            selectedTab = index
            showsFontSuggestions = false
        } label: {
            Text(title)
                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .medium))
                .foregroundStyle(isActive ? themeStore.fontColor() : themeStore.fontColor(opacityMultiplier: 0.72))
                .frame(maxWidth: .infinity)
                .padding(.vertical, 7)
                .background(
                    (isActive ? .white.opacity(0.16) : .white.opacity(0.06)),
                    in: RoundedRectangle(cornerRadius: 8, style: .continuous)
                )
        }
        .buttonStyle(.plain)
    }

    private func placeCaretAtEndOfFontField() {
        guard let editor = NSApp.keyWindow?.firstResponder as? NSTextView else {
            return
        }
        let location = (editor.string as NSString).length
        editor.setSelectedRange(NSRange(location: location, length: 0))
    }

    private var fontSuggestionsDropdown: some View {
        ScrollView(.vertical) {
            VStack(alignment: .leading, spacing: 2) {
                ForEach(fontSuggestions, id: \.self) { suggestion in
                    Button {
                        isPickingFontSuggestion = true
                        settings.fontName = suggestion
                        fontSuggestions = themeStore.fontNameSuggestions(for: suggestion, limit: 120)
                        showsFontSuggestions = false
                        DispatchQueue.main.async {
                            focusedField = .fontName
                            placeCaretAtEndOfFontField()
                            isPickingFontSuggestion = false
                        }
                    } label: {
                        Text(suggestion)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 6)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(4)
        }
        .frame(width: 240, height: 320, alignment: .topLeading)
        .scrollIndicators(.hidden)
        .background(.black.opacity(0.72), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(.white.opacity(0.12), lineWidth: 1)
        )
        .shadow(color: .black.opacity(0.25), radius: 8, y: 4)
    }

    private var backgroundTab: some View {
        VStack(alignment: .leading, spacing: 10) {
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    Text("Background")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                    HStack {
                        Button("Choose Background Image") {
                            selectBackgroundImage()
                        }
                        if settings.backgroundImagePath != nil {
                            Button("Clear") {
                                themeStore.setBackgroundImage(url: nil)
                            }
                        }
                    }

                    Text(settings.backgroundImagePath ?? "No image selected")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
                        .lineLimit(1)

                    HStack(spacing: 10) {
                        Text("Image Layout")
                            .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            .foregroundStyle(.secondary)

                        Picker("Image Layout", selection: $settings.backgroundImageMode) {
                            ForEach(BackgroundImageMode.allCases) { mode in
                                Text(mode.title).tag(mode)
                            }
                        }
                        .pickerStyle(.menu)
                        .labelsHidden()
                        .frame(width: AppConstants.ThemeUI.pickerWidth)

                        Text(settings.backgroundImageMode.detail)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                            .lineLimit(1)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    LabeledSlider(title: "Image Opacity", value: $settings.backgroundImageOpacity, range: 0...1)
                    LabeledSlider(title: "Image Blur", value: $settings.backgroundImageBlur, range: 0...30)

                    Divider()
                        .overlay(.white.opacity(0.1))
                        .padding(.vertical, 4)

                    Text("Indexing")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                    HStack(spacing: 10) {
                        Text("File Scan Depth")
                            .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            .foregroundStyle(.secondary)

                        Stepper(value: $settings.fileScanDepth, in: 1...12) {
                            Text("\(settings.fileScanDepth)")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                                .frame(width: 50, alignment: .leading)
                        }
                        .frame(width: 190, alignment: .leading)

                        Text("How many directory levels to index")
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                            .lineLimit(1)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    HStack(spacing: 10) {
                        Text("File Scan Limit")
                            .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            .foregroundStyle(.secondary)

                        Stepper(value: $settings.fileScanLimit, in: 500...50000, step: 500) {
                            Text("\(settings.fileScanLimit)")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                                .frame(width: 70, alignment: .leading)
                        }
                        .frame(width: 220, alignment: .leading)

                        Text("Max files indexed per refresh")
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                            .lineLimit(1)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Divider()
                        .overlay(.white.opacity(0.1))
                        .padding(.vertical, 4)

                    Text("Privacy & Logs")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                    Toggle(isOn: $settings.translateAllowNetwork) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Enable network translation")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            Text("Allow t\"... to send text to translation API")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                                .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                        }
                    }

                    HStack(spacing: 10) {
                        Text("Backend Log Level")
                            .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            .foregroundStyle(.secondary)

                        Picker("Backend Log Level", selection: $settings.backendLogLevel) {
                            ForEach(BackendLogLevel.allCases) { level in
                                Text(level.title).tag(level)
                            }
                        }
                        .pickerStyle(.menu)
                        .labelsHidden()
                        .frame(width: AppConstants.ThemeUI.pickerWidth)

                        Text("Error only by default; use Info/Debug for troubleshooting")
                            .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                            .lineLimit(1)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }

                    Divider()
                        .overlay(.white.opacity(0.1))
                        .padding(.vertical, 4)

                    Text("Startup")
                        .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .semibold))
                        .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                    Toggle(isOn: $settings.launchAtLogin) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Launch at login")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                            Text("Start look automatically when you sign in")
                                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                                .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
                        }
                    }
                }
            }
            .scrollIndicators(.hidden)

            Spacer(minLength: 0)

            Text(HintText.Settings.advancedApply)
                .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 2), weight: .regular))
                .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.64))
        }
    }

    private var shortcutsTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                ShortcutSection(
                    title: "Core launcher",
                    items: [
                        ShortcutItem(keys: "Tab", action: "Select next result"),
                        ShortcutItem(keys: "Shift+Tab", action: "Select previous result"),
                        ShortcutItem(keys: "Up / Down", action: "Move selection"),
                        ShortcutItem(keys: "Cmd+Enter", action: "Search query on Google"),
                        ShortcutItem(keys: "Cmd+/", action: "Enter command mode"),
                        ShortcutItem(keys: "Esc", action: "Back to app list (in command mode)"),
                        ShortcutItem(keys: "Shift+Esc", action: "Hide launcher"),
                    ]
                )

                ShortcutSection(
                    title: "Panels",
                    items: [
                        ShortcutItem(keys: "Cmd+Shift+,", action: "Open/close theme and docs panel"),
                        ShortcutItem(keys: "Cmd+Shift+;", action: "Reload .look.config"),
                        ShortcutItem(keys: "Save Config", action: "Write current UI settings to .look.config"),
                    ]
                )

                ShortcutSection(
                    title: "Zoom",
                    items: [
                        ShortcutItem(keys: "Cmd+-", action: "Zoom out UI scale"),
                        ShortcutItem(keys: "Cmd+=", action: "Zoom in UI scale"),
                        ShortcutItem(keys: "Cmd+0", action: "Reset UI scale"),
                    ]
                )

                ShortcutSection(
                    title: "Theme controls",
                    items: [
                        ShortcutItem(keys: "Appearance tab", action: "Tint, blur material, blur opacity"),
                        ShortcutItem(keys: "Advanced tab", action: "Background, indexing, privacy, logging controls"),
                        ShortcutItem(keys: "Shortcuts tab", action: "In-app keyboard documentation"),
                    ]
                )

                Text("This panel is intended as living documentation. We can add command and workflow docs here as features grow.")
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                    .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))

                Text(HintText.Settings.shortcutsTips)
                    .font(themeStore.uiFont(size: CGFloat(settings.fontSize - 1), weight: .regular))
                    .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
            }
            .padding(.top, 4)
        }
        .scrollIndicators(.hidden)
    }

    private func selectBackgroundImage() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.allowedContentTypes = [.image]
        if panel.runModal() == .OK {
            themeStore.setBackgroundImage(url: panel.url)
        }
    }
}

private struct ShortcutItem: Identifiable {
    let id = UUID()
    let keys: String
    let action: String
}

private struct ShortcutSection: View {
    @EnvironmentObject private var themeStore: ThemeStore

    let title: String
    let items: [ShortcutItem]

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .medium))
                .foregroundStyle(.secondary)

            ForEach(items) { item in
                HStack(alignment: .firstTextBaseline, spacing: 10) {
                    Text(item.keys)
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(.white.opacity(0.14), in: Capsule())
                    Text(item.action)
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                        .foregroundStyle(.primary)
                    Spacer(minLength: 0)
                }
            }
        }
    }
}

private struct LabeledSlider: View {
    @EnvironmentObject private var themeStore: ThemeStore

    let title: String
    @Binding var value: Double
    let range: ClosedRange<Double>

    var body: some View {
        HStack(spacing: 10) {
            Text(title)
                .frame(width: AppConstants.ThemeUI.labelWidth, alignment: .leading)
                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                .foregroundStyle(.secondary)
            Slider(value: $value, in: range)
                .controlSize(.mini)
                .tint(themeStore.fontColor(opacityMultiplier: 0.92))
            Text(value, format: .number.precision(.fractionLength(2)))
                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                .frame(width: 42, alignment: .trailing)
                .foregroundStyle(.secondary)
        }
    }
}
