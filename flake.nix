{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";

    alpine-js-src = {
      url = "https://cdn.jsdelivr.net/npm/alpinejs@3.14.9/dist/cdn.min.js";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    alpine-js-src,
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
          pkgs.bacon
          pkgs.rust-bin.stable.latest.default
          pkgs.sqlite
          pkgs.tailwindcss_3
          (pkgs.writeShellScriptBin "tw-watch" ''
            ${pkgs.tailwindcss_3}/bin/tailwindcss -i ./assets/tailwind.css -o ./assets/main.css --watch
          '')
          (pkgs.writeShellScriptBin "tw-prod" ''
            ${pkgs.tailwindcss_3}/bin/tailwindcss -i ./assets/tailwind.css -o ./assets/main.css --minify
          '')
        ];
        shellHook = ''
          mkdir -p assets
          cp -f ${alpine-js-src} assets/alpine.min.js
        '';
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
        buildInputs = [pkgs.sqlite];

        postUnpack = ''
          # pushd source
          mkdir -p assets
          cp -f ${alpine-js-src} assets/alpine.min.js
          # popd
        '';

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
    nixosModules.default = import ./nix/module.nix self;
  };
}
