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
  # only the crash fixes below. A crashed oracle answers nothing, so a
  # conformance test that trips one of these tells us about tmux, not about
  # hmux; every patch here has to stay this narrow.
  #
  # 0001, tmux c515d8ca ("Do not crash looking for next or previous session.
  # GitHub issue 5344.", released in 3.7c): `server_destroy_session()` passed a
  # NULL `struct sort_criteria *` to `session_{next,previous}_session()`, which
  # `sort_qsort()` dereferences. In 3.7b, `set -g detach-on-destroy next` (or
  # `previous`) followed by `kill-session` kills the whole server.
  #
  # 0002, local fix, not upstream: `server_check_unattached()` called
  # `session_destroy()` directly, so a client that is already exiting - not
  # counted in `s->attached`, but still holding `c->session` - was left with a
  # dangling session pointer. Calling `server_destroy_session()` first clears
  # those references. In 3.7b, `set -g destroy-unattached on` can kill the
  # server as a client detaches.
  #
  # 0003, tmux 97472e37 ("Return early if cannot construct cell.", post-3.7b):
  # `layout_construct()` inserted the child cell without checking that
  # `layout_construct_cell()` returned one, and the empty-slot cases
  # (`,`/`}`/`]`/`>`/NUL) return success, so a custom layout with an empty
  # slot - `select-layout` on `<csum>,80x24,0,0{,40x24,0,0}` and friends -
  # linked a NULL cell and dereferenced it. In 3.7b this kills the server from
  # a single client command. The transpilation reproduces the 3.7b crash
  # faithfully; the matching Rust guard lives in tmux-c2rs.
  patches = [
    ./tmux-3.7b-0001-do-not-crash-looking-for-next-or-previous-session.patch
    ./tmux-3.7b-0002-detach-clients-when-processing-destroy-unattached.patch
    ./tmux-3.7b-0003-do-not-crash-on-empty-custom-layout-slot.patch
  ];
})
