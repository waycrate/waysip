<p align=center>
  <p align=center>A native, blazing-fast 🚀🚀🚀 area selection tool for wlroots based compositors such as sway and river.</p>

  <p align="center">
  <a href="./LICENSE.md"><img src="https://img.shields.io/github/license/waycrate/waysip?style=flat-square&logo=appveyor"></a>
  <img src="https://img.shields.io/badge/cargo-v0.6.1-green?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/issues/waycrate/waysip?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/forks/waycrate/waysip?style=flat-square&logo=appveyor">
  <img src="https://img.shields.io/github/stars/waycrate/waysip?style=flat-square&logo=appveyor">
  <br>
  <img src="https://repology.org/badge/vertical-allrepos/waysip.svg">
  </p>
</p>

# Some usage examples:

NOTE: Run `waysip --help` for the full list of flags and options.

Interactive rectangular area (prints position and size using the default format):

```bash
waysip -d
```

Pick a single point:

```bash
waysip -p
```

Print information about the focused screen:

```bash
waysip -i
```

Select a display output:

```bash
waysip -o
```

Combined dimensions / output mode (single click selects an output; drag selects a region):

```bash
waysip -d -o
```

Restrict selection to predefined boxes (pipe one box per line: `x,y WIDTHxHEIGHT`):

```bash
printf '100,200 400x300\n' | waysip -r
```

Custom output format (see `%` placeholders in `--help`; default is `%x,%y %wx%h\n`):

```bash
waysip -d -f '%x %y %w %h\n'
```

Shell completions:

```bash
waysip --completions fish | source
waysip --completions zsh > ~/.zfunc/_waysip
waysip --completions bash > /etc/bash_completion.d/waysip
waysip --completions elvish >> ~/.config/elvish/rc.elv
waysip --completions pwsh >> $PROFILE
waysip --completions nushell | save -f ~/.config/nushell/completions/waysip.nu
```

Freeze the screen while selecting, so the visible desktop stays static instead of updating live (requires the optional `freeze` feature, see below):

```bash
waysip --freeze -d
```

# Optional features

All features except `benchmark` and `freeze` are enabled in the default build. To reduce binary size or compile-time dependencies, features can be selectively disabled, or `freeze` can be opted into:

```bash
cargo build --no-default-features --features frame-limiter
cargo build --no-default-features --features logger
cargo build --no-default-features --features completions
cargo build --no-default-features --features frame-limit,logger,completions
cargo build --features freeze
```

| Feature       | What it adds                                                                     | Extra dependency          |
| ------------- | -------------------------------------------------------------------------------- | ------------------------- |
| `frame-limit` | Workaround to fix frametime issue on low frequency CPUs                          | None                      |
| `logger`      | `--log-level` flag, tracing output to stderr                                     | tracing-subscriber        |
| `completions` | `--completions <SHELL>`, generate shell completion scripts                       | clap_complete (+ nushell) |
| `benchmark`   | Benchmarking options for development described [here](#development-benchmarking) | None                      |
| `freeze`      | `--freeze`, freeze the screen while selecting                                    | libwayshot, image         |

# Installation

## Compile time dependencies:

- rustup (Rust toolchain)
- pkg-config
- wayland
- cairo
- pango

## Compiling:

- `git clone https://github.com/waycrate/waysip && cd waysip`
- `cargo build --release`
- `sudo mv ./target/release/waysip /usr/local/bin`

## Using Nix flakes (nixOS / Nix)

This repository provides a Nix flake for building and running waysip.

### Build

```bash
nix build github:waycrate/waysip
```

### Run

```bash
nix run github:waycrate/waysip
```

# Development benchmarking

To enable benchmarking options use one of those options depending on usecase described later:

```bash
cargo build --no-default-features --features "logger completions benchmark"
cargo build --no-default-features --features "logger completions frame-limit benchmark"
```

Regarding `frame-limit` feature. It is a workaround enabled by default meant to solve issue with unstable frametime on low freq CPUs or under heavy load.

## Benchmark usage

### `--bench`

Records each `wl_callback.done` timestamp from the compositor for the focused screen (duplicate timestamps are dropped, so it's roughly one entry per frame) and prints frame stats on stderr when the selection ends.

Reported metrics (written to stderr):

- `fps avg`
- `frametime: min / avg / max`
- `latencies` - array of entries meant track where frametime issues happened

For drag modes the `raw duration` and the amount trimmed from each end are also printed.
For non-drag options less frames-better.

To test frametime stability try lowering your cpu freq to 0.8 GHz and build without frame-limit feature before testing.

# Support:

1. https://matrix.to/#/#waycrate-tools:matrix.org
2. https://discord.gg/KKZRDYrRYW
