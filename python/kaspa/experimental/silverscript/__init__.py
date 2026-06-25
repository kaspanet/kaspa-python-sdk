"""Python bindings for the SilverScript compiler (experimental).

Compile SilverScript contract source into locking-script bytes and build the
unlocking (signature) scripts that spend them. A separate extension module from
the core ``kaspa`` package; the two interoperate only through script ``bytes``.

This lives under ``kaspa.experimental`` because both SilverScript and these
bindings are under active development and may change in breaking ways between
releases (including the compiler's output). Pin your version, test thoroughly,
and verify any contract end-to-end on a test network before locking real value.
"""

from .silverscript import *  # noqa: F403
