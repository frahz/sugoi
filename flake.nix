{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

        buildInputs = with pkgs; [
          rust-bin.stable.latest.default
          openssl
          pkg-config
        ];
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
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

        devShells.default = with pkgs;
          mkShell {
            inherit buildInputs;
          };
      }
    );
}
