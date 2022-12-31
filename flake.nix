{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    dream2nix.url = "github:nix-community/dream2nix";
    dream2nix.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = inp:
    inp.dream2nix.lib.makeFlakeOutputs {
      systemsFromFile = ./nix_systems;
      config.projectRoot = ./.;
      source = ./.;
      projects = ./projects.toml;
      packageOverrides = {
        "^.*" = {
          set-toolchain.overrideRustToolchain = old: let
            toolchain = inp.fenix.packages.x86_64-darwin.minimal.toolchain;
          in {
            cargo = toolchain;
            rustc = toolchain;
          };
          add-missing-libiconv-on-darwin.nativeBuildInputs = old:
            let
              frameworks = with inp.nixpkgs.legacyPackages.x86_64-darwin.darwin.apple_sdk.frameworks; [
                QuartzCore
                Metal
                AppKit
                Foundation
                ApplicationServices
                CoreGraphics
                CoreVideo
                CoreFoundation
              ];
            in
              old ++ inp.nixpkgs.lib.optionals inp.nixpkgs.legacyPackages.x86_64-darwin.stdenv.isDarwin [
                # https://github.com/cargo2nix/cargo2nix/issues/187
                inp.nixpkgs.legacyPackages.x86_64-darwin.libiconv # serde
                # https://github.com/NixOS/nixpkgs/pull/187065#issuecomment-1220906696
                inp.nixpkgs.legacyPackages.x86_64-darwin.darwin.libobjc  # objc_exception
              ] ++ frameworks; # TODO cross platform
        };
      };
    };
}
