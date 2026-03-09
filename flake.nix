{
  description = "Forge — deterministic execution fabric (Nix + Firecracker + SHA-256)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, crane, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
          targets = [ "wasm32-unknown-unknown" "x86_64-unknown-linux-musl" ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        forge = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Packages shared between both shells
        commonShellPackages = with pkgs; [
          cargo-nextest
          cargo-llvm-cov
          cargo-deny
          cargo-machete
          cargo-flamegraph
          cargo-fuzz
          cargo-zigbuild
          cargo-mutants
          cargo-insta

          # WASM
          wasm-pack
          wasm-bindgen-cli

          # Compilation tooling
          sccache
          mold
          wild
          clang  # linker driver for -fuse-ld=wild

          # Workflow
          just
          typos
          lefthook

          # Dev tools
          bun
          jq
          taplo
          direnv
          git
          gh

          # Forge: Firecracker microVM runtime
          firecracker
        ];

        commonShellHook = ''
          export RUSTC_WRAPPER="${pkgs.sccache}/bin/sccache"
          echo "Rust $(rustc --version) | sccache | wild linker"
        '';
      in
      {
        packages.default = forge;

        devShells = {
          # Agent shell — minimal, no interactive/human tools
          default = craneLib.devShell {
            packages = commonShellPackages;
            shellHook = commonShellHook + ''
              echo "Forge agent shell — type 'just' for commands"
            '';
          };

          # Human shell — full toolset + MoonBit
          human = craneLib.devShell {
            packages = commonShellPackages ++ (with pkgs; [
              bacon         # Background test runner
              cargo-expand  # Macro expansion viewer
            ]);
            shellHook = commonShellHook + ''
              echo "Forge human shell — type 'just' for commands"

              # MoonBit (not in nixpkgs — install via official script)
              if ! command -v moon &>/dev/null; then
                echo "Installing MoonBit toolchain..."
                curl -fsSL https://cli.moonbitlang.com/install/unix.sh | bash
              fi
              export PATH="$HOME/.moon/bin:$PATH"
            '';
          };
        };
      }
    );
}
