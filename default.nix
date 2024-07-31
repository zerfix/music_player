{ pkgs ? import <nixpkgs> { config.allowUnfree = true; } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    alsaLib
    pkg-config
    openssl
    rustc
    cargo
  ];
}
