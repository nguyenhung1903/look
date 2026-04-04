import Foundation
import Combine
import SwiftUI
import AppKit

final class ThemeStore: ObservableObject {
    @Published private(set) var backgroundImageURL: URL?
    @Published var uiScale: CGFloat = 1.0

    @Published var settings: ThemeSettings {
        didSet {
            save()
            if oldValue.backgroundImagePath != settings.backgroundImagePath
                || oldValue.backgroundImageBookmark != settings.backgroundImageBookmark
            {
                refreshBackgroundImageURL()
            }
        }
    }

    private let defaultsKey = "look.theme.settings"
    private var scopedBackgroundURL: URL?

    init() {
        Self.ensureDefaultConfigFileExists(at: Self.configPath())

        if let data = UserDefaults.standard.data(forKey: defaultsKey),
            let decoded = try? JSONDecoder().decode(ThemeSettings.self, from: data)
        {
            settings = decoded
        } else {
            settings = .default
        }

        applyThemeOverridesFromConfigFile()

        refreshBackgroundImageURL()
    }

    func reset() {
        settings = .default
        applyThemeOverridesFromConfigFile()
    }

    func reloadFromConfig() {
        Self.ensureDefaultConfigFileExists(at: Self.configPath())
        applyThemeOverridesFromConfigFile()
    }

    func saveCurrentConfigToFile() -> Bool {
        let path = Self.configPath()
        Self.ensureDefaultConfigFileExists(at: path)

        var lines = ((try? String(contentsOf: path, encoding: .utf8)) ?? "")
            .split(omittingEmptySubsequences: false, whereSeparator: \ .isNewline)
            .map(String.init)

        if !lines.contains(where: { stripComment($0).trimmingCharacters(in: .whitespacesAndNewlines) == "# UI theme" }) {
            if !lines.isEmpty, !(lines.last?.isEmpty ?? true) {
                lines.append("")
            }
            lines.append("# UI theme")
        }

        upsertConfigLine(&lines, key: "ui_tint_red", value: String(format: "%.2f", settings.tintRed))
        upsertConfigLine(&lines, key: "ui_tint_green", value: String(format: "%.2f", settings.tintGreen))
        upsertConfigLine(&lines, key: "ui_tint_blue", value: String(format: "%.2f", settings.tintBlue))
        upsertConfigLine(&lines, key: "ui_tint_opacity", value: String(format: "%.2f", settings.tintOpacity))
        upsertConfigLine(&lines, key: "ui_blur_material", value: settings.blurMaterial.rawValue)
        upsertConfigLine(&lines, key: "ui_blur_opacity", value: String(format: "%.2f", settings.blurOpacity))
        upsertConfigLine(&lines, key: "ui_font_name", value: settings.fontName)
        upsertConfigLine(&lines, key: "ui_font_size", value: String(format: "%.0f", settings.fontSize))
        upsertConfigLine(&lines, key: "ui_font_red", value: String(format: "%.2f", settings.fontRed))
        upsertConfigLine(&lines, key: "ui_font_green", value: String(format: "%.2f", settings.fontGreen))
        upsertConfigLine(&lines, key: "ui_font_blue", value: String(format: "%.2f", settings.fontBlue))
        upsertConfigLine(&lines, key: "ui_font_opacity", value: String(format: "%.2f", settings.fontOpacity))
        upsertConfigLine(&lines, key: "ui_border_thickness", value: String(format: "%.2f", settings.borderThickness))
        upsertConfigLine(&lines, key: "ui_border_red", value: String(format: "%.2f", settings.borderRed))
        upsertConfigLine(&lines, key: "ui_border_green", value: String(format: "%.2f", settings.borderGreen))
        upsertConfigLine(&lines, key: "ui_border_blue", value: String(format: "%.2f", settings.borderBlue))
        upsertConfigLine(&lines, key: "ui_border_opacity", value: String(format: "%.2f", settings.borderOpacity))

        let payload = lines.joined(separator: "\n") + "\n"
        do {
            try payload.write(to: path, atomically: true, encoding: .utf8)
            return true
        } catch {
            return false
        }
    }

    func zoomIn() {
        uiScale = min(1.8, uiScale + 0.1)
    }

    func zoomOut() {
        uiScale = max(0.7, uiScale - 0.1)
    }

    func resetZoom() {
        uiScale = 1.0
    }

    func uiFont(size: CGFloat? = nil, weight: Font.Weight = .regular) -> Font {
        let baseSize = size ?? CGFloat(settings.fontSize)
        let resolvedSize = max(8, baseSize * uiScale)
        let resolvedName = settings.fontName.trimmingCharacters(in: .whitespacesAndNewlines)
        if !resolvedName.isEmpty, let fontName = resolveUsableFontName(resolvedName) {
            return .custom(fontName, size: resolvedSize).weight(weight)
        }

        return .system(size: resolvedSize, weight: weight)
    }

    func fontColor(opacityMultiplier: Double = 1.0) -> Color {
        let alpha = min(1, max(0, settings.fontOpacity * opacityMultiplier))
        return Color(red: settings.fontRed, green: settings.fontGreen, blue: settings.fontBlue, opacity: alpha)
    }

    func borderColor() -> Color {
        Color(
            red: settings.borderRed,
            green: settings.borderGreen,
            blue: settings.borderBlue,
            opacity: settings.borderOpacity
        )
    }

    func borderLineWidth() -> CGFloat {
        CGFloat(max(0.25, settings.borderThickness))
    }

    func fontNameSuggestions(for input: String, limit: Int = 8) -> [String] {
        let allFonts = NSFontManager.shared.availableFontFamilies.sorted { $0.localizedCaseInsensitiveCompare($1) == .orderedAscending }
        let query = input.trimmingCharacters(in: .whitespacesAndNewlines)
        if query.isEmpty {
            return Array(allFonts.prefix(limit))
        }

        let lowered = query.lowercased()
        var startsWithMatches = allFonts.filter { $0.lowercased().hasPrefix(lowered) }
        var containsMatches = allFonts.filter { !$0.lowercased().hasPrefix(lowered) && $0.lowercased().contains(lowered) }
        startsWithMatches.append(contentsOf: containsMatches)
        return Array(startsWithMatches.prefix(limit))
    }

    func setBackgroundImage(url: URL?) {
        guard let url else {
            settings.backgroundImagePath = nil
            settings.backgroundImageBookmark = nil
            return
        }

        let bookmark = try? url.bookmarkData(
            options: .withSecurityScope,
            includingResourceValuesForKeys: nil,
            relativeTo: nil
        )

        settings.backgroundImagePath = url.path
        settings.backgroundImageBookmark = bookmark
    }

    deinit {
        scopedBackgroundURL?.stopAccessingSecurityScopedResource()
    }

    private func save() {
        guard let data = try? JSONEncoder().encode(settings) else { return }
        UserDefaults.standard.set(data, forKey: defaultsKey)
    }

    private func applyThemeOverridesFromConfigFile() {
        guard let raw = try? String(contentsOf: Self.configPath(), encoding: .utf8) else {
            return
        }

        for line in raw.split(whereSeparator: \ .isNewline) {
            let stripped = stripComment(String(line)).trimmingCharacters(in: .whitespacesAndNewlines)
            if stripped.isEmpty {
                continue
            }

            guard let splitPoint = stripped.firstIndex(of: "=") else {
                continue
            }

            let key = stripped[..<splitPoint].trimmingCharacters(in: .whitespacesAndNewlines)
            let value = stripped[stripped.index(after: splitPoint)...].trimmingCharacters(in: .whitespacesAndNewlines)

            switch key {
            case "ui_tint_red":
                if let parsed = parseUnitDouble(value) {
                    settings.tintRed = parsed
                }
            case "ui_tint_green":
                if let parsed = parseUnitDouble(value) {
                    settings.tintGreen = parsed
                }
            case "ui_tint_blue":
                if let parsed = parseUnitDouble(value) {
                    settings.tintBlue = parsed
                }
            case "ui_tint_opacity":
                if let parsed = parseUnitDouble(value) {
                    settings.tintOpacity = parsed
                }
            case "ui_blur_material":
                if let material = parseBlurMaterial(value) {
                    settings.blurMaterial = material
                }
            case "ui_blur_opacity":
                if let parsed = parseUnitDouble(value) {
                    settings.blurOpacity = parsed
                }
            case "ui_font_name":
                if !value.isEmpty {
                    settings.fontName = value
                }
            case "ui_font_size":
                if let parsed = parsePositiveDouble(value) {
                    settings.fontSize = parsed
                }
            case "ui_font_red":
                if let parsed = parseUnitDouble(value) {
                    settings.fontRed = parsed
                }
            case "ui_font_green":
                if let parsed = parseUnitDouble(value) {
                    settings.fontGreen = parsed
                }
            case "ui_font_blue":
                if let parsed = parseUnitDouble(value) {
                    settings.fontBlue = parsed
                }
            case "ui_font_opacity":
                if let parsed = parseUnitDouble(value) {
                    settings.fontOpacity = parsed
                }
            case "ui_border_thickness":
                if let parsed = parsePositiveDouble(value) {
                    settings.borderThickness = parsed
                }
            case "ui_border_red":
                if let parsed = parseUnitDouble(value) {
                    settings.borderRed = parsed
                }
            case "ui_border_green":
                if let parsed = parseUnitDouble(value) {
                    settings.borderGreen = parsed
                }
            case "ui_border_blue":
                if let parsed = parseUnitDouble(value) {
                    settings.borderBlue = parsed
                }
            case "ui_border_opacity":
                if let parsed = parseUnitDouble(value) {
                    settings.borderOpacity = parsed
                }
            default:
                continue
            }
        }
    }

    private static func configPath() -> URL {
        let env = ProcessInfo.processInfo.environment
        if let custom = env["LOOK_CONFIG_PATH"]?.trimmingCharacters(in: .whitespacesAndNewlines),
            !custom.isEmpty
        {
            return URL(fileURLWithPath: custom)
        }

        let home = env["HOME"] ?? NSHomeDirectory()
        return URL(fileURLWithPath: home).appendingPathComponent(".look.config")
    }

    private static func ensureDefaultConfigFileExists(at path: URL) {
        if FileManager.default.fileExists(atPath: path.path) {
            return
        }

        try? defaultConfigContents.write(to: path, atomically: true, encoding: .utf8)
    }

    private func stripComment(_ line: String) -> String {
        guard let index = line.firstIndex(of: "#") else {
            return line
        }
        return String(line[..<index])
    }

    private func parseUnitDouble(_ value: String) -> Double? {
        guard let parsed = Double(value), (0...1).contains(parsed) else {
            return nil
        }
        return parsed
    }

    private func parsePositiveDouble(_ value: String) -> Double? {
        guard let parsed = Double(value), parsed > 0 else {
            return nil
        }
        return parsed
    }

    private func parseBlurMaterial(_ value: String) -> LauncherBlurMaterial? {
        switch value.lowercased() {
        case "hudwindow", "high_contrast", "high-contrast":
            return .hudWindow
        case "sidebar", "soft":
            return .sidebar
        case "menu", "balanced":
            return .menu
        case "underwindowbackground", "under_window_background", "subtle":
            return .underWindowBackground
        default:
            return LauncherBlurMaterial(rawValue: value)
        }
    }

    private func upsertConfigLine(_ lines: inout [String], key: String, value: String) {
        let wanted = "\(key)="
        for index in lines.indices {
            let trimmed = stripComment(lines[index]).trimmingCharacters(in: .whitespacesAndNewlines)
            if trimmed.hasPrefix(wanted) {
                lines[index] = "\(key)=\(value)"
                return
            }
        }
        lines.append("\(key)=\(value)")
    }

    private func resolveUsableFontName(_ input: String) -> String? {
        if let exact = NSFont(name: input, size: 12) {
            return exact.fontName
        }

        let manager = NSFontManager.shared
        if let members = manager.availableMembers(ofFontFamily: input),
            let postscriptName = extractPostScriptName(from: members)
        {
            return postscriptName
        }

        let lowercasedInput = input.lowercased()
        for family in NSFontManager.shared.availableFontFamilies {
            if family.lowercased() == lowercasedInput,
                let members = manager.availableMembers(ofFontFamily: family),
                let postscriptName = extractPostScriptName(from: members)
            {
                return postscriptName
            }
        }

        return nil
    }

    private func extractPostScriptName(from members: [[Any]]) -> String? {
        guard let firstMember = members.first,
            let postScript = firstMember.first as? String,
            !postScript.isEmpty
        else {
            return nil
        }
        return postScript
    }

    private static let defaultConfigContents = """
# look configuration
# Generated on first launch. Edit values and press Cmd+Shift+; to reload.

# Backend indexing
app_scan_roots=/Applications,/System/Applications,/System/Applications/Utilities
app_scan_depth=3
file_scan_roots=Desktop,Documents,Downloads
file_scan_depth=2
file_scan_limit=2000
skip_dir_names=node_modules,target,build,dist,library,applications,old firefox data

# UI theme
ui_tint_red=0.08
ui_tint_green=0.10
ui_tint_blue=0.12
ui_tint_opacity=0.55
ui_blur_material=hudWindow
ui_blur_opacity=0.95
ui_font_name=SF Pro Text
ui_font_size=14
ui_font_red=0.96
ui_font_green=0.96
ui_font_blue=0.98
ui_font_opacity=0.96
ui_border_thickness=1.0
ui_border_red=1.0
ui_border_green=1.0
ui_border_blue=1.0
ui_border_opacity=0.12
"""

    private func refreshBackgroundImageURL() {
        scopedBackgroundURL?.stopAccessingSecurityScopedResource()
        scopedBackgroundURL = nil
        backgroundImageURL = nil

        if let bookmark = settings.backgroundImageBookmark {
            var isStale = false
            if let resolved = try? URL(
                resolvingBookmarkData: bookmark,
                options: .withSecurityScope,
                relativeTo: nil,
                bookmarkDataIsStale: &isStale
            ) {
                _ = resolved.startAccessingSecurityScopedResource()
                scopedBackgroundURL = resolved
                backgroundImageURL = resolved
                return
            }
        }

        if let path = settings.backgroundImagePath {
            backgroundImageURL = URL(fileURLWithPath: path)
        }
    }
}
