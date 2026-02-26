{
  description = "Ilias Scraper Tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        ilias-pkg = pkgs.rustPlatform.buildRustPackage {
          pname = "ilias";
          version = "2.3.0";
          src = ./.;
          meta = with pkgs.lib; {
            description = "Ilias Scraper Tool";
            homepage = "https://github.com/FelixSmtt/iliasScraper";
            license = licenses.mit;
          };

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [ 
            pkg-config 
            rustToolchain
          ];
          
          buildInputs = with pkgs; [ 
            openssl 
          ];

          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
        };

        ilias-desktop = pkgs.makeDesktopItem {
          name = "ilias";
          exec = "ilias";
          icon = "ilias-icon";
          comment = "Ilias Scraper Tool";
          desktopName = "Ilias Scraper";
          categories = [ "Utility" "Network" ];
          terminal = true;
        };

      in {
        packages.default = pkgs.symlinkJoin {
          name = "ilias-full";
          paths = [ ilias-pkg ilias-desktop ];
          postBuild = ''
            mkdir -p $out/share/icons/hicolor/256x256/apps
            mkdir -p $out/share/pixmaps
            
            cp ${./assets/icon.png} $out/share/icons/hicolor/256x256/apps/ilias-icon.png
            cp $out/share/icons/hicolor/256x256/apps/ilias-icon.png $out/share/pixmaps/ilias-icon.png
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain pkgs.openssl pkgs.pkg-config ];
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}