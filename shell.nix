{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    rustc

    rustfmt
    clippy
    rust-analyzer

    fzf
    fontconfig

    ghostty
    wezterm
  ];
}
