{ lib, tmux, fetchFromGitHub }:

let
  revision = "e880cf63e0a9fe095d7c5d313761520fb1a8653c";
in
tmux.overrideAttrs (old: {
  version = "next-3.9";
  src = fetchFromGitHub {
    owner = "tmux";
    repo = "tmux";
    rev = revision;
    hash = "sha256-ODpffQUf7obGWS7Cl/4KxNJGkvKNI9w5+/Jf2vNXJEU=";
  };
  configureFlags =
    (lib.filter (flag: flag != "--enable-sixel") old.configureFlags)
    ++ [ "--disable-sixel" "--disable-debug" ];
  outputs = (old.outputs or [ "out" ]) ++ [ "source" ];
  postPatch = (old.postPatch or "") + ''
    mkdir -p "$source"
    tar -cf "$source/tmux.tar" .
  '';
  postInstall = (old.postInstall or "") + ''
    mkdir -p "$out/share/tmux"
    printf '%s\n' '${revision}' > "$out/share/tmux/hmux-upstream-revision"
    printf '%s\n' "$source/tmux.tar" > "$out/share/tmux/source"
  '';
})
