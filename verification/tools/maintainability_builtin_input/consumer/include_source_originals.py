"""Lossless immutable original storage and authenticated complete decode reuse."""
from dataclasses import dataclass
import hashlib
import weakref
import zlib
import include_source_encoding as canonical

_DECODED={}


@dataclass(frozen=True)
class Original:
    compressed:bytes
    length:int

    @classmethod
    def hold(cls,data):return cls(zlib.compress(data,1),len(data))

    def raw(self):
        data=zlib.decompress(self.compressed)
        if len(data)!=self.length:raise ValueError('incomplete lossless original storage')
        return data

    def digest(self):
        decoder=zlib.decompressobj();sha=hashlib.sha256();length=0
        for start in range(0,len(self.compressed),65536):
            pending=self.compressed[start:start+65536]
            while True:
                chunk=decoder.decompress(pending,1048576);pending=decoder.unconsumed_tail
                length+=len(chunk);sha.update(chunk)
                if length>self.length:raise ValueError('excess lossless original storage')
                if not pending and len(chunk)<1048576:break
        if not decoder.eof or decoder.unused_data or length!=self.length:
            raise ValueError('incomplete/noncanonical lossless original storage')
        return sha.hexdigest()


def _entry(authority):
    value=_DECODED.get(id(authority))
    return value if value is not None and value[0]() is authority else None


def authenticate_decoded(authority):
    value=_entry(authority)
    if value is not None:
        for original,expected in zip(value[1],value[2]):
            if canonical.digest(original)!=expected:raise ValueError('cached complete original decode drift')


def read(authority,decode):
    authenticate_decoded(authority)
    value=_entry(authority)
    if value is not None:return value[1]
    originals=tuple(decode(original.raw()) for original in authority.originals)
    hashes=tuple(canonical.digest(original) for original in originals)
    key=id(authority)
    def discard(reference):
        current=_DECODED.get(key)
        if current is not None and current[0] is reference:del _DECODED[key]
    _DECODED[key]=(weakref.ref(authority,discard),originals,hashes)
    return originals
