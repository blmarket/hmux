{
  description = "tmux-compatible server with agent control";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
  # nixpkgs ships stable rustc only; the nightly toolchain comes from here.
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";
  inputs.rust-overlay.inputs.nixpkgs.follows = "nixpkgs";

  # A dev shell only. The shipped `hmux`, `agentmon` and `looper` come from the
  # tmux-c2rs flake, which is what the public repository publishes; this tree
  # borrows hmux-agent and hmux-rt from there by path, so it cannot build
  # itself from its own flake root.
  outputs = { nixpkgs, rust-overlay, ... }:
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
      # The default profile already carries cargo, rustfmt, and clippy.
      rustNightlyFor = pkgs: pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" ];
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          tmux = tmux37bFor pkgs;
        });

      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          tmux37b = tmux37bFor pkgs;
          rustNightly = rustNightlyFor pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = [
              tmux37b
              rustNightly
              pkgs.cargo-nextest
            ];

            RUST_SRC_PATH = "${rustNightly}/lib/rustlib/src/rust/library";
          };
        });
    };
}
