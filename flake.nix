{
  description = "Monad rekry";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        with pkgs; 
        rec {
          devShells.default = mkShell {
              buildInputs = with pkgs; [
                cargo rustc rustfmt pre-commit rustPackages.clippy openssl
              ];

              nativeBuildInputs = with pkgs; [
                pkg-config
              ];

              RUST_SRC_PATH = rustPlatform.rustLibSrc;
            };
        }
      );
}

