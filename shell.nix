{ pkgs ? import <nixpkgs> {}}:
let 
    esp-rs-src = builtins.fetchGit {
        url = "https://github.com/leighleighleigh/esp-rs-nix";
        rev = "d82a564a65cf91f8018bfc18a55d5cc23be4f86c";
    };

    # This will build esp-rs-src, chosen above
    esp-rs = pkgs.callPackage "${esp-rs-src}/esp-rs/default.nix" {
        pkgs = pkgs;
        version = "1.90.0.0"; # Rust version
        crosstool-version = "15.2.0_20250920"; # Cross-compiler toolchain version (GCC)
        binutils-version = "16.3_20250913"; # Binutils version (GDB)
    };
in
pkgs.mkShell rec {
    name = "esp-rs-nix";
  

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [
        esp-rs 
        #pkgs.espflash
        pkgs.rust-analyzer
        pkgs.rustup 
        pkgs.stdenv.cc 
        pkgs.just 
        pkgs.inotify-tools
        pkgs.picocom
        pkgs.libusb1
        # for libudev
        pkgs.systemdMinimal
    ];

    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

    shellHook = ''
    # custom bashrc stuff
    export PS1_PREFIX="(esp-rs)"
    . ~/.bashrc

    export LD_LIBRARY_PATH="''${LD_LIBRARY_PATH}:${LD_LIBRARY_PATH}"
    # this is important - it tells rustup where to find the esp toolchain,
    # without needing to copy it into your local ~/.rustup/ folder.
    export RUSTUP_TOOLCHAIN=${esp-rs}

    # Load shell completions for espflash
    if (which espflash >/dev/null 2>&1); then
    . <(espflash completions $(basename $SHELL))
    fi
    '';
}
