{
  fenix,
  pkgs,
  lib,
  ...
}: let
  toolchain = fenix.packages.${pkgs.system}.latest.toolchain;
  rustPlatformNightly = pkgs.makeRustPlatform {
    cargo = toolchain;
    rustc = toolchain;
  };
in
  rustPlatformNightly.buildRustPackage rec {
    pname = "waysip";
    version = "0.5.0-dev";

    auditable = false;

    src = lib.cleanSource ../.;

    cargoLock.lockFile = "${src}/Cargo.lock";

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    buildInputs = with pkgs; [
      glib
      pango
      cairo
    ];
    postFixup = ''
      patchelf $out/bin/waysip \
        --add-rpath ${lib.makeLibraryPath buildInputs}
    '';
  }
