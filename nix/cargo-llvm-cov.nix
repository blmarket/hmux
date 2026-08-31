# nixpkgs pins 0.8.5, which predates support for Cargo's new build-dir layout
# and so finds no object files under this nightly's target directory. 0.8.6
# added `-Zbuild-dir-new-layout` support; 0.9.0 stopped build scripts leaking
# into the report under that layout.
{ lib, rustPlatform, fetchCrate }:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "cargo-llvm-cov";
  version = "0.9.0";

  src = fetchCrate {
    inherit (finalAttrs) pname version;
    hash = "sha256-mQNH7Y+a21Zuppdd8u1PrmTqSY1lFzRmJDFaM+ytyuU=";
  };

  cargoHash = "sha256-JFyBEICPiekTTXlnFw4FioeQyr+EULjbnGlQQ6UCnHc=";

  # The published crate omits the fixtures the test suite needs.
  doCheck = false;

  meta = {
    description = "Cargo subcommand to easily use LLVM source-based code coverage";
    homepage = "https://github.com/taiki-e/cargo-llvm-cov";
    license = with lib.licenses; [ asl20 mit ];
    mainProgram = "cargo-llvm-cov";
  };
})
