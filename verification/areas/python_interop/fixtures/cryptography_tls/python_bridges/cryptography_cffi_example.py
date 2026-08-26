import ssl

import certifi
import cffi
from cryptography.fernet import Fernet


def run() -> str:
    key = Fernet.generate_key()
    cipher = Fernet(key)
    if cipher.decrypt(cipher.encrypt(b"sifr-secret")) != b"sifr-secret":
        raise RuntimeError("Fernet round trip failed")
    ffi = cffi.FFI()
    ffi.cdef("int add(int, int);")
    trust_store = ssl.create_default_context(cafile=certifi.where())
    if trust_store.cert_store_stats()["x509_ca"] <= 100:
        raise RuntimeError("certifi did not populate the platform trust store")
    return (
        "sifr-python-interop:cryptography-cffi:roundtrip=sifr-secret:certifi=ca-store"
    )
