{ pkgs ? import <nixpkgs> { config.allowUnfree = true; } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    alsa-lib
    pkg-config
    glibc
    openssl
    rustc
    cargo
  ];
  shellHook = ''
    export PKG_CONFIG_PATH=${pkgs.alsa-lib}/lib/pkgconfig
  '';
}
