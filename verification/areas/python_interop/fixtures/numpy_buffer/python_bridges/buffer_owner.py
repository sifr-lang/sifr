import mmap


def make_owner(payload: bytes) -> mmap.mmap:
    owner = mmap.mmap(-1, len(payload))
    owner.write(payload)
    owner.seek(0)
    return owner
