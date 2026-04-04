import Foundation

enum CalcResult {
    case value(String)
    case error(String)
}

struct CalcCommand {
    static let maxMagnitude: Double = 1_000_000_000_000.0

    static func evaluate(_ expression: String) -> CalcResult {
        guard isReadyForEvaluation(expression) else {
            return .error("Invalid expression")
        }

        let normalized = decimalizeIntegerTokens(in: normalizeExpression(expression))

        if containsDivisionByZero(in: normalized) {
            return .error("Error: division by zero")
        }

        let parsed = NSExpression(format: normalized)
        guard let value = parsed.expressionValue(with: nil, context: nil) else {
            return .error("Invalid expression")
        }

        if let number = value as? NSNumber {
            let evaluated = number.doubleValue
            if abs(evaluated) > maxMagnitude {
                return .error("Error: result out of range (±1,000,000,000,000)")
            }
            return .value(formatFloat(evaluated))
        }
        return .error("Invalid expression")
    }

    static func isReadyForEvaluation(_ expression: String) -> Bool {
        let trimmed = expression.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { return false }

        var balance = 0
        for ch in trimmed {
            if ch == "(" { balance += 1 }
            if ch == ")" {
                balance -= 1
                if balance < 0 { return false }
            }
        }
        if balance != 0 { return false }

        if let last = trimmed.last, "+-*/.(".contains(last) {
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

    private static func formatFloat(_ value: Double) -> String {
        if value.isNaN || value.isInfinite { return "nan" }

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

    private static func decimalizeIntegerTokens(in expression: String) -> String {
        let pattern = "(?<![A-Za-z0-9_\\.])([0-9]+)(?![A-Za-z0-9_\\.])"
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

    private static func containsDivisionByZero(in expression: String) -> Bool {
        let pattern = "/\\s*0+(?:\\.0+)?(?:\\b|(?=\\)))"
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return false }
        let range = NSRange(expression.startIndex..<expression.endIndex, in: expression)
        return regex.firstMatch(in: expression, range: range) != nil
    }

    private static func normalizeExpression(_ expression: String) -> String {
        var normalized = expression
            .replacingOccurrences(of: "x", with: "*")
            .replacingOccurrences(of: "X", with: "*")
            .replacingOccurrences(of: ":", with: "/")
        return replacePrefixSqrt(in: normalized)
    }

    private static func replacePrefixSqrt(in expression: String) -> String {
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
}
