Execute one minimal improvement listed in the below, then make `git commit`

1. Identify E2E conformance test which can increase test coverage.
  - E2E tests are more useful as they communicate with tmux server protocol -
    they can test real world scenario. Increasing test coverage is useful.
2. Reduce raw pointer usage in the codebase.
  - Some code can be migrated to owned types with no behavioral change,
    eventually it can help avoid memory safety issue.
3. Reduce use of `unsafe` keyword
  - After making migration on item 2, sometimes we can remove `unsafe` for
    free. It helps identify what is remaining unsafe and focus.
4. Narrow untyped pointer
  - Hard to reason about void* types. Need to analyze writes + read castings
    and check we can narrow it down to specific type or union of known types.
  - To minimize diff size, change only the data type without parameter name. It
    also helps to trace relationship with tmux code.
5. Simplify inline struct init
  - Check struct::default() can be used instead. If some of the fields are
    initialized, then { field1: value1, ...Default() } style is allowed.
