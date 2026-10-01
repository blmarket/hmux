# hmux-agent

Agent detection and pane classification imported from the sibling
`hmux/hmux-agent` crate. The detector sources and their tests are retained
unchanged. Keeping this crate in the hmux2 workspace allows standalone Cargo
and Nix builds without depending on the other daemon's source tree.

The hmux2 agent plugin implements `observability::v1` over weak pane
observations. This crate has no dependency on either server implementation.
