{ lib,
  rustPlatform,
  pkg-config,
  glib,
  pango,
  ...
}: rustPlatform.buildRustPackage rec {
    pname = "waysip";
    src = lib.cleanSource ../.;

    version = "${(builtins.fromTOML (builtins.readFile (src + "/Cargo.toml"))).workspace.package.version}-git";

    cargoLock.lockFile = "${src}/Cargo.lock";

    nativeBuildInputs = [
      pkg-config
    ];

    buildInputs = [
      glib
      pango
    ];
  }
