# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-02-08

### Added

- Live autocomplete dropdown: typing `/` shows all commands with descriptions in a visual dropdown above the input
- Interactive `/config` command with subcommands: `model`, `thinking`, `compact`, `reset`
- CSS sibling combinators: `+` (adjacent sibling) and `~` (general sibling) selectors in TCSS
- In-process local LLM inference via mistralrs (optional `mistralrs` feature)
- TUI runtime (`SaorsaUi`) with retained-mode rendering and render throttling
- TCSS `apply` module for stylesheet application
- Test environment helpers (`test_env.rs`) for safe concurrent test isolation
- `Suggestion` struct with optional description field for autocomplete
- `AutocompleteAccept` input action variant
- 28+ new tests for autocomplete, config, and input handling

### Changed

- `Autocomplete` commands now include descriptions: `Vec<(&str, &str)>` pairs
- `AppState` extended with autocomplete state management (suggestions, index, navigation)
- Input handler intercepts Up/Down/Tab/Enter/Escape when autocomplete dropdown is visible
- `/settings` command now accepts subcommands for changing model, thinking level, and compact mode
- `SaorsaTuiError` renamed from `SaorsaCoreError` to match crate rename
- Improved error types and documentation across all crates

### Fixed

- Render throttle test reliability on macOS CI
- `#[cfg(feature = "mistralrs")]` gating for conditional compilation correctness

## [0.3.0] - 2026-02-07

### Added

- UX overhaul: render throttling, scrollback, functional slash commands, model management
- Complete Milestone 9 implementation

## [0.2.0] - 2026-02-06

### Added

- Initial multi-provider LLM support
- Agent runtime with tool execution

## [0.1.0] - 2026-02-05

### Added

- Initial release: TUI framework, compositor, CSS styling, widget library
