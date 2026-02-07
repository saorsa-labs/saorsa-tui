# saorsa-cli

Thin CLI entry point for the saorsa-tui AI coding agent.

[![Crates.io](https://img.shields.io/crates/v/saorsa-cli.svg)](https://crates.io/crates/saorsa-cli)
[![License](https://img.shields.io/crates/l/saorsa-cli.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](#minimum-supported-rust-version)

## Overview

**saorsa-cli** is the command-line entry point for the saorsa-tui project. It is a thin wrapper that delegates to `saorsa` for all application logic.

This crate exists as a separate binary to keep the CLI entry point minimal and to allow `saorsa` to be used as both a library and a binary.

## Installation

```bash
cargo install saorsa-cli
```

## Usage

```bash
# Run the AI coding agent
saorsa-cli

# For the full interactive experience, run saorsa directly:
saorsa
```

See the [saorsa README](../saorsa/README.md) for complete usage documentation including CLI arguments, slash commands, and keybindings.

## Architecture

```
saorsa-cli (thin binary)
    └→ saorsa (application logic + TUI)
         ├→ saorsa-core (TUI framework)
         ├→ saorsa-ai (LLM providers)
         └→ saorsa-agent (agent runtime + tools)
```

## Minimum Supported Rust Version

The MSRV is **1.88** (Rust Edition 2024). This is enforced in CI.

## License

Licensed under either of:

- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
- [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

Part of the [saorsa-tui](https://github.com/saorsa-labs/saorsa-tui) workspace. See the workspace root for contribution guidelines.
