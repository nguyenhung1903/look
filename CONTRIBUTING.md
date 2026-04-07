# Contributing to look

Thanks for contributing.

## Before you open an issue

- search existing issues first to avoid duplicates
- use a clear title with area prefix when possible (`ui:`, `engine:`, `indexing:`, `ffi:`)
- include enough context so someone else can reproduce quickly

## Bug reports

A good bug report must include:

- expected behavior
- actual behavior
- exact reproduction steps (numbered)
- frequency (`always`, `sometimes`, `once`)
- environment details:
  - macOS version
  - look app version or commit SHA
  - install method (Xcode run, zip install, Homebrew tap)
  - architecture (`arm64` or `x86_64`)
- logs or screenshots if available

If crash related, include:

- crash dialog text
- stack trace or Xcode console output
- whether it happens on clean launch

## Feature requests

Please include:

- problem statement (what pain exists today)
- proposed behavior
- alternatives considered
- impact/risk (perf, UX, safety)

## Development setup

Prerequisites:

- macOS + Xcode
- Rust stable toolchain

Checks:

```bash
./scripts/bootstrap.sh
cargo test --workspace --manifest-path core/Cargo.toml
cargo test --manifest-path bridge/ffi/Cargo.toml
```

## Branch and PR flow

- default contributor target branch is `dev`
- open PRs to `main` only for hotfixes or release-critical patches coordinated with maintainers
- keep `main` stable/releasable; regular feature and refactor work should merge through `dev`

Suggested local flow:

```bash
git fetch origin
git checkout dev
git pull --ff-only origin dev
git checkout -b feat/short-description
```

Before opening PR:

- rebase/merge latest `dev`
- run local checks from the Development setup section
- ensure docs are updated when behavior changes

## CI behavior

CI runs for pushes and pull requests targeting `dev` and `main`.

- Rust jobs (`lint`, `test`, `cargo-audit`, release `build`) run only when Rust-related paths change
- secrets scanning (`gitleaks`) always runs
- macOS app build runs only for PRs to `dev`/`main` when Swift files change
- release-style Rust build artifacts run only on push to `main`

## Pull request checklist

- scope is focused and minimal
- base branch is `dev` (unless maintainer requested `main`)
- docs updated when behavior changes
- no unrelated formatting-only changes
- tests/checks pass locally
- PR description explains why this change is needed

## Commit style

Keep commits small and descriptive.

Recommended prefixes:

- `fix:` bug fix
- `feat:` new behavior
- `docs:` documentation
- `refactor:` internal cleanup without behavior change
- `test:` tests only
