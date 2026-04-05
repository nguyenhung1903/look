cask "look" do
  version "1.0.0"
  sha256 "REPLACE_WITH_RELEASE_ZIP_SHA256"

  url "https://github.com/kunkka19xx/look/releases/download/v#{version}/Look-#{version}-macOS.zip"
  name "look"
  desc "Keyboard-first local launcher for macOS"
  homepage "https://github.com/kunkka19xx/look"

  app "Look.app"
  binary "Look.app/Contents/MacOS/Look", target: "lookapp"
end
