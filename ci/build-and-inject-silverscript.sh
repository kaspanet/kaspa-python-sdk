#!/usr/bin/env bash
# Build the kaspa.experimental.silverscript extension and inject its shared
# library into python/kaspa/experimental/silverscript/ for the next `maturin`
# build to bundle into the kaspa wheel.
#
# It's a separate workspace member on a different rusty-kaspa revision, and not
# abi3. So, it's built once per CPython version and
# named with the interpreter's EXT_SUFFIX, so only that version imports it.
#
# Output: python/kaspa/experimental/silverscript/silverscript<EXT_SUFFIX>
#
# Env (interpreter selection, in priority order):
#   SILVERSCRIPT_PYTHON      explicit interpreter path.
#   SILVERSCRIPT_PY_VERSION  version like "3.12"; finds the manylinux
#                            /opt/python/cp3XX or python3.X on PATH.
#   (neither)                python3 on PATH.
#   SILVERSCRIPT_TARGET      Rust target triple to cross-compile.
set -euo pipefail

profile="${1:-debug}"
flag=""
[ "$profile" = "release" ] && flag="--release"

# Resolve the interpreter (priority: SILVERSCRIPT_PYTHON, SILVERSCRIPT_PY_VERSION,
# then python3/python on PATH). Must match the wheel's CPython version — not abi3.
req_ver="${SILVERSCRIPT_PY_VERSION:-}"
pybin="${SILVERSCRIPT_PYTHON:-}"
if [ -z "$pybin" ] && [ -n "$req_ver" ]; then
    tag="cp${req_ver//./}"
    for candidate in "/opt/python/${tag}-${tag}/bin/python" "python${req_ver}"; do
        if command -v "$candidate" >/dev/null 2>&1; then
            pybin="$candidate"
            break
        fi
    done
fi
if [ -z "$pybin" ]; then
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
py_version="$("$pybin" -c 'import sys; print(f"{sys.version_info[0]}.{sys.version_info[1]}")')"
# If a version was requested, the resolved interpreter must match it.
if [ -n "$req_ver" ] && [ "$py_version" != "$req_ver" ]; then
    echo "requested Python $req_ver but '$pybin' is $py_version" >&2
    echo "set SILVERSCRIPT_PYTHON to the matching interpreter" >&2
    exit 1
fi
ext_suffix="$("$pybin" -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))')"

target_flag=""
target_subdir=""
if [ -n "${SILVERSCRIPT_TARGET:-}" ]; then
    target_flag="--target $SILVERSCRIPT_TARGET"
    target_subdir="$SILVERSCRIPT_TARGET/"
    # Cross-compiling: pyo3 can't run the target interpreter, so pass the version.
    export PYO3_CROSS_PYTHON_VERSION="$py_version"
fi

case "$(uname -s)" in
    Darwin)
        # Let the linker leave Python symbols undefined; resolved at load time.
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

# Name the module with the interpreter's EXT_SUFFIX so only that version imports it.
out="silverscript${ext_suffix}"

# Enable the cdylib-only feature explicitly here (not as a crate default) so it
# never unifies into the core crate during a workspace build.
cargo build -p kaspa-python-sdk-silverscript --lib --features extension-module $flag $target_flag --target-dir target
lib="target/${target_subdir}${profile}/${libname}"

mkdir -p python/kaspa/experimental/silverscript
# Drop any previously-injected library; each maturin call bundles exactly one.
rm -f python/kaspa/experimental/silverscript/silverscript*.so python/kaspa/experimental/silverscript/silverscript*.pyd
cp "$lib" "python/kaspa/experimental/silverscript/$out"
echo "injected python/kaspa/experimental/silverscript/$out (py $py_version, from $lib)"
