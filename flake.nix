{
  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";

    alpine-js-src = {
      url = "https://cdn.jsdelivr.net/npm/alpinejs@3.14.9/dist/cdn.min.js";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      alpine-js-src,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems =
        function: nixpkgs.lib.genAttrs systems (system: function nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = builtins.attrValues {
            inherit (pkgs)
              bacon
              cargo
              clippy
              rustc
              rustfmt
              sqlite
              tailwindcss_3
              ;
            tw-watch = pkgs.writeShellScriptBin "tw-watch" ''
              ${pkgs.tailwindcss_3}/bin/tailwindcss -i ./assets/tailwind.css -o ./assets/main.css --watch
            '';
            tw-prod = pkgs.writeShellScriptBin "tw-prod" ''
              ${pkgs.tailwindcss_3}/bin/tailwindcss -i ./assets/tailwind.css -o ./assets/main.css --minify
            '';
          };
          shellHook = ''
            mkdir -p assets
            cp -f ${alpine-js-src} assets/alpine.min.js
          '';
        };
      });

      packages = forAllSystems (
        pkgs:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            inherit (cargoToml.package) name version;

            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
            nativeBuildInputs = [ pkgs.makeWrapper ];
            buildInputs = [ pkgs.sqlite ];

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
        }
      );
      nixosModules.default = import ./nix/module.nix self;
    };
}
