"""Python bindings for the SilverScript compiler (experimental).

This lives under ``kaspa.experimental`` because both SilverScript and these
bindings are under active development and may change in breaking ways between
releases (including the compiler's output). Pin your version, test thoroughly,
and verify any contract end-to-end on a test network before locking real value.
"""

try:
    from .silverscript import *  # noqa: F403
except ModuleNotFoundError as exc:
    # The compiled ``silverscript<EXT_SUFFIX>`` extension is gitignored and built
    # separately by ci/build-and-inject-silverscript.sh, which must run before
    # maturin bundles it into the wheel. If that step was skipped (a bare
    # ``maturin develop``/``maturin build``, or an sdist install with no matching
    # prebuilt wheel), the submodule is absent and ``import *`` fails here even
    # though ``import kaspa`` works. Re-raise with an actionable message instead
    # of the opaque "No module named 'kaspa.experimental.silverscript.silverscript'".
    if exc.name and exc.name.startswith(__name__):
        raise ModuleNotFoundError(
            "kaspa.experimental.silverscript native extension is missing. It is "
            "compiled and injected separately by ci/build-and-inject-silverscript.sh "
            "(run before maturin); use ./build-dev / ./build-release, or install a "
            "prebuilt kaspa wheel that bundles it.",
            name=exc.name,
        ) from exc
    raise
