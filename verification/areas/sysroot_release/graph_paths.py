"""Shared producer/consumer authority for explicitly session-owned graph roots."""
from pathlib import Path
import uuid


def graph_path(root, environment, name):
    if name not in {'source-cargo-target','cargo-target'}:
        raise ValueError('unknown sysroot graph kind')
    session=environment.get('SIFR_VERIFY_SYSROOT_GRAPH_SESSION')
    if session is None:
        return root.resolve()/'target/sysroot_release'/name
    if str(uuid.UUID(session)) != session:
        raise ValueError('sysroot graph session must be a canonical UUID')
    return root.resolve()/'target/sysroot_release/graphs'/session/name
