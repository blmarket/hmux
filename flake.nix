{
  description = "tmux-compatible server with agent control";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  # nixpkgs ships stable rustc only; the nightly toolchain comes from here.
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      tmux37bFor = pkgs: pkgs.callPackage ./nix/tmux.nix { };
      agentmonFor = pkgs: pkgs.callPackage ./agentmon-tui/package.nix { };
      # The default profile already carries cargo, rustfmt, and clippy.
      rustNightlyFor = pkgs: pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" ];
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          tmux37b = tmux37bFor pkgs;
          agentmon = agentmonFor pkgs;
          rustNightly = rustNightlyFor pkgs;
          # Build hmux with the same nightly toolchain the dev shell uses.
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustNightly;
            rustc = rustNightly;
          };
          hmux = rustPlatform.buildRustPackage {
            pname = "hmux";
            version = "0.1.0";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
              pkgs.apple-sdk
            ];
          };
        in
        {
          inherit agentmon hmux;
          tmux = tmux37b;
          default = hmux;
        });

      apps = forAllSystems (system: {
        agentmon = {
          type = "app";
          program = "${self.packages.${system}.agentmon}/bin/agentmon";
          meta.description = "Create and monitor coding-agent runs through hmux";
        };
      });

      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          tmux37b = tmux37bFor pkgs;
          agentmon = agentmonFor pkgs;
          rustNightly = rustNightlyFor pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = [
              tmux37b
              rustNightly
              pkgs.cargo-nextest
              agentmon # the `agentmon` dashboard and the `looper` loop runner
            ];

            RUST_SRC_PATH = "${rustNightly}/lib/rustlib/src/rust/library";
          };
        });
    };
}
