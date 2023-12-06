{
  description = "Waysip devel";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable"; };

  outputs = { self, nixpkgs, ... }:
    let
      pkgsFor = system:
        import nixpkgs {
          inherit system;
          overlays = [ ];
        };

      targetSystems = [ "aarch64-linux" "x86_64-linux" ];
    in {
      devShells = nixpkgs.lib.genAttrs targetSystems (system:
        let pkgs = pkgsFor system;
        in {
          default = pkgs.mkShell {
            name = "Waysip-devel";
            nativeBuildInputs = with pkgs; [
              # Compilers
              cargo
              rustc
			  glib
			  pango
			  cairo
			  #pangocairo

              # Libs
              wayland-protocols
              wayland

              # Tools
			  pkg-config
              wayland-scanner
              clippy
              rust-analyzer
              rustfmt
              strace
              valgrind
            ];
          };
        });
    };
}
