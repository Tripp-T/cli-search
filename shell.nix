{ pkgs ? import <nixpkgs> {} }:
    pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
            rustup
            rust-analyzer
            cargo-watch
            clippy
            pkg-config
            openssl
        ];
        packages = with pkgs; [
            just
        ];
        shellHook = ''
            alias j=just
        '';
    }