import SwiftUI

struct SearchInputBar: View {
    @Binding var text: String
    @Binding var isCommandMode: Bool
    let isQueryFocused: FocusState<Bool>.Binding
    let activeCommand: AppCommand?
    let themeStore: ThemeStore
    let onSubmit: () -> Void
    let onExitCommandMode: () -> Void

    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: isCommandMode ? "terminal" : "magnifyingglass")
                .foregroundStyle(isCommandMode ? .green : .secondary)
            TextField(
                isCommandMode
                    ? (activeCommand?.placeholder ?? "Choose a command with Tab")
                    : "Search apps",
                text: $text
            )
                .textFieldStyle(.plain)
                .focused(isQueryFocused)
                .onTapGesture {
                    DispatchQueue.main.async {
                        isQueryFocused.wrappedValue = true
                    }
                }
                .onSubmit(onSubmit)

            if isCommandMode {
                if let command = activeCommand {
                    Text("/\(command.title)")
                        .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                        .foregroundStyle(themeStore.fontColor())
                        .padding(.horizontal, 8)
                        .padding(.vertical, 3)
                        .background(.green.opacity(0.18), in: Capsule())
                }
                Button("Exit") { onExitCommandMode() }
                    .keyboardShortcut(.escape, modifiers: [.shift])
                    .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                    .buttonStyle(.plain)
                    .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .background(.white.opacity(0.14), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }
}

struct CommandFeedbackView: View {
    let message: String
    let themeStore: ThemeStore

    var body: some View {
        Text(message)
            .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize + 4), weight: .semibold))
            .foregroundStyle(themeStore.fontColor())
            .lineLimit(30)
    }
}

struct CommandListView: View {
    let commands: [AppCommand]
    let selectedID: String?
    let activeID: String?
    let themeStore: ThemeStore
    let onSelect: (String) -> Void

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 4) {
                ForEach(commands) { command in
                    HStack(spacing: 10) {
                        Image(systemName: "terminal")
                            .frame(width: 22, height: 22)
                            .foregroundStyle(.green)
                        VStack(alignment: .leading, spacing: 2) {
                            Text("/\(command.title)")
                                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize), weight: .semibold))
                            Text(command.detail)
                                .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
                                .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
                        }
                        Spacer(minLength: 0)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(
                        (selectedID == command.id || activeID == command.id)
                            ? .green.opacity(0.20) : .white.opacity(0.08),
                        in: RoundedRectangle(cornerRadius: 8, style: .continuous)
                    )
                    .onTapGesture { onSelect(command.id) }
                }
            }
            .padding(2)
        }
        .frame(maxHeight: AppConstants.Launcher.commandListMaxHeight)
    }
}

struct ResultsListView: View {
    let results: [LauncherResult]
    let selectedID: String?
    let themeStore: ThemeStore
    let onSelect: (String) -> Void
    let onOpen: (String) -> Void

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 4) {
                    ForEach(results) { result in
                        LauncherRowView(
                            result: result,
                            isSelected: selectedID == result.id,
                            onOpen: {
                                onSelect(result.id)
                                onOpen(result.id)
                            }
                        )
                        .id(result.id)
                    }
                }
                .padding(2)
            }
            .onChange(of: selectedID) { _, newID in
                guard let newID else { return }
                withAnimation(.easeOut(duration: 0.12)) {
                    proxy.scrollTo(newID, anchor: .center)
                }
            }
        }
    }
}

struct HintBar: View {
    let isCommandMode: Bool
    let themeStore: ThemeStore

    var body: some View {
        Text(isCommandMode ? AppConstants.Launcher.commandHint : AppConstants.Launcher.normalHint)
            .font(themeStore.uiFont(size: CGFloat(themeStore.settings.fontSize - 1), weight: .regular))
            .foregroundStyle(themeStore.fontColor(opacityMultiplier: 0.72))
    }
}
