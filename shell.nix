{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    cargo
    rustc
    rust-analyzer
    rustfmt
    rustPlatform.rustLibSrc
  ];

  buildInputs = with pkgs; [
    openssl
  ];

  # This tells cargo where to find the openssl headers
  shellHook = ''
    export RUST_SRC_PATH="${pkgs.rustPlatform.rustLibSrc}";
    export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
  '';
}