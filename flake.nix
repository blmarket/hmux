{
  description = "tmux-compatible server with agent control";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs = { self, nixpkgs, ... }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      tmux37bFor = pkgs: pkgs.callPackage ./nix/tmux.nix { };
      agentmonFor = pkgs: pkgs.callPackage ./agentmon-tui/package.nix { };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          tmux37b = tmux37bFor pkgs;
          agentmon = agentmonFor pkgs;
          hmux = pkgs.rustPlatform.buildRustPackage {
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
          pkgs = import nixpkgs { inherit system; };
          tmux37b = tmux37bFor pkgs;
          agentmon = agentmonFor pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = [
              tmux37b
              pkgs.rustc
              pkgs.rustfmt
              pkgs.clippy
              pkgs.cargo
              pkgs.cargo-nextest
              agentmon # the `agentmon` dashboard and the `looper` loop runner
            ];
          };
        });
    };
}
