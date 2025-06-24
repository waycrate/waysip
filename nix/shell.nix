{
  pkgs,
  fenix,
}: let
  fenixPkgs = fenix.packages.${pkgs.system};
  rust-toolchain = with fenixPkgs.latest;
    fenixPkgs.combine [
      cargo
      rustc
      rust-analyzer
      rustfmt
      clippy
    ];
in
  pkgs.mkShell rec {
    name = "Waysip-devel";
    nativeBuildInputs = with pkgs; [
      pkg-config
      rust-toolchain

      # Tools
      scdoc
      cargo-flamegraph
      cargo-audit
      cargo-xbuild
      cargo-deny
    ];
    buildInputs = with pkgs; [
      glib
      pango
      cairo
    ];
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
  }
