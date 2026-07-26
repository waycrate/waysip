# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Optional `freeze` feature and `--freeze` flag: freezes the screen while a selection is in progress.
- `--edit-selection` / `-e` flag: after making an area/dimensions selection, shows draggable
  handles on the 4 corners so the rectangle can be adjusted before confirming with a hotkey
  (Enter by default, configurable with `--edit-selection-key <keycode>`). The whole rectangle
  can also be dragged from its interior to move it.

## [0.6.1] - 2026-03-24

### Changed

- Fixed panic on hyperland by @id3v1669
- Set the namespace to osk
- make dimension_or_output also show text
- Fix comment in Error
- Updated deps

[0.6.1]: https://github.com/waycrate/waysip/compare/v0.6.0...v0.6.1
