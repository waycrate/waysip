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

# Optional features

All features are enabled in the default build. To reduce binary size or compile-time dependencies, features can be selectively disabled:

```bash
cargo build --no-default-features --features logger
cargo build --no-default-features --features completions
cargo build --no-default-features --features logger,completions
```

| Feature       | What it adds                                               | Extra dependency          |
| ------------- | ---------------------------------------------------------- | ------------------------- |
| `logger`      | `--log-level` flag, tracing output to stderr               | tracing-subscriber        |
| `completions` | `--completions <SHELL>`, generate shell completion scripts | clap_complete (+ nushell) |

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

# Support:

1. https://matrix.to/#/#waycrate-tools:matrix.org
2. https://discord.gg/KKZRDYrRYW
