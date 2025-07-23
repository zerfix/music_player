{ pkgs ? import <nixpkgs> { config.allowUnfree = true; } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    alsa-lib
    pkg-config
    binutils
    gcc
    glibc
    openssl
    rustc
    cargo
    rust-analyzer
    lldb
  ];
  shellHook = ''
    export PKG_CONFIG_PATH=${pkgs.alsa-lib}/lib/pkgconfig
  '';
}
