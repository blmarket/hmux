/*
 * Generates hmux/src/vt/width/table.rs.
 *
 * The width oracle is the pinned tmux 3.7b, which links libutf8proc and
 * resolves a codepoint's width through its own compat wrapper in
 * tmux's compat/utf8proc.c:
 *
 *     int utf8proc_wcwidth(wchar_t wc) {
 *             if (utf8proc_category(wc) == UTF8PROC_CATEGORY_CO)
 *                     return (1);
 *             return (utf8proc_charwidth(wc));
 *     }
 *
 * Private-use codepoints (category Co) are where powerline glyphs live; they
 * are "ambiguous" width and tmux calls them 1. Everything else is
 * utf8proc_charwidth.
 *
 * This program reproduces that function for every codepoint and emits the
 * result as a range-compressed Rust table, so the default hmux build needs no
 * C or Zig toolchain while still agreeing with the oracle codepoint for
 * codepoint.
 *
 * Regenerate with (adjust the utf8proc prefix to match the tmux being pinned):
 *
 *     UTF8PROC=$(ldd "$(command -v tmux)" | awk '/libutf8proc/ {print $3}')
 *     PREFIX=${UTF8PROC%/lib/*}
 *     cc -O2 -I"$PREFIX/include" hmux/scripts/gen-width-table.c \
 *         -L"$PREFIX/lib" -lutf8proc -o /tmp/gen-width-table
 *     /tmp/gen-width-table > hmux/src/vt/width/table.rs
 *
 * The generated file records the utf8proc version it came from; if the pinned
 * tmux moves to a different utf8proc, regenerate and expect the conformance
 * suite to be the judge of whether anything shifted.
 */

#include <stdio.h>
#include <utf8proc.h>

#define MAX_CODEPOINT 0x10FFFF

static int
tmux_width(utf8proc_int32_t cp)
{
	if (utf8proc_category(cp) == UTF8PROC_CATEGORY_CO)
		return 1;
	return utf8proc_charwidth(cp);
}

int
main(void)
{
	utf8proc_int32_t	cp, start;
	int			width, prev, ranges = 0;

	printf("//! Character widths, generated from utf8proc %s.\n",
	    utf8proc_version());
	printf("//!\n");
	printf("//! Do not edit by hand: regenerate with\n");
	printf("//! `hmux/scripts/gen-width-table.c`, whose header comment\n");
	printf("//! explains why this table exists and how it is built.\n");
	printf("\n");
	printf("/// The utf8proc release this table was generated from. The\n");
	printf("/// pinned tmux links the same one, which is what makes the\n");
	printf("/// widths agree. Only the test that pins the oracle reads it.\n");
	printf("#[cfg(test)]\n");
	printf("pub(super) const UTF8PROC_VERSION: &str = \"%s\";\n",
	    utf8proc_version());
	printf("\n");
	printf("/// Half-open-free width ranges, sorted and non-overlapping:\n");
	printf("/// every codepoint in `start..=end` has width `width`.\n");
	printf("/// Codepoints wider than the last range's end are not covered\n");
	printf("/// here; see the lookup for what happens to them.\n");
	printf("pub(super) static WIDTHS: &[(u32, u32, u8)] = &[\n");

	start = 0;
	prev = tmux_width(0);
	for (cp = 1; cp <= MAX_CODEPOINT; cp++) {
		width = tmux_width(cp);
		if (width == prev)
			continue;
		printf("    (0x%05X, 0x%05X, %d),\n", start, cp - 1, prev);
		ranges++;
		start = cp;
		prev = width;
	}
	printf("    (0x%05X, 0x%05X, %d),\n", start, MAX_CODEPOINT, prev);
	ranges++;

	printf("];\n");
	fprintf(stderr, "%d ranges\n", ranges);
	return 0;
}
