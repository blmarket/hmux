The next refactoring I'd like to check is to find `extern "C"` which can be
safely removed.

First check safe to remove vs would break compile, and check why they would
break compile.

Create ./report-externc.md with suggestions.
