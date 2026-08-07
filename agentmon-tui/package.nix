{ lib, python3Packages, git, tmux }:

# Shared by hmux's own flake and by the private superproject's flake, so both
# `nix develop` shells get the same `agentmon` and `looper` binaries from one
# definition.
python3Packages.buildPythonApplication {
  pname = "hmux-agentmon";
  version = "0.1.0";
  pyproject = true;
  src = ./.;

  build-system = with python3Packages; [
    hatchling
  ];
  dependencies = with python3Packages; [
    textual
  ];
  nativeCheckInputs = [
    git
    python3Packages.pytestCheckHook
  ];
  pythonImportsCheck = [ "agentmon" ];
  makeWrapperArgs = [
    "--prefix"
    "PATH"
    ":"
    (lib.makeBinPath [
      git
      tmux
    ])
  ];

  meta.mainProgram = "agentmon";
}
