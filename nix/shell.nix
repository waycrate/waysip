{pkgs, ...}:
pkgs.mkShell rec {
  name = "Waysip-devel";
  nativeBuildInputs = with pkgs; [
    # linker
    pkg-config
    # rust
    rustc
    cargo
    scdoc

    # Tools
    rustfmt
    clippy
    rust-analyzer
    cargo-flamegraph
    cargo-audit
    cargo-xbuild
    cargo-deny
  ];
  buildInputs = with pkgs; [
    glib
    pango
  ];
}
