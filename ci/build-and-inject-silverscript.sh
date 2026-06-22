#!/usr/bin/env bash
# Build the kaspa.silverscript extension module and inject the resulting shared
# library into python/kaspa/silverscript/ so the subsequent `maturin` build
# bundles it into the kaspa wheel (under kaspa/silverscript/).
#
# It's a SEPARATE workspace member that links a different rusty-kaspa revision
# than the core, so it must be compiled on its own and dropped into the
# python-source tree.
#
# The module is NOT abi3 (it subclasses PyException via #[pyclass], which the
# limited API can't do), so it must be built once per CPython version. The
# output is named with the interpreter's EXT_SUFFIX (e.g.
# silverscript.cpython-312-x86_64-linux-gnu.so) so only the matching interpreter
# imports it. Run this once per interpreter, each time immediately before the
# `maturin build --interpreter <that version>` that should bundle it.
#
# Usage: ci/build-and-inject-silverscript.sh [debug|release]
# Env (interpreter selection, in priority order):
#   SILVERSCRIPT_PYTHON      explicit interpreter path; wins if set.
#   SILVERSCRIPT_PY_VERSION  version like "3.12". When building inside a
#                            manylinux container the matching
#                            /opt/python/cp312-cp312/bin/python is used; on a
#                            native runner (no /opt/python) it falls back to
#                            python3 (which setup-python pins to that version).
#   (neither)                python3 on PATH.
#   SILVERSCRIPT_TARGET      Rust target triple to cross-compile (e.g. the macOS
#                            x86_64 wheel built on an arm64 runner).
set -euo pipefail

profile="${1:-debug}"
flag=""
[ "$profile" = "release" ] && flag="--release"

# Resolve the interpreter to build for. An explicit SILVERSCRIPT_PYTHON wins.
# Otherwise, if SILVERSCRIPT_PY_VERSION is given and the matching manylinux
# interpreter exists under /opt/python (release builds run in that container),
# use it; else fall back to python3 (native runners use setup-python's python3).
pybin="${SILVERSCRIPT_PYTHON:-}"
if [ -z "$pybin" ] && [ -n "${SILVERSCRIPT_PY_VERSION:-}" ]; then
    tag="cp${SILVERSCRIPT_PY_VERSION//./}"
    candidate="/opt/python/${tag}-${tag}/bin/python"
    [ -x "$candidate" ] && pybin="$candidate"
fi
if [ -z "$pybin" ]; then
    # Native runners: setup-python pins python3 (Windows bash may only have python).
    if command -v python3 >/dev/null 2>&1; then
        pybin="python3"
    elif command -v python >/dev/null 2>&1; then
        pybin="python"
    else
        echo "no python interpreter found (set SILVERSCRIPT_PYTHON)" >&2
        exit 1
    fi
fi
# Build pyo3 against this specific interpreter (no abi3 → version-specific).
export PYO3_PYTHON="$pybin"
ext_suffix="$("$pybin" -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"
py_version="$("$pybin" -c 'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")')"

target_flag=""
target_subdir=""
if [ -n "${SILVERSCRIPT_TARGET:-}" ]; then
    target_flag="--target $SILVERSCRIPT_TARGET"
    target_subdir="$SILVERSCRIPT_TARGET/"
    # Cross-compiling: pyo3 can't run the target interpreter, so tell it the
    # version explicitly. On macOS the extension links no libpython (symbols
    # resolve at load time via dynamic_lookup, below), so only the version matters.
    export PYO3_CROSS_PYTHON_VERSION="$py_version"
fi

case "$(uname -s)" in
    Darwin)
        # extension-module cdylibs resolve Python symbols from the host at load
        # time; tell the linker to allow the resulting undefined symbols.
        export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup ${RUSTFLAGS:-}"
        libname="libsilverscript.dylib"
        ;;
    Linux)
        libname="libsilverscript.so"
        ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        libname="silverscript.dll"
        ;;
    *)
        echo "unsupported OS: $(uname -s)" >&2
        exit 1
        ;;
esac

# Name the injected module with the interpreter's extension suffix so Python's
# import machinery loads it only on the matching version.
out="silverscript${ext_suffix}"

# Enable the cdylib-only feature explicitly here (not as a crate default) so it
# never unifies into the core crate during a workspace build.
cargo build -p kaspa-python-sdk-silverscript --lib --features extension-module $flag $target_flag --target-dir target
lib="target/${target_subdir}${profile}/${libname}"

mkdir -p python/kaspa/silverscript
# Drop any previously-injected library so only the version we're about to build
# for is present — each maturin call should bundle exactly one silverscript .so.
rm -f python/kaspa/silverscript/silverscript*.so python/kaspa/silverscript/silverscript*.pyd
cp "$lib" "python/kaspa/silverscript/$out"
echo "injected python/kaspa/silverscript/$out (py $py_version, from $lib)"
