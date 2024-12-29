{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ];
    forEachSystem = nixpkgs.lib.genAttrs systems;
    pkgsForEach = forEachSystem (system:
      import nixpkgs {
        inherit system;
        overlays = [rust-overlay.overlays.default];
      });
  in {
    devShells = forEachSystem (system: let
      pkgs = pkgsForEach.${system};
    in {
      default = pkgs.mkShell {
        buildInputs = [
          pkgs.rust-bin.stable.latest.default
        ];
      };
    });

    packages = forEachSystem (system: let
      pkgs = pkgsForEach.${system};
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in {
      default = pkgs.rustPlatform.buildRustPackage {
        inherit (cargoToml.package) name version;

        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
          allowBuiltinFetchGit = true;
        };
        nativeBuildInputs = [pkgs.makeWrapper];

        postInstall = ''
          mkdir -p $out/share
          cp -r assets $out/share
          wrapProgram $out/bin/sugoi \
            --set ASSETS_DIR $out/share/assets
        '';

        meta = with pkgs.lib; {
          description = "small web server for waking up and putting my server to sleep.";
          homepage = "https://git.iatze.cc/frahz/sugoi";
          changelog = "https://git.iatze.cc/frahz/sugoi/releases/tag/v${version}";
          license = licenses.mit;
        };
      };
    });
  };
}
