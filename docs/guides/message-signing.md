# Sign and verify a message

Sign arbitrary bytes with a `PrivateKey` and verify with the
corresponding `PublicKey`. Useful for proving address ownership,
authenticating off-chain actions, or stamping structured payloads.

Read [Security](../getting-started/security.md) before signing with real
keys.

## Sign

```python
from kaspa import PrivateKey, sign_message

key = PrivateKey("<64-char hex>")
signature = sign_message("Hello, I own this address!", key)
```

For deterministic signatures (same message + same key produces the same
signature):

```python
signature = sign_message(message, key, no_aux_rand=True)
```

The default uses fresh auxiliary randomness; that's the right choice
unless a downstream consumer needs determinism.

## Verify

```python
from kaspa import PublicKey, verify_message

pub = PublicKey("02a1b2c3...")
# or: pub = key.to_public_key()

ok = verify_message(message, signature, pub)
```

`verify_message` returns `bool`. It does not raise on a mismatch.

## Recipe: prove address ownership

```python
import time
from kaspa import sign_message, verify_message

def prove_ownership(key, address):
    timestamp = int(time.time())
    message = f"I own {address.to_string()} at {timestamp}"
    return {
        "address": address.to_string(),
        "timestamp": timestamp,
        "message": message,
        "signature": sign_message(message, key),
    }

def verify_ownership(proof, pub):
    return verify_message(proof["message"], proof["signature"], pub)
```

Include a timestamp so the proof can't be replayed indefinitely; the
verifier should reject anything older than its acceptable window.

## Recipe: signed JSON payload

When the thing you're signing is structured, canonicalise first:

```python
import json
from kaspa import sign_message, verify_message

def sign_json(data, key):
    canonical = json.dumps(data, sort_keys=True, separators=(",", ":"))
    return {"data": data, "signature": sign_message(canonical, key)}

def verify_json(envelope, pub):
    canonical = json.dumps(envelope["data"], sort_keys=True, separators=(",", ":"))
    return verify_message(canonical, envelope["signature"], pub)
```

The canonicalisation matters: any non-deterministic serialisation
(default `json.dumps`, key reordering, whitespace) will produce a
signature that won't verify.

## Recipe: time-limited auth token

```python
import time
from kaspa import sign_message, verify_message

def create_token(key, address, ttl=300):
    expires = int(time.time()) + ttl
    message = f"auth:{address.to_string()}:{expires}"
    return {
        "address": address.to_string(),
        "expires": expires,
        "signature": sign_message(message, key),
    }

def verify_token(token, pub):
    if int(time.time()) > token["expires"]:
        return False, "expired"
    message = f"auth:{token['address']}:{token['expires']}"
    return verify_message(message, token["signature"], pub), "ok"
```

The signature covers the expiry, so a token can't be forged with a later
expiry without re-signing. Use it as a bearer token in HTTP headers, or
embed it in a session payload.
