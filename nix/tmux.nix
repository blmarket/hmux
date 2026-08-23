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
  # only what is below. Most of it is crash fixes: a crashed oracle answers
  # nothing, so a conformance test that trips one tells us about tmux, not
  # about hmux. The other reason a patch may stand here is that hmux has
  # already stopped doing the thing and the fix has gone upstream, so that
  # oracle and hmux agree and there is no lasting difference to describe.
  # Nothing wider than those two belongs here: a patch that changes what the
  # oracle *answers* would hide a real difference rather than settle one.
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
  #
  # 0004, submitted upstream as 68d54cfb, not a crash fix: a pane's
  # `border_status_line.expanded` holds the last expansion of
  # `pane-border-format`, and the pane was freed without it -- in 3.7b, and
  # still on master, where the drawing has moved to `window-border.c` and the
  # teardown has split in two. `status.c` frees the client's own five copies of
  # the same struct, so only the pane's was missed. `pane-border-format` is the
  # user's to set, so each leak is as large as the user makes it. hmux frees
  # it, and this keeps the oracle from being the only one of the two that does
  # not; `tmux-c2rs/demo-expanded-mem.sh` measures either binary.
  #
  # 0005, submitted upstream as 9261bcd5, not a crash fix: `server_client_lost`
  # frees every other string a client owns -- `title` among them -- and not
  # `c->path`, which is freed only where `server_client_set_path` replaces it,
  # so the last one a client held goes with it. The string is the active pane's
  # OSC 7 path, which whatever runs in the pane sets and `input-buffer-size`
  # lets reach a megabyte, so a program that writes to a terminal decides how
  # much each lost client costs. `tmux-c2rs/demo-client-path-mem.sh` measures
  # either binary.
  patches = [
    ./tmux-3.7b-0001-do-not-crash-looking-for-next-or-previous-session.patch
    ./tmux-3.7b-0002-detach-clients-when-processing-destroy-unattached.patch
    ./tmux-3.7b-0003-do-not-crash-on-empty-custom-layout-slot.patch
    ./tmux-3.7b-0004-free-pane-border-status-string-on-destroy.patch
    ./tmux-3.7b-0005-free-client-path-when-losing-client.patch
  ];
})
