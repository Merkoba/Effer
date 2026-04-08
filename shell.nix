{pkgs ? import (builtins.fetchTarball {
  url = "https://github.com/NixOS/nixpkgs/archive/36a601196c4ebf49e035270e10b2d103fe39076b.tar.gz";
  sha256 = "1vs1g86i75rgpsvs7kyqfv22j6x3sg3daf4cv6ws3d0ghkb2ggpz";
}) {}}:

pkgs.mkShell {
  # These packages will only be available when you are in the nix-shell
  buildInputs = with pkgs; [
    # Rust Toolchain
    cargo
    rustc
    rustfmt
    clippy
    rust-analyzer
    rustPlatform.rustLibSrc

    # C Build Tools
    gcc
    gnumake
    file
    pkg-config

    # Native System Dependencies
    libsodium
  ];

  # Set up the environment variable for rust-analyzer to find the standard library
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
}