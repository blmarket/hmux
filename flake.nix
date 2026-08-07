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
      tmux37bFor = pkgs:
        pkgs.tmux.overrideAttrs {
          version = "3.7b";
          src = pkgs.fetchFromGitHub {
            owner = "tmux";
            repo = "tmux";
            tag = "3.7b";
            hash = "sha256-CTq06XP997M0ODxQihTq34dI9H6jSRLUXLYuTWOwDpc=";
          };
          # remove patches specified in pkgs.tmux
          patches = [ ];
        };
      agentmonFor = pkgs: pkgs.callPackage ./agentmon-tui/package.nix { };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          tmux37b = tmux37bFor pkgs;
          agentmon = agentmonFor pkgs;
          zigDeps = pkgs.callPackage ./ghostty-sys/vendor/libghostty-vt/build.zig.zon.nix {
            name = "libghostty-vt-zig-deps";
            linkFarm = name: entries:
              pkgs.runCommand name { } ''
                mkdir -p $out
                ${pkgs.lib.concatMapStringsSep "\n" (e: ''
                  cp -rL ${e.path} $out/${e.name}
                '') entries}
              '';
          };
          hmux = pkgs.rustPlatform.buildRustPackage {
            pname = "hmux";
            version = "0.1.0";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.zig_0_16 ]
              ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
                pkgs.xcbuild
                pkgs.darwin.cctools
              ];
            buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
              pkgs.apple-sdk
            ];
            dontUseZigBuild = true;
            dontUseZigCheck = true;
            dontUseZigInstall = true;
            postPatch = ''
              substituteInPlace ghostty-sys/build.rs \
                --replace-fail '"build",' '"build", "--system", "${zigDeps}",'
            '';
            preBuild = ''
              export ZIG_GLOBAL_CACHE_DIR="$TMPDIR/zig-cache"
            '';
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
              pkgs.zig_0_16
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
