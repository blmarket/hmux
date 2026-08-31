{ lib
, stdenv
, fetchFromGitHub
, makeRustPlatform
, makeWrapper
, rustStable
, llvmPackages
, cmake
, pkg-config
, openssl
, zlib
, tinycbor
}:

let
  rustPlatform = makeRustPlatform {
    cargo = rustStable;
    rustc = rustStable;
  };
  libcDev = stdenv.cc.libc.dev or stdenv.cc.libc;
in
rustPlatform.buildRustPackage rec {
  pname = "c2rust";
  version = "0.22.1";

  src = fetchFromGitHub {
    owner = "immunant";
    repo = "c2rust";
    rev = "v${version}";
    hash = "sha256-fm6XQOxmLno/jJHowzaW0d7uaYTnCt7Ziv9UnZWVRvY=";
  };

  # Upstream's own nix build replaces the tinycbor ExternalProject, which
  # clones from git while cmake runs, with the packaged library.
  patches = [ "${src}/nix-tinycbor-cmake.patch" ];

  cargoHash = "sha256-Qr7vHjzxSrOHNNJBNyDsCBUS3a4uKqhJOVLpjMgRuxc=";

  # The rest of the workspace is the refactoring and analysis tooling, which
  # wants a nightly toolchain and a Python environment of its own.
  cargoBuildFlags = [ "-p" "c2rust" ];

  dontUseCmakeConfigure = true;
  doCheck = false;

  nativeBuildInputs = [
    cmake
    pkg-config
    makeWrapper
    llvmPackages.clang
    llvmPackages.llvm
  ];

  buildInputs = [
    llvmPackages.libclang
    llvmPackages.libllvm
    tinycbor
    openssl
    zlib
  ];

  # c2rust-ast-exporter is a clang plugin: it links against the LLVM and clang
  # C++ libraries and finds them through these variables.
  CMAKE_LLVM_DIR = "${llvmPackages.libllvm.dev}/lib/cmake/llvm";
  CMAKE_CLANG_DIR = "${llvmPackages.libclang.dev}/lib/cmake/clang";
  LLVM_CONFIG_PATH = "${llvmPackages.libllvm.dev}/bin/llvm-config";
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  CLANG_PATH = "${llvmPackages.clang}/bin/clang";
  TINYCBOR_DIR = "${tinycbor}";

  # The clang linked into the transpiler is not the wrapper, so it starts with
  # neither the compiler's own headers nor libc's on its search path and fails
  # on `#include <stddef.h>`. CPATH is a suffix so a caller can still put its
  # own headers first. The `c2rust` driver is left unwrapped: it finds its
  # `c2rust-<subcommand>` siblings by inspecting its own path.
  postFixup = ''
    wrapProgram $out/bin/c2rust-transpile \
      --suffix CPATH : "${llvmPackages.clang}/resource-root/include:${libcDev}/include"
  '';

  meta = {
    description = "Migrate C code to Rust";
    homepage = "https://github.com/immunant/c2rust";
    license = lib.licenses.bsd3;
    mainProgram = "c2rust";
  };
}
