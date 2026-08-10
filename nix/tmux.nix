# The tmux the conformance suite runs against.
#
# Every flake in this tree pins the oracle through this one file: a build
# difference between two of them is a difference in every conformance result
# they produce, so there is only one build to differ from.
{ lib, tmux, fetchFromGitHub }:

tmux.overrideAttrs (old: {
  version = "3.7b";
  src = fetchFromGitHub {
    owner = "tmux";
    repo = "tmux";
    tag = "3.7b";
    hash = "sha256-CTq06XP997M0ODxQihTq34dI9H6jSRLUXLYuTWOwDpc=";
  };

  # hmux does not implement sixel, so the oracle must not either. `ENABLE_SIXEL`
  # changes the primary DA answer, the XTSMGRAPHICS reply, `#{sixel_support}`,
  # and whether a `DCS q` payload reaches the grid at all.
  configureFlags =
    (lib.filter (flag: flag != "--enable-sixel") old.configureFlags)
    ++ [ "--disable-sixel" ];

  # Drop the patches nixpkgs' tmux carries; the oracle must be stock 3.7b plus
  # only the upstream crash fix below.
  #
  # tmux c515d8ca ("Do not crash looking for next or previous session. GitHub
  # issue 5344.", released in 3.7c): `server_destroy_session()` passed a NULL
  # `struct sort_criteria *` to `session_{next,previous}_session()`, which
  # `sort_qsort()` dereferences. In 3.7b, `set -g detach-on-destroy next` (or
  # `previous`) followed by `kill-session` kills the whole server.
  patches = [ ./tmux-3.7b-0001-do-not-crash-looking-for-next-or-previous-session.patch ];
})
