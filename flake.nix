{
  description = "tmux-c2rs development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { nixpkgs, rust-overlay, ... }:
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
      cargoLlvmCovFor = pkgs: pkgs.callPackage ./nix/cargo-llvm-cov.nix { };
      c2rustFor = pkgs: pkgs.callPackage ./nix/c2rust.nix {
        rustStable = rustStableFor pkgs;
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          c2rust = c2rustFor pkgs;
          tmux = tmux37bFor pkgs;
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
