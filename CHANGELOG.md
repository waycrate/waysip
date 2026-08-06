# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### libwaysip

#### Added

- optional **`benchmark`** feature for profiling the render/dispatch loop by [@id3v1669](https://github.com/id3v1669) in [#120](https://github.com/waycrate/waysip/pull/120).

#### Changed

- redraw only the surface that actually changed instead of every output on each frame by [@id3v1669](https://github.com/id3v1669) in [#114](https://github.com/waycrate/waysip/pull/114).

### waysip

#### Added

- optional **`freeze`** feature and **`--freeze`** flag: freezes the screen while a selection is in progress by [@Gigas002](https://github.com/Gigas002) in [#123](https://github.com/waycrate/waysip/pull/123).
- **`--edit-selection`** / **`-e`** flag: after making an area/dimensions selection, shows draggable handles on the 4 corners so the rectangle can be adjusted before confirming with a hotkey (Enter by default, configurable with **`--edit-selection-key <keycode>`**); the whole rectangle can also be dragged from its interior to move it by [@Gigas002](https://github.com/Gigas002) in [#124](https://github.com/waycrate/waysip/pull/124).
- optional **`logger`** feature exposing a **`--log-level`** flag, and optional **`completions`** feature exposing **`--completions <SHELL>`** for shell completion generation by [@Gigas002](https://github.com/Gigas002) in [#116](https://github.com/waycrate/waysip/pull/116).

#### Changed

- CLI implementation refactored out of `main.rs` into focused modules (`cli`, `settings`, `utils`), and `Args` renamed to `Cli` to line up with wayshot's CLI style by [@Gigas002](https://github.com/Gigas002) in [#116](https://github.com/waycrate/waysip/pull/116).
- CI reworked: `rust.yml` replaced by a `deploy.yml` build pipeline, plus new `cargo deny` and `fmt`/`clippy` jobs by [@Gigas002](https://github.com/Gigas002) in [#116](https://github.com/waycrate/waysip/pull/116).

#### Fixed

- <kbd>Esc</kbd> now cancels an in-progress edit-selection drag instead of confirming it by [@Gigas002](https://github.com/Gigas002) in [#126](https://github.com/waycrate/waysip/pull/126).

## [0.6.1] - 2026-03-24

### libwaysip

#### Fixed

- `-g`/dimensions-or-output selection spinning the CPU at low frame rates by [@id3v1669](https://github.com/id3v1669) in [#113](https://github.com/waycrate/waysip/pull/113).
- selection now only reacts to input on the currently active surface, instead of every output by [@Decodetalkers](https://github.com/Decodetalkers) in [`eb99ccf`](https://github.com/waycrate/waysip/commit/eb99ccfc71c001519038d97168a7591aebb35de3).

[0.6.1]: https://github.com/waycrate/waysip/compare/v0.6.0...v0.6.1

## [0.6.0] - 2025-12-18

### libwaysip

#### Fixed

- panic when a "dimensions or output" selection didn't play the output's role correctly by [@Decodetalkers](https://github.com/Decodetalkers) in [`f2a8e9e`](https://github.com/waycrate/waysip/commit/f2a8e9e66baa5a4a0746be0c276e8a6cf6cf24c5).

#### Changed

- namespace of the layer-shell surface set to `osk` by [@Decodetalkers](https://github.com/Decodetalkers) in [`55cdf6e`](https://github.com/waycrate/waysip/commit/55cdf6e8173c2bb65981b39c1b8033d42da63408).
- "output and dimensions" mode now also renders the dimensions text overlay, matching the other selection modes by [@Decodetalkers](https://github.com/Decodetalkers) in [`07a8ecc`](https://github.com/waycrate/waysip/commit/07a8ecc075b89c309a9946c67fd7dfd817294d52).

[0.6.0]: https://github.com/waycrate/waysip/compare/v0.5.0...v0.6.0

## [0.5.0] - 2025-09-19

### libwaysip

#### Added

- slurp-style combined `-d`/`-o` mode: pick an output, then draw a region confined to it, in one interaction by [@cyrinux](https://github.com/cyrinux) in [#80](https://github.com/waycrate/waysip/pull/80).
- `-f`/aspect-ratio-locked selection support by [@id3v1669](https://github.com/id3v1669) in [#65](https://github.com/waycrate/waysip/pull/65).
- `-r` flag for selecting one of several predefined boxes/regions by [@id3v1669](https://github.com/id3v1669) in [#65](https://github.com/waycrate/waysip/pull/65).
- colors configurable for the option boxes drawn by `-r`, and a background plate behind the dimensions text so it stays readable over any wallpaper by [@id3v1669](https://github.com/id3v1669) in [#65](https://github.com/waycrate/waysip/pull/65).
- Nix dev shell for building/packaging with a nightly toolchain by [@id3v1669](https://github.com/id3v1669) in [#62](https://github.com/waycrate/waysip/pull/62).

#### Changed

- font and text layout are now cached instead of being rebuilt on every render, cutting per-frame overhead by [@id3v1669](https://github.com/id3v1669) in [#64](https://github.com/waycrate/waysip/pull/64).
- `dimensions` selection performance workaround, and `Color` changed from `Clone` to `Copy` by [@id3v1669](https://github.com/id3v1669) in [#63](https://github.com/waycrate/waysip/pull/63) and [#64](https://github.com/waycrate/waysip/pull/64).
- internal event loop timeout tuned to 8ms as the most responsive value that doesn't waste CPU by [@id3v1669](https://github.com/id3v1669) in [#65](https://github.com/waycrate/waysip/pull/65).
- `PassingData` renamed to `Style`, with a proper `Default` impl and a `hex_to_color` helper on `Color`; internal `OnceLock` globals moved onto `LayerSurfaceInfo` by [@id3v1669](https://github.com/id3v1669) in [#64](https://github.com/waycrate/waysip/pull/64).
- error handling switched from boxed errors to `thiserror` by [@id3v1669](https://github.com/id3v1669) in [#64](https://github.com/waycrate/waysip/pull/64).

#### Breaking Changes

- the global/singleton style state removed from the public API by [@Decodetalkers](https://github.com/Decodetalkers) in [`17c84d1`](https://github.com/waycrate/waysip/commit/17c84d1add5450d88dbd424067bb8336330d0a24).
  - **Migration:** pass state explicitly instead of relying on the removed globals; see `libwaysip/examples/base.rs`.
- selection API reshaped to accommodate the new modes (`-r`, `-f`, combined `-d`/`-o`) by [@Decodetalkers](https://github.com/Decodetalkers) in [`c139f55`](https://github.com/waycrate/waysip/commit/c139f556c3069118eced5e4809e4fa408d1d85ec).
  - **Migration:** update call sites to the current signatures in `libwaysip/src/lib.rs` and `waysip/src/main.rs`.

### waysip

#### Added

- `-d`/`-o` combined mode, `-f` aspect ratio, `-r` predefined boxes, and box colors exposed on the CLI by [@cyrinux](https://github.com/cyrinux) in [#80](https://github.com/waycrate/waysip/pull/80) and [@id3v1669](https://github.com/id3v1669) in [#65](https://github.com/waycrate/waysip/pull/65).

#### Fixed

- dependency on the unmaintained `atty` crate removed by [@Decodetalkers](https://github.com/Decodetalkers) in [`e2f869b`](https://github.com/waycrate/waysip/commit/e2f869bcd0fe807eca3d28ccaff0fb7c77043e62).

[0.5.0]: https://github.com/waycrate/waysip/compare/v0.4.0...v0.5.0

## [0.4.0] - 2025-05-10

Promotes `0.4.0-rc1` (below) to a stable release; no further code changes.

[0.4.0]: https://github.com/waycrate/waysip/compare/v0.4.0-rc1...v0.4.0

## [0.4.0-rc1] - 2025-05-10

### libwaysip

#### Changed

- public API tidied up across `dispatch`, `lib`, `render` and `state` ahead of the 0.4 release by [@Decodetalkers](https://github.com/Decodetalkers) in [`3acdac9`](https://github.com/waycrate/waysip/commit/3acdac9b87ff9d399e13c457df2d4b02f2055d65).

[0.4.0-rc1]: https://github.com/waycrate/waysip/compare/v0.3.0...v0.4.0-rc1

## [0.3.0] - 2025-03-24

### libwaysip

#### Changed

- edition bumped to Rust 2024 by [@Decodetalkers](https://github.com/Decodetalkers) in [`7ce9073`](https://github.com/waycrate/waysip/commit/7ce9073447b0d80d48e2b2de0ddb4a86782ac322).
- buffer/style release tracking reworked around the flash fix below by [@Decodetalkers](https://github.com/Decodetalkers) in [`00bbf19`](https://github.com/waycrate/waysip/commit/00bbf196a051e2f23fa75eaccc44715770ae7052).
- `std::mem::take` used instead of `ManuallyDrop` when releasing buffers by [@Decodetalkers](https://github.com/Decodetalkers) in [`8b50236`](https://github.com/waycrate/waysip/commit/8b5023614db4b275f708965bcbe7dde52e60d6fe).

#### Fixed

- a flashing/flicker problem during selection by [@Decodetalkers](https://github.com/Decodetalkers) in [`4163430`](https://github.com/waycrate/waysip/commit/41634307a6adcd281b43405058de956a3d0bb461).
- a just-released buffer was still considered busy, delaying the next redraw by [@Decodetalkers](https://github.com/Decodetalkers) in [`d5ce2a7`](https://github.com/waycrate/waysip/commit/d5ce2a71022796ba24ac52edf4e28e81e77cdf03).

[0.3.0]: https://github.com/waycrate/waysip/compare/v0.2.7...v0.3.0

## [0.2.7] - 2025-02-10

### libwaysip

#### Fixed

- `wl_surface` handles were never destroyed, leaking a surface per run; tightened up in two follow-up fixes by [@Decodetalkers](https://github.com/Decodetalkers) in [`6efc146`](https://github.com/waycrate/waysip/commit/6efc146b259612e1626f53de775ed5fe7707b2d2) and [#49](https://github.com/waycrate/waysip/pull/49).

[0.2.7]: https://github.com/waycrate/waysip/compare/v0.2.6...v0.2.7

## [0.2.6] - 2025-02-10

### libwaysip

#### Fixed

- the layer-shell surface was not destroyed on drop by [@Decodetalkers](https://github.com/Decodetalkers) in [`a42b724`](https://github.com/waycrate/waysip/commit/a42b724b1b31943903ffaef924cba759413d124f).

[0.2.6]: https://github.com/waycrate/waysip/compare/v0.2.5...v0.2.6

## [0.2.5] - 2025-02-10

### Changed

- dependencies updated by [@Decodetalkers](https://github.com/Decodetalkers).

[0.2.5]: https://github.com/waycrate/waysip/compare/v0.2.4...v0.2.5

## [0.2.4] - 2025-02-10

### libwaysip

#### Changed

- selection rendering performance improved by [@Decodetalkers](https://github.com/Decodetalkers) in [`0a65869`](https://github.com/waycrate/waysip/commit/0a65869ed9e3cf4bfbeca29c5d7d03e8c369ab7d).

[0.2.4]: https://github.com/waycrate/waysip/compare/v0.2.3...v0.2.4

## [0.2.3] - 2024-06-19

### libwaysip

#### Added

- support for a client-supplied Wayland connection and global list, instead of always connecting internally by [@Shinyzenith](https://github.com/Shinyzenith) in [#9](https://github.com/waycrate/waysip/pull/9).
- state and dispatch logic split into their own files for readability by [@Shinyzenith](https://github.com/Shinyzenith) in [#7](https://github.com/waycrate/waysip/pull/7).
- `SelectionType` re-exported at the crate root by [@Shinyzenith](https://github.com/Shinyzenith) in [#7](https://github.com/waycrate/waysip/pull/7).

#### Changed

- redundant application state structs removed, and the ambiguous `WaySipKind` struct renamed by [@Shinyzenith](https://github.com/Shinyzenith) in [#7](https://github.com/waycrate/waysip/pull/7).
- Cargo/CI dependencies bumped, including `actions/checkout`, `cachix/install-nix-action` and `crate-ci/typos` by [@Decodetalkers](https://github.com/Decodetalkers).

#### Fixed

- a misspelling of "dimensions" in the public API by [@Shinyzenith](https://github.com/Shinyzenith) in [`6256101`](https://github.com/waycrate/waysip/commit/625610187041dc8fb6a3abaa44927aab58882a6c).

### waysip

#### Changed

- the display name is now shown while choosing a screen by [@Decodetalkers](https://github.com/Decodetalkers) in [`83598ff`](https://github.com/waycrate/waysip/commit/83598ff90dcb40f3bf8dfc7cd7f8507783beac85).

[0.2.3]: https://github.com/waycrate/waysip/compare/v0.2.2...v0.2.3

## [0.2.2] - 2023-12-19

### libwaysip

#### Changed

- the internal `wl_output` handle is no longer exposed on the public output info, since it wasn't actionable for consumers by [@Decodetalkers](https://github.com/Decodetalkers) in [`3959636`](https://github.com/waycrate/waysip/commit/395963680c1d06bc16e7c6c0336debc5a383e74e).

[0.2.2]: https://github.com/waycrate/waysip/compare/v0.2.1...v0.2.2

## [0.2.1] - 2023-12-19

### libwaysip

#### Added

- output info now records the underlying `wl_output` handle by [@Decodetalkers](https://github.com/Decodetalkers) in [`ca430c5`](https://github.com/waycrate/waysip/commit/ca430c5729ed4dee823e0808bff592ec34fcc706).

[0.2.1]: https://github.com/waycrate/waysip/compare/v0.2.0...v0.2.1

## [0.2.0] - 2023-12-19

### libwaysip

#### Added

- point selection mode, alongside area/region selection by [@Decodetalkers](https://github.com/Decodetalkers) in [`753a905`](https://github.com/waycrate/waysip/commit/753a90526b475aec83b5397cb4c1714295331789).
- an interactive screen/output chooser by [@Decodetalkers](https://github.com/Decodetalkers) in [`f796b3f`](https://github.com/waycrate/waysip/commit/f796b3f2565076cfc454526f7b78d76f388c6c8d).
- point and width/height now returned as `i32`, matching Wayland's coordinate types by [@Decodetalkers](https://github.com/Decodetalkers) in [`615b5ff`](https://github.com/waycrate/waysip/commit/615b5ff36828206e54c84a8023c3e150c4327e82).

### waysip

#### Added

- `-o`/output-only selection mode exposed on the CLI by [@Decodetalkers](https://github.com/Decodetalkers) in [`cbf9c45`](https://github.com/waycrate/waysip/commit/cbf9c4550f06795c530701fa6be27ed58ddd64f9).

[0.2.0]: https://github.com/waycrate/waysip/compare/v0.1.1...v0.2.0

## [0.1.1] - 2023-12-18

Initial release.

### libwaysip

#### Added

- Wayland-native region and area selection built on `wlr-layer-shell`, drawn with `cairo`, as a `slurp` alternative that doesn't need a compositor-specific protocol shim.
- cursor theming via `wp-cursor-shape-manager`, falling back to a client-side cursor otherwise, by [@Decodetalkers](https://github.com/Decodetalkers) in [#3](https://github.com/waycrate/waysip/pull/3).
- selection color configurable via environment variable, with a sensible default theme when unset by [@Decodetalkers](https://github.com/Decodetalkers) in [`7ea1fab`](https://github.com/waycrate/waysip/commit/7ea1fab16a21996c4811727ca5693a3765072d6b).

### waysip

#### Added

- CLI front-end printing the selected region/point/output for use in scripts (e.g. piping into a screenshot tool).

### Infra

- CI (GitHub Actions) and a Nix development flake for building the workspace by [@Shinyzenith](https://github.com/Shinyzenith) in [`d2db6b9`](https://github.com/waycrate/waysip/commit/d2db6b9f77024a51afc50d3ea2d125ef1290722b) and [`960abd7`](https://github.com/waycrate/waysip/commit/960abd7fc1154e607c784f9d8c2dc94cfb3531ea).

[0.1.1]: https://github.com/waycrate/waysip/releases/tag/v0.1.1
[Unreleased]: https://github.com/waycrate/waysip/compare/v0.6.1...HEAD
