# Phase 7.4: CI/CD & Releases

**Milestone 7 — Polish & Release (FINAL PHASE)**

## Overview

Set up GitHub Actions CI/CD pipeline, cross-compilation configuration,
crates.io publishing metadata, and release automation. This is the last
phase of the project.

## Tasks

### Task 1: CI Workflow — Format, Clippy, Tests

**Files:** `.github/workflows/ci.yml`

Create the main CI workflow:
- Trigger on push to `main` and on pull requests
- Matrix: `ubuntu-latest`, `macos-latest`, `windows-latest`
- Steps: checkout, install Rust stable + nightly (for fmt), cargo fmt --check,
  cargo clippy -- -D warnings, cargo nextest run --all-features
- Install nextest via `cargo install cargo-nextest --locked`
- Cache: `~/.cargo/registry`, `~/.cargo/git`, `target/`
- Set `RUSTFLAGS: "-D warnings"` env var
- MSRV check on separate job with `rust-toolchain` 1.85

### Task 2: CI Workflow — Security Audit & Docs

**Files:** `.github/workflows/audit.yml`, `.github/workflows/docs.yml`

- `audit.yml`: Run `cargo audit` on push to main (weekly schedule too).
  Install via `cargo install cargo-audit --locked`.
- `docs.yml`: Build docs with `cargo doc --workspace --no-deps --all-features`.
  Upload as artifact. Optionally deploy to GitHub Pages on main branch.
  Set `RUSTDOCFLAGS: "-D warnings"` to catch doc warnings.

### Task 3: Release Workflow — Binary Builds

**Files:** `.github/workflows/release.yml`

Create release workflow triggered on tag push (`v*`):
- Matrix: linux-x86_64 (ubuntu-latest), linux-aarch64 (cross),
  macos-x86_64 (macos-13), macos-aarch64 (macos-latest/14),
  windows-x86_64 (windows-latest)
- Build with `--release` profile
- Binary name: `saorsa-tui` (from saorsa-cli crate)
- Package: tar.gz for unix, zip for windows
- Include LICENSE-MIT, LICENSE-APACHE2, README.md in archive
- Upload artifacts

### Task 4: Release Workflow — GitHub Release Publishing

**Files:** `.github/workflows/release.yml` (extend from Task 3)

Extend the release workflow:
- After all matrix builds complete, create GitHub Release
- Use `softprops/action-gh-release` or `ncipollo/release-action`
- Attach all platform binaries
- Generate changelog from commits since last tag
- Mark as pre-release if version contains `-alpha`, `-beta`, `-rc`

### Task 5: Crates.io Publishing Metadata

**Files:** `Cargo.toml` (workspace), `crates/*/Cargo.toml`

Add crates.io metadata to all publishable crates:
- `categories` and `keywords` for each crate
- `readme` pointing to crate-specific or workspace README
- `rust-version` = "1.85" (MSRV)
- Ensure `description` is present on all crates (already done)
- Add `publish = false` to `saorsa-cli` (binary-only, not published to crates.io)
- Verify publish order: saorsa-core first, then saorsa-ai, saorsa-agent, saorsa-app
  (respecting dependency graph)

### Task 6: Release Profile & Build Optimization

**Files:** `Cargo.toml` (workspace)

Configure release profile for optimized binaries:
- `[profile.release]` with `lto = "thin"`, `strip = true`, `codegen-units = 1`
- `[profile.release-debug]` inheriting release with `debug = true` for profiling
- `[profile.dev]` with `opt-level = 0` for fast builds
- `[profile.test]` inheriting dev for test speed
- Verify binary size is reasonable with the release profile

### Task 7: Cross-Compilation Configuration

**Files:** `.cargo/config.toml`, `Cross.toml`

Set up cross-compilation:
- `.cargo/config.toml` with target-specific linker settings
- `Cross.toml` for `cross` tool configuration (linux-aarch64)
- Document build commands for each target in comments
- Ensure GitHub Actions workflow references correct targets

### Task 8: Integration Tests & Final Verification

**Files:** `crates/saorsa-core/tests/ci_integration.rs`

Final verification tests:
- Test that all public API types are Send + Sync where expected
- Test that all error types implement std::error::Error
- Test version consistency across workspace (all crates same version)
- Run full workspace build verification
- Verify MSRV compatibility (features available on Rust 1.85)

## Acceptance Criteria

- [ ] CI workflow runs on push/PR with format, clippy, tests
- [ ] Security audit workflow on schedule
- [ ] Release workflow builds binaries for 5 platform targets
- [ ] GitHub Release created automatically on version tag
- [ ] All crates have crates.io metadata
- [ ] Release profile configured for optimized builds
- [ ] Cross-compilation documented and configured
- [ ] Final integration tests verify API contracts
