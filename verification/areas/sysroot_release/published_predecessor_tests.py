"""Published predecessor bytes, version order and extraction boundary controls."""
import copy
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import published_predecessor as predecessor


class PublishedTests(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads(predecessor.POLICY.read_text())
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.capacity = patch.object(predecessor.shutil, 'disk_usage', return_value=SimpleNamespace(free=20*1024**3))
        self.capacity.start()
        self.addCleanup(self.capacity.stop)

    def test_registered_native_inventory_and_real_predecessor_order(self):
        asset = predecessor.select(self.policy, 'x86_64-unknown-linux-gnu', '0.1.0-beta.1300')
        self.assertEqual(asset['digest'], 'sha256:8d796c321cc2b5a898c5e751c6769154b068060a69ae3aa302051e4fa9bb5448')
        for version in ('0.1.0-beta.15', '0.1.0-beta.16'):
            with self.assertRaises(ValueError):
                predecessor.select(self.policy, 'x86_64-unknown-linux-gnu', version)
        broken = copy.deepcopy(self.policy)
        broken['assets'].pop()
        with self.assertRaises(ValueError):
            predecessor.select(broken, 'aarch64-apple-darwin', '0.1.0-beta.1300')

    def test_exact_download_hash_size_and_bound(self):
        raw = b'actual published bytes fixture'
        asset = {'browser_download_url': 'unused', 'size':len(raw), 'digest':'sha256:'+hashlib.sha256(raw).hexdigest()}
        output = self.root/'verified'
        predecessor.fetch_archive(asset, output, opener=lambda *_args, **_kwargs:io.BytesIO(raw))
        self.assertEqual(output.read_bytes(),raw)
        for payload in (raw+b'extra',raw[:-1],b'x'*len(raw)):
            with self.subTest(payload=payload), self.assertRaises(ValueError):
                predecessor.fetch_archive(asset,self.root/str(len(list(self.root.iterdir()))),
                                          opener=lambda *_args, **_kwargs:io.BytesIO(payload))

    def archive(self, escaping=False):
        path=self.root/'package.tar.gz'
        with tarfile.open(path,'w:gz') as archive:
            member=tarfile.TarInfo('../outside' if escaping else 'bin/sifr')
            member.size=3
            archive.addfile(member,io.BytesIO(b'bin'))
        return path

    def test_verified_inventory_and_escaping_paths(self):
        archive=self.archive()
        report=predecessor.unpack(archive,self.root/'valid')
        self.assertEqual(report['decoded_bytes'],3)
        self.assertEqual(report['files'][0]['path'],'bin/sifr')
        archive=self.archive(escaping=True)
        with self.assertRaises(tarfile.FilterError):
            predecessor.unpack(archive,self.root/'escape')
        self.assertFalse((self.root/'outside').exists())

    def test_reserve_refusal_does_not_contact_network(self):
        asset=predecessor.select(self.policy,'x86_64-unknown-linux-gnu','0.1.0-beta.1300')
        with patch.object(predecessor.shutil,'disk_usage',return_value=SimpleNamespace(free=predecessor.RESERVE_BYTES)), \
                patch.object(predecessor.urllib.request,'urlopen') as network:
            with self.assertRaises(OSError):
                predecessor.fetch_archive(asset,self.root/'absent')
            network.assert_not_called()

    def test_failed_transfer_retains_failed_receipt_not_qualification(self):
        output=self.root/'attempt'
        with patch.object(predecessor,'fetch_archive',side_effect=ValueError('hash drift')):
            with self.assertRaises(ValueError):
                predecessor.prepare(policy=self.policy,target='x86_64-unknown-linux-gnu',
                                    candidate_version='0.1.0-beta.1300',output=output)
        receipt=json.loads((output/'receipt.json').read_text())
        self.assertEqual(receipt['status'],'failed')
        self.assertEqual(receipt['kind'],'preparation-output')
        self.assertEqual(receipt['runtime_assertions'],0)
        self.assertIn('hash drift',receipt['failure'])


if __name__ == '__main__':
    unittest.main()
