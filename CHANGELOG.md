# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.4.0] - 2026-02-11

### Breaking

- Refactored core device APIs to be target-aware and fallible: getters/setters now use `Result`-based return values and explicit device selection.
- Removed implicit fan-out writes when multiple dongles are connected; commands now require `-s/--device BB:DD` in multi-device scenarios.

### Added

- Added typed core error model (`MdropError`) and result alias (`MdropResult`) for transport, selection, and payload failures.
- Added `DeviceSelector` to support explicit bus-targeted operations.
- Added payload validation and strict enum parsing (`TryFrom<u8>`) for filter/gain/indicator state.
- Added optional `clap` feature in the core crate to enable `ValueEnum` derives when used by CLI consumers.
- Added JSON output mode for `get` and `devices` commands.

### Changed

- Updated CLI and GUI to use the new result-based core API.
- Restored CLI `set` argument value hints by enabling core enums as clap `ValueEnum` behind feature flag.
- Improved GUI update loop to avoid synthetic disconnect flicker and handle watch/update failures without panics.
- Updated Nix and workspace dependencies, including migration to newer Iced.

### Fixed

- Fixed overflow when setting volume to `100`.
- Improved USB/watch failure handling by replacing panic-prone control flow with surfaced errors.

### Docs

- Updated `README.md` to document deterministic device selection behavior and multi-device usage requirements.

## [0.3.0] - 2025-06-10

### Added

- Added `get` subcommands for `filter`, `gain`, and `indicator-state`.

### Changed

- Bumped project version to `0.3.0`.
- Updated lockfile and dependencies.

## [0.2.0] - 2025-04-01

### Added

- Added initial GUI application (`mdrop-gui`) with dongle detection and volume controls.
- Added Nix packaging for GUI and macOS support.
- Added logging integration and additional CLI/UX improvements.

### Changed

- Migrated USB backend from `rusb` to `nusb`.
- Refined volume model using a dedicated newtype and improved GUI interactions.

### Fixed

- Fixed `mdrop devices` behavior when no devices are connected.
- Fixed Linux GUI runtime wrapping for `libwayland`.
