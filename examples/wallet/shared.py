"""Shared variables for the wallet examples.
"""

from kaspa import AccountKind

BIP32_KIND = AccountKind("bip32")

FIXED_MNEMONIC_PHRASE = (
    "abandon abandon abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon abandon abandon art"
)
WALLET_SECRET = "example-wallet-secret"
NETWORK_ID = "testnet-10"
