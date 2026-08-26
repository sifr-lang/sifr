import ssl
from datetime import datetime, timezone

import certifi
import cffi
from cryptography import x509
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.x509.oid import ExtendedKeyUsageOID, NameOID
from cryptography.x509.verification import (
    DNSName,
    PolicyBuilder,
    Store,
    VerificationError,
)

_VERIFICATION_TIME = datetime(2026, 8, 26, tzinfo=timezone.utc)


def _build_certificate_path() -> tuple[x509.Certificate, x509.Certificate]:
    root_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    root_name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "Sifr Test Root")])
    root = (
        x509.CertificateBuilder()
        .subject_name(root_name)
        .issuer_name(root_name)
        .public_key(root_key.public_key())
        .serial_number(1)
        .not_valid_before(datetime(2026, 1, 1, tzinfo=timezone.utc))
        .not_valid_after(datetime(2027, 1, 1, tzinfo=timezone.utc))
        .add_extension(x509.BasicConstraints(ca=True, path_length=0), critical=True)
        .add_extension(
            x509.KeyUsage(
                digital_signature=True,
                content_commitment=False,
                key_encipherment=False,
                data_encipherment=False,
                key_agreement=False,
                key_cert_sign=True,
                crl_sign=True,
                encipher_only=None,
                decipher_only=None,
            ),
            critical=True,
        )
        .add_extension(
            x509.SubjectKeyIdentifier.from_public_key(root_key.public_key()),
            critical=False,
        )
        .sign(root_key, hashes.SHA256())
    )

    leaf_key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    leaf_name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, "sifr.test")])
    leaf = (
        x509.CertificateBuilder()
        .subject_name(leaf_name)
        .issuer_name(root_name)
        .public_key(leaf_key.public_key())
        .serial_number(2)
        .not_valid_before(datetime(2026, 1, 1, tzinfo=timezone.utc))
        .not_valid_after(datetime(2027, 1, 1, tzinfo=timezone.utc))
        .add_extension(x509.BasicConstraints(ca=False, path_length=None), critical=True)
        .add_extension(
            x509.KeyUsage(
                digital_signature=True,
                content_commitment=False,
                key_encipherment=True,
                data_encipherment=False,
                key_agreement=False,
                key_cert_sign=False,
                crl_sign=False,
                encipher_only=None,
                decipher_only=None,
            ),
            critical=True,
        )
        .add_extension(
            x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), critical=False
        )
        .add_extension(
            x509.SubjectAlternativeName([x509.DNSName("sifr.test")]), critical=False
        )
        .add_extension(
            x509.SubjectKeyIdentifier.from_public_key(leaf_key.public_key()),
            critical=False,
        )
        .add_extension(
            x509.AuthorityKeyIdentifier.from_issuer_public_key(root_key.public_key()),
            critical=False,
        )
        .sign(root_key, hashes.SHA256())
    )
    return root, leaf


def _verify_certificate_path() -> None:
    root, leaf = _build_certificate_path()
    policy = PolicyBuilder().store(Store([root])).time(_VERIFICATION_TIME)
    chain = policy.build_server_verifier(DNSName("sifr.test")).verify(leaf, [])
    if chain != [leaf, root]:
        raise RuntimeError("Cryptography returned an unexpected certificate path")
    try:
        policy.build_server_verifier(DNSName("wrong.test")).verify(leaf, [])
    except VerificationError:
        return
    raise RuntimeError("Cryptography accepted a certificate for the wrong hostname")


def run() -> str:
    key = Fernet.generate_key()
    cipher = Fernet(key)
    if cipher.decrypt(cipher.encrypt(b"sifr-secret")) != b"sifr-secret":
        raise RuntimeError("Fernet round trip failed")
    _verify_certificate_path()
    ffi = cffi.FFI()
    ffi.cdef("int add(int, int);")
    trust_store = ssl.create_default_context(cafile=certifi.where())
    if trust_store.cert_store_stats()["x509_ca"] == 0:
        raise RuntimeError("certifi did not populate the platform trust store")
    return (
        "sifr-python-interop:cryptography-cffi:roundtrip=sifr-secret:certifi=ca-store"
        ":x509=verified:wrong-host=rejected"
    )
