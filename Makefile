SHELL := /bin/bash

XCODE_PROJECT := apps/macos/LauncherApp/look-app.xcodeproj
XCODE_SCHEME := Look
XCODE_CONFIG := Debug
XCODE_DERIVED_DATA := .build/xcode
APP_BUNDLE := $(XCODE_DERIVED_DATA)/Build/Products/$(XCODE_CONFIG)/Look.app
APP_BINARY := $(APP_BUNDLE)/Contents/MacOS/Look
REAL_DB_PATH := $(HOME)/Library/Application Support/look/look.db

.PHONY: help core-check ffi-check app-build app-run app-open symbols db-path db-status db-shell db-reset db-refresh

help:
	@printf "look developer tasks\n\n"
	@printf "  make core-check   - cargo check core workspace\n"
	@printf "  make ffi-check    - cargo check ffi crate\n"
	@printf "  make app-build    - xcodebuild app (includes rust ffi build/link)\n"
	@printf "  make app-run      - run app binary with real default db path\n"
	@printf "  make app-open     - open built app bundle\n"
	@printf "  make symbols      - verify ffi symbols in app binary\n"
	@printf "  make db-path      - print real db path\n"
	@printf "  make db-status    - inspect candidates/usage tables\n"
	@printf "  make db-shell     - open sqlite shell for real db\n"
	@printf "  make db-reset     - delete real db file\n"
	@printf "  make db-refresh   - delete db and rebuild app\n"

core-check:
	cargo check --workspace --manifest-path core/Cargo.toml

ffi-check:
	cargo check --manifest-path bridge/ffi/Cargo.toml

app-build:
	xcodebuild -project "$(XCODE_PROJECT)" -scheme "$(XCODE_SCHEME)" -configuration "$(XCODE_CONFIG)" -derivedDataPath "$(XCODE_DERIVED_DATA)" build

app-run: app-build
	@if [ ! -d "$(APP_BUNDLE)" ]; then echo "missing app bundle: $(APP_BUNDLE)"; exit 1; fi
	@echo "Opening app with real db path (LOOK_DB_PATH unset): $(REAL_DB_PATH)"
	@env -u LOOK_DB_PATH open "$(APP_BUNDLE)"

app-open: app-build
	@if [ ! -d "$(APP_BUNDLE)" ]; then echo "missing app bundle: $(APP_BUNDLE)"; exit 1; fi
	@open "$(APP_BUNDLE)"

symbols: app-build
	@nm -gU "$(XCODE_DERIVED_DATA)/Build/Products/$(XCODE_CONFIG)/Look.app/Contents/MacOS/Look.debug.dylib" | rg "_look_search_json|_look_record_usage|_look_free_cstring"

db-path:
	@echo "$(REAL_DB_PATH)"

db-status:
	@sqlite3 "$(REAL_DB_PATH)" "SELECT id,title,use_count,last_used_at_unix_s FROM candidates ORDER BY use_count DESC, title ASC LIMIT 20;"
	@echo "---"
	@sqlite3 "$(REAL_DB_PATH)" "SELECT candidate_id,action,used_at_unix_s FROM usage_events ORDER BY id DESC LIMIT 20;"

db-shell:
	sqlite3 "$(REAL_DB_PATH)"

db-reset:
	@rm -f "$(REAL_DB_PATH)"
	@echo "Removed: $(REAL_DB_PATH)"

db-refresh: db-reset app-build
	@echo "DB refreshed and app rebuilt"
