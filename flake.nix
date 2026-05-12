{
  description = "Vectomancy - Image to Parametric Equations CLI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "vectomancy";
          version = "4.0.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          buildInputs = with pkgs; [ vulkan-loader pkg-config ];
        };
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            pkg-config
            vulkan-loader
          ];
          LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
        };
      }
    );
}