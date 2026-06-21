#!/usr/bin/env bash
# Build the kaspa.silverscript abi3 extension module and inject the resulting
# shared library into python/kaspa/silverscript/ so the subsequent `maturin`
# build bundles it into the kaspa wheel (under kaspa/silverscript/).
#
# It's a SEPARATE workspace member that links a different rusty-kaspa revision
# than the core, so it must be compiled on its own and dropped into the
# python-source tree. abi3 means one .so serves every supported Python version.
#
# Usage: ci/build-and-inject-silverscript.sh [debug|release]
# Set SILVERSCRIPT_TARGET to a Rust target triple to cross-compile (e.g. the
# macOS x86_64 wheel built on an arm64 runner).
set -euo pipefail

profile="${1:-debug}"
flag=""
[ "$profile" = "release" ] && flag="--release"

target_flag=""
target_subdir=""
if [ -n "${SILVERSCRIPT_TARGET:-}" ]; then
    target_flag="--target $SILVERSCRIPT_TARGET"
    target_subdir="$SILVERSCRIPT_TARGET/"
fi

case "$(uname -s)" in
    Darwin)
        # extension-module cdylibs resolve Python symbols from the host at load
        # time; tell the linker to allow the resulting undefined symbols.
        export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup ${RUSTFLAGS:-}"
        libname="libsilverscript.dylib"
        out="silverscript.abi3.so"
        ;;
    Linux)
        libname="libsilverscript.so"
        out="silverscript.abi3.so"
        ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        libname="silverscript.dll"
        out="silverscript.pyd"
        ;;
    *)
        echo "unsupported OS: $(uname -s)" >&2
        exit 1
        ;;
esac

# Enable the cdylib-only features explicitly here (not as crate defaults) so
# they never unify into the core crate during a workspace build.
cargo build -p kaspa-silverscript --lib --features extension-module,abi3 $flag $target_flag --target-dir target
lib="target/${target_subdir}${profile}/${libname}"

mkdir -p python/kaspa/silverscript
# Drop any previously-injected library so stale per-version copies don't shadow
# the abi3 one (extension suffixes are matched in a fixed priority order).
rm -f python/kaspa/silverscript/silverscript*.so python/kaspa/silverscript/silverscript*.pyd
cp "$lib" "python/kaspa/silverscript/$out"
echo "injected python/kaspa/silverscript/$out (from $lib)"
