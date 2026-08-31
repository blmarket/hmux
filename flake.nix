{
  description = "hmux development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustStableFor = pkgs: pkgs.rust-bin.stable.latest.minimal;
      rustNightlyFor = pkgs: pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" "rust-analyzer" "llvm-tools-preview" ];
      };
      tmux37bFor = pkgs: pkgs.callPackage ./nix/tmux.nix { };
      agentmonFor = pkgs: pkgs.callPackage ./agentmon-tui/package.nix { };
      cargoLlvmCovFor = pkgs: pkgs.callPackage ./nix/cargo-llvm-cov.nix { };
      c2rustFor = pkgs: pkgs.callPackage ./nix/c2rust.nix {
        rustStable = rustStableFor pkgs;
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustNightly = rustNightlyFor pkgs;
          # Build with the same nightly toolchain the dev shell uses; the crate
          # needs `extern_types` and `local_waker`.
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustNightly;
            rustc = rustNightly;
          };
          # The server ships as `hmux`. The crate's [[bin]] keeps tmux's own
          # name because that is what the conformance harness drives it as, so
          # the rename happens here rather than in the manifest.
          hmux = rustPlatform.buildRustPackage {
            pname = "hmux";
            version = "0.0.0";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            # build.rs links these unconditionally, which is also why this
            # package is Linux-only.
            buildInputs = [
              pkgs.ncurses
              pkgs.utf8proc
              pkgs.libutempter
              pkgs.systemd
            ];

            # The unit tests reach into the server's process-wide state and
            # need a process each; `make unit-c2rs` is where they run.
            doCheck = false;

            postInstall = ''
              mv $out/bin/tmux $out/bin/hmux
            '';

            meta.mainProgram = "hmux";
          };
        in
        {
          agentmon = agentmonFor pkgs;
          c2rust = c2rustFor pkgs;
          tmux = tmux37bFor pkgs;
        }
        // nixpkgs.lib.optionalAttrs (pkgs.stdenv.hostPlatform.isLinux) {
          inherit hmux;
          default = hmux;
        });

      apps = forAllSystems (system: {
        agentmon = {
          type = "app";
          program = "${self.packages.${system}.agentmon}/bin/agentmon";
          meta.description = "Create and monitor coding-agent runs through hmux";
        };
        looper = {
          type = "app";
          program = "${self.packages.${system}.agentmon}/bin/looper";
          meta.description = "Run a coding agent in a loop through hmux";
        };
      });

      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          tmux37b = tmux37bFor pkgs;
          llvm = pkgs.llvmPackages;
          rustStable = rustStableFor pkgs;
          rustNightly = rustNightlyFor pkgs;
          c2rust = c2rustFor pkgs;
          cargoLlvmCov = cargoLlvmCovFor pkgs;
          agentmon = agentmonFor pkgs;
        in
        {
          default = pkgs.mkShell ({
            TMUX_SRC = tmux37b.src;
            RUST_SRC_PATH = "${rustNightly}/lib/rustlib/src/rust/library";

            ASAN_SYMBOLIZER_PATH = "${llvm.llvm}/bin/llvm-symbolizer";

            CMAKE_LLVM_DIR = "${llvm.libllvm.dev}/lib/cmake/llvm";
            CMAKE_CLANG_DIR = "${llvm.libclang.dev}/lib/cmake/clang";
            LLVM_CONFIG_PATH = "${llvm.libllvm.dev}/bin/llvm-config";
            CLANG_PATH = "${llvm.clang}/bin/clang";
            LIBCLANG_PATH = "${llvm.libclang.lib}/lib";

            C2RUST_BUILD_CARGO = "${rustStable}/bin/cargo";
            C2RUST_BUILD_RUSTC = "${rustStable}/bin/rustc";

            packages = [
              tmux37b
              rustNightly
              c2rust
              agentmon # the `agentmon` dashboard and the `looper` loop runner
              pkgs.cargo-nextest
              cargoLlvmCov
              pkgs.gnumake
              pkgs.git
              pkgs.python3
              pkgs.jq
              pkgs.bear
              pkgs.cmake
              pkgs.pkg-config
              llvm.clang
              llvm.llvm
              llvm.libclang
              llvm.libllvm
            ]
            # Valgrind does not build for aarch64-darwin.
            ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              pkgs.valgrind
            ];

            buildInputs = [
              pkgs.openssl
              pkgs.zlib
              llvm.libclang
              llvm.libllvm
            ]
            ++ (tmux37b.buildInputs or [ ]);

            nativeBuildInputs = tmux37b.nativeBuildInputs or [ ];
          } // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath
              (map (p: p.lib or p.out or p) (tmux37b.buildInputs or [ ]));
            LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";
            LOCALE_ARCHIVE_2_27 = "${pkgs.glibcLocales}/lib/locale/locale-archive";
          });
        });
    };
}
