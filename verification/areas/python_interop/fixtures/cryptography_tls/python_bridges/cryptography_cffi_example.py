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
    if not certifi.where():
        raise RuntimeError("certifi returned an empty certificate path")
    return "sifr-python-interop:cryptography-cffi:roundtrip=sifr-secret:certifi=ok"
