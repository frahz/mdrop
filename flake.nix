{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs systems (
          system:
          function (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (
        pkgs:
        let
          inherit (pkgs) lib;
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              pkgs.rust-bin.stable.latest.default
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath [
              pkgs.libGL
              pkgs.libxkbcommon
              pkgs.wayland
            ];
          };
        }
      );
      packages = forAllSystems (
        pkgs:
        let
          inherit (pkgs) lib;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./mdrop
              ./mdrop-cli
              ./mdrop-gui
              ./Cargo.lock
              ./Cargo.toml
            ];
          };
          meta = with lib; {
            description = "Linux CLI tool for controlling Moondrop USB audio dongles.";
            homepage = "https://github.com/frahz/mdrop";
            license = licenses.mit;
          };

        in
        rec {
          mdrop =
            let
              cargoToml = builtins.fromTOML (builtins.readFile ./mdrop-cli/Cargo.toml);
            in
            pkgs.rustPlatform.buildRustPackage {
              inherit (cargoToml.package) name version;
              inherit meta src;

              cargoLock.lockFile = ./Cargo.lock;
              cargoFlags = [
                "--bin"
                "mdrop"
              ];
            };
          gui =
            let
              inherit (pkgs) lib stdenv;

              cargoToml = builtins.fromTOML (builtins.readFile ./mdrop-gui/Cargo.toml);
            in
            pkgs.rustPlatform.buildRustPackage {
              inherit (cargoToml.package) name version;
              inherit meta src;

              cargoLock.lockFile = ./Cargo.lock;
              cargoFlags = [
                "--bin"
                "mdrop-gui"
              ];

              postFixup = lib.optionalString stdenv.hostPlatform.isLinux ''
                patchelf $out/bin/mdrop-gui \
                  --add-rpath ${
                    lib.makeLibraryPath [
                      pkgs.libGL
                      pkgs.libxkbcommon
                      pkgs.wayland
                    ]
                  }
              '';
            };
          default = mdrop;
        }
      );
    };
}
