# Homebrew Release Plan

This checklist tracks the work required to ship look via Homebrew (cask).

## Step-by-step work plan (custom tap first)

1. Build and package signed macOS app artifact (`Look.app` -> zip)
2. Publish release artifact to GitHub Releases
3. Generate/update custom tap cask (`look.rb`) with URL + sha256
4. Test install/upgrade/uninstall flow with `brew install --cask`
5. Document user install instructions in `README.md` and `docs/user-guide.md`

## Current status

- [x] Step 1 scaffolded: `scripts/release-macos-app.sh`
- [ ] Step 2 publish flow
- [x] Step 3 scaffolded: `scripts/generate-homebrew-cask.sh`
- [ ] Step 4 local validation
- [ ] Step 5 docs updates

Latest local packaging run:

- version: `1.0.0`
- artifact: `dist/Look-1.0.0-macOS.zip`
- sha256: `9d03d144278b72d690af5c9e7b9964f847e17e1ac75e7bcec77260d92fede32d`
- generated cask: `.build/homebrew-tap/Casks/look.rb`

## How to build artifact

```bash
./scripts/release-macos-app.sh 1.0.0
```

or with manifest output:

```bash
./scripts/build-release.sh 1.0.0
```

Outputs:

- `dist/Look-<version>-macOS.zip`
- printed `SHA256` for cask update
- `dist/Look-<version>-manifest.txt` (when using `build-release.sh`)

## Curl installer script (prepare-only)

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | LOOK_VERSION=1.0.0 bash
```

Options:

- `--version <x.y.z>` or env `LOOK_VERSION`
- `--repo <owner/repo>` or env `LOOK_REPO`
- `--url <direct-zip-url>` or env `LOOK_DOWNLOAD_URL`

## How to generate cask for your tap

```bash
./scripts/generate-homebrew-cask.sh 1.0.0 <sha256> <github-owner/repo>
```

This writes:

- `.build/homebrew-tap/Casks/look.rb`

Then copy `look.rb` into your tap repository at `Casks/look.rb` (example tap name: `homebrew-tap`).

## Example install flow (after cask is in tap)

```bash
brew tap <github-owner>/tap
brew install --cask look
```

## Notes

- Notarization and signing are recommended before public release.
- Homebrew cask should point to immutable GitHub Release URLs.
