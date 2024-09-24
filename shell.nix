{ pkgs ? import <nixpkgs> {}}:
let 
    esp-rs-src = builtins.fetchTarball "https://github.com/leighleighleigh/esp-rs-nix/archive/master.tar.gz";
    esp-rs = pkgs.callPackage "${esp-rs-src}/esp-rs/default.nix" {};
in
pkgs.mkShell rec {
    name = "esp-rs-nix";

    buildInputs = [ esp-rs pkgs.rustup pkgs.espflash pkgs.rust-analyzer pkgs.pkg-config pkgs.stdenv.cc pkgs.bacon pkgs.systemdMinimal pkgs.just pkgs.lunarvim pkgs.duckdb pkgs.gnuplot pkgs.inotify-tools ];

    shellHook = ''
    # this is important - it tells rustup where to find the esp toolchain,
    # without needing to copy it into your local ~/.rustup/ folder.
    export RUSTUP_TOOLCHAIN=${esp-rs}
    '';
}
