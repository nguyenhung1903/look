# Rofi-like macOS Launcher — Project Starter Spec

## 1. Project Goal

Build a **minimal, extremely fast, lightweight macOS launcher** inspired by the interaction model of **rofi**.

The product should focus on only two core capabilities:

1. **Application launching**
2. **Local file and folder search by name**

The launcher should be:
- **keyboard-first**
- **local-first**
- **native-feeling on macOS**
- **simple to understand**
- **free of unnecessary UI and features**
- **fast enough to feel instant**

This is **not** intended to be a Spotlight clone, a workflow automation platform, or a plugin marketplace.

A better description is:

> A rofi-style launcher for macOS with ultra-fast local search.

---

## 2. Product Principles

The project should follow these principles from the start:

### 2.1 Performance first
Every technical and product decision should be evaluated primarily by latency, memory usage, and implementation simplicity.

Questions to ask before adding anything:
- Does this make startup slower?
- Does this increase query latency?
- Does this increase idle CPU or memory usage?
- Does this complicate the UI?
- Does this move the product away from its core use case?

If the answer is yes, the feature should probably be rejected.

### 2.2 Minimal UI
The UI should remain extremely small in scope:
- one launcher window
- one input field
- one result list
- minimal metadata
- no decorative panels
- no large previews
- no unnecessary animations

### 2.3 Keyboard-first UX
The launcher must be usable without touching the mouse.

Core interactions:
- open launcher
- type query
- move selection with keyboard
- confirm with Enter
- dismiss with Escape
- use modifier shortcuts for alternate actions

### 2.4 Local-first and privacy-respecting
The app should operate entirely on local machine data for the main feature set.

No cloud dependency should be required for search or launching.

### 2.5 Predictable behavior
The user should understand what is being indexed, where search results come from, and how to control indexed locations.

---

## 3. Non-Goals

The following should be explicitly out of scope for the first versions:

- plugin platform
- clipboard history
- calculator
- web search integrations
- OCR
- semantic / vector search
- content indexing of documents
- command workflows
- automation pipelines
- theming system
- cloud sync
- complex settings UI
- multi-pane dashboard UI

These can be reconsidered later, but they should not influence the initial architecture more than necessary.

---

## 4. Target User

The target user is a keyboard-oriented macOS user who wants:
- something faster and simpler than Spotlight
- a launcher closer in spirit to rofi / dmenu
- minimal visual overhead
- local search with deterministic behavior

This includes:
- developers
- power users
- terminal users
- users who prefer low-friction tools over feature-heavy utility apps

---

## 5. Core Feature Scope

## 5.1 V1 Features

### App launching
- Index installed applications from:
  - `/Applications`
  - `/System/Applications`
  - `~/Applications`
- Search by app name
- Launch selected application with `Enter`

### File and folder search
- Search by **file name** and **folder name only**
- Default indexed locations:
  - `~/Desktop`
  - `~/Documents`
  - `~/Downloads`
- Allow users to add more indexed directories later
- Actions:
  - `Enter`: open
  - `Cmd+Enter`: reveal in Finder
  - `Option+Enter` or similar: copy full path

### Launcher UI
- Global hotkey to open launcher
- Floating launcher window
- Focus input automatically when opened
- Realtime result updates while typing
- Keyboard navigation with up/down arrows
- Escape to close

### Basic ranking
- Fuzzy matching
- Prefix and exact-match boosts
- Light history-based ranking based on past selections

### Basic settings
- global hotkey
- launch at login
- indexed folders
- excluded folders
- reset ranking history

---

## 6. UX Model

The product should feel close to rofi in interaction flow, while still behaving like a native macOS app.

### 6.1 Interaction pipeline
The main interaction model is:

`query -> candidate collection -> scoring -> top results -> selection -> action`

### 6.2 Window behavior
- Small floating window
- Center or top-center placement
- Appears immediately after hotkey press
- Closes immediately on dismissal or action completion
- No persistent main window required for normal usage

### 6.3 Result presentation
Each result should contain only the information needed for quick selection:
- title
- type indicator (app/file/folder)
- optional short secondary text (such as path)

Visual density should remain high, but readable.

### 6.4 Input and keyboard handling
Required shortcuts:
- `Hotkey`: open launcher
- `Enter`: execute default action
- `Esc`: close launcher
- `Up/Down`: move selection
- `Cmd+Enter`: alternate action (reveal)
- `Tab`: optional autocomplete / query completion

---

## 7. Performance Targets

These targets should guide engineering choices.

### Startup and interaction
- Launcher visible after hotkey press: ideally under **50 ms perceived latency**
- Query-to-results update: ideally under **10 ms** for top-N results from in-memory index
- No visible lag while typing

### Resource usage
- Idle CPU usage should be near zero
- Memory footprint should remain small and stable
- Background indexing must not interfere with interactive search

### Indexing
- Initial app indexing should be fast
- File indexing should happen incrementally and in background
- Reconciliation should be lightweight

These are design goals, not strict guarantees for every machine, but they should strongly shape implementation priorities.

---

## 8. Overall Stack

## 8.1 Recommended stack

### UI and macOS integration
- **AppKit** for launcher window behavior and keyboard control
- **Swift** for native macOS application layer
- Optional **SwiftUI** only for small settings screens if useful

### Core search and indexing engine
- **Rust** for:
  - indexing
  - search
  - scoring / ranking
  - storage-facing core logic

### Storage
- **SQLite** for metadata and persistent state

### File watching and system integration
- Native macOS APIs for:
  - global hotkeys
  - application discovery
  - file system notifications
  - app launching
  - Finder-related actions

## 8.2 Why this stack

### Why not a web stack?
The product depends on:
- very low latency
- tight keyboard handling
- native macOS launcher behavior
- low memory footprint

A browser-based or Electron-style stack adds unnecessary overhead for this product.

### Why Rust for the core?
Rust is a strong fit for:
- low-level performance-sensitive logic
- memory efficiency
- deterministic core behavior
- clear separation between UI and engine
- future portability if needed

### Why AppKit instead of fully SwiftUI?
A launcher window is sensitive to:
- focus behavior
- keyboard navigation
- floating window control
- activation/deactivation behavior

AppKit generally provides more direct control over these details.

---

## 9. Architecture Overview

The architecture should remain simple and layered.

```text
macOS App Shell (Swift / AppKit)
    |
    | bridge / FFI
    v
Core Engine (Rust)
    |- sources
    |- matcher
    |- ranker
    |- indexer
    |- storage
```

## 9.1 Main modules

### A. macOS App Shell
Responsibilities:
- app lifecycle
- global hotkey registration
- launcher window creation and display
- keyboard input and selection handling
- rendering the result list
- dispatching actions to the system
- opening settings UI

### B. Core Engine
Responsibilities:
- loading indexes
- candidate retrieval
- fuzzy matching
- ranking
- history weighting
- returning top results efficiently

### C. Source Layer
Two initial sources:
- applications
- files/folders

Each source should produce a normalized candidate shape.

### D. Indexer
Responsibilities:
- initial scanning
- incremental updates
- file system watch handling
- persistence updates

### E. Storage Layer
Responsibilities:
- metadata persistence
- configuration persistence
- launch/select history persistence
- index state tracking

---

## 10. Candidate Model

A single normalized candidate model helps keep the pipeline simple.

Example conceptual structure:

```text
Candidate {
  id
  kind            // app | file | folder
  title
  subtitle        // optional path or metadata
  path
  score_data
  last_used_at
  use_count
}
```

### Candidate kinds
Initial kinds:
- app
- file
- folder

The UI should not need to know much more than this.

---

## 11. Search and Ranking

## 11.1 Matching behavior
The matching engine should support:
- exact matches
- prefix matches
- subsequence / fuzzy matches
- acronym-like matching where useful

Examples:
- `saf` -> Safari
- `vsc` -> Visual Studio Code
- `doc` -> Documents / Docker depending on score and usage history

## 11.2 Ranking model
A simple scoring formula is preferred at first.

Example conceptual score:

`final_score = fuzzy_score + exact_bonus + prefix_bonus + history_bonus + recency_bonus`

The ranking system should remain:
- simple
- explainable
- cheap to compute
- easy to benchmark

## 11.3 Ranking principles
- Exact matches should rank very high
- Strong prefix matches should rank above weaker fuzzy matches
- Frequently selected results should get a boost
- Recently used results should get a smaller boost
- Weak matches in deep or noisy paths should be penalized where needed

---

## 12. Indexing Strategy

## 12.1 Applications
Initial app scan should include:
- `/Applications`
- `/System/Applications`
- `~/Applications`

Captured metadata may include:
- display name
- bundle path
- bundle identifier if needed

## 12.2 Files and folders
Initial indexed locations:
- `~/Desktop`
- `~/Documents`
- `~/Downloads`

Captured metadata should remain minimal:
- full path
- file/folder name
- parent path
- extension
- modified time
- whether item is a directory

## 12.3 What should not be indexed initially
Do not index:
- file contents
- OCR text
- browser history
- mail data
- messages
- cloud sources

## 12.4 Update strategy
Use a combination of:
- initial crawl
- file system notifications
- lightweight periodic reconciliation

Search should query a local persisted index, not crawl live paths on every keystroke.

---

## 13. Repository Strategy

Yes — **a single source repository is the right choice** for this project.

A monorepo is appropriate because:
- the app shell and core engine are tightly related
- the project is still small in scope
- shared versioning is simpler
- OSS contributors can clone one repository and get the full project
- architecture and release flow stay easier to understand

Unless the project grows much larger later, there is little benefit in splitting it into multiple repositories now.

## 13.1 Recommended repository layout

```text
repo/
  README.md
  LICENSE
  docs/
    architecture.md
    roadmap.md
    decisions/
  apps/
    macos/
      LauncherApp/
  core/
    engine/
    indexing/
    matching/
    ranking/
    storage/
  bridge/
    ffi/
  scripts/
  benchmarks/
  assets/
  examples/
```

## 13.2 Why monorepo is useful for OSS
A single repository helps with:
- onboarding
- issue tracking
- project visibility
- architecture clarity
- CI setup
- release management
- keeping docs next to code

It also prevents fragmentation at an early stage.

---

## 14. Suggested Internal Boundaries

Even inside a monorepo, the code should be separated by responsibility.

### `apps/macos`
Contains:
- native app entrypoint
- launcher window
- result list UI
- settings UI
- system action execution
- macOS integration code

### `core/*`
Contains:
- candidate source logic
- indexing pipeline
- fuzzy matching implementation
- ranking logic
- query engine
- persistence logic

### `bridge/ffi`
Contains:
- bridging layer between Swift and Rust
- narrow, stable interface between UI shell and engine

### `benchmarks/`
Contains:
- query latency tests
- ranking benchmarks
- indexing benchmarks

Benchmarks should exist early for a product like this.

---

## 15. Open Source Considerations

## 15.1 License
Recommended choices:
- **MIT**
- **Apache-2.0**

MIT is the simplest default if the goal is broad adoption.

## 15.2 OSS priorities
To make the project contributor-friendly, prepare early:
- clear README
- architecture document
- roadmap
- local development instructions
- performance goals
- issue labels
- benchmark instructions

## 15.3 What matters most for an OSS launcher project
Contributors need to understand:
- project scope
- what is intentionally excluded
- where performance matters
- how modules are separated
- how to test latency-sensitive changes

---

## 16. Development Phases

## Phase 1 — App launcher only
- create launcher window
- implement global hotkey
- index installed apps
- search apps
- launch selected app

## Phase 2 — File and folder search
- index target directories
- implement file/folder search by name
- add open / reveal / copy path actions

## Phase 3 — Ranking and tuning
- selection history
- recency/frequency ranking
- benchmark query latency
- reduce startup cost
- reduce memory usage

## Phase 4 — Product polish
- settings UI
- launch at login
- indexed path management
- exclusion rules
- packaging and release process
- contributor docs

---

## 17. First Milestone Definition

A usable first milestone should satisfy the following:

- open launcher with hotkey
- type a query
- see matching apps immediately
- launch selected app with Enter
- search indexed files/folders by name
- open or reveal them
- no noticeable lag on normal datasets

If this works reliably and feels fast, the project already has value.

---

## 18. Final Direction Statement

This project should remain focused on a narrow and strong identity:

> A minimal, rofi-inspired macOS launcher for ultra-fast app launching and local file-name search.

The project should prefer:
- simplicity over feature count
- latency over novelty
- native behavior over cross-platform abstraction
- clarity over configurability

That focus is likely the main reason the project could become genuinely useful.
