"""Published installation identity and fail-closed custody controls."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import published_installation as installation


class InstallationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.policy = json.loads(installation.POLICY.read_text())

    def test_installer_identity_rejects_wrong_registered_content_size_or_url(self):
        path = self.root/'installer'
        path.write_bytes(b'installer fixture')
        self.policy['installer'].update(size=path.stat().st_size, digest='sha256:'+installation.digest(path))
        installation.installer_identity(self.policy, path)
        for field, value in (('digest','sha256:'+'0'*64), ('size',1), ('browser_download_url','https://wrong.invalid')):
            broken = copy.deepcopy(self.policy)
            broken['installer'][field] = value
            with self.assertRaises(ValueError):
                installation.installer_identity(broken, path)

    def test_installed_inventory_rejects_changed_missing_extra_and_linked_bytes(self):
        binary = self.root/'bin/sifr'
        binary.parent.mkdir()
        binary.write_bytes(b'bin')
        rows = [{'path':'bin/sifr','size_bytes':3,'sha256':installation.digest(binary)},
                {'path':'vendor/dep','size_bytes':3,'sha256':hashlib.sha256(b'dep').hexdigest()}]
        dependency = self.root/'vendor/dep'
        dependency.parent.mkdir()
        dependency.write_bytes(b'dep')
        for name in ('.cargo', 'crates', 'lib'):
            (self.root/name).mkdir()
        for name in ('Cargo.toml', 'Cargo.lock', 'sysroot.toml'):
            path = self.root/name
            path.write_bytes(b'fixture')
            rows.append({'path':name,'size_bytes':7,'sha256':installation.digest(path)})
        self.assertEqual(installation.verify_installed(self.root,rows), len(rows))
        dependency.write_bytes(b'bad')
        with self.assertRaises(ValueError): installation.verify_installed(self.root,rows)
        dependency.unlink()
        with self.assertRaises(ValueError): installation.verify_installed(self.root,rows)
        dependency.write_bytes(b'dep')
        extra = dependency.with_name('extra')
        extra.write_bytes(b'extra')
        with self.assertRaises(ValueError): installation.verify_installed(self.root,rows)
        extra.unlink()
        empty = self.root/'empty'
        empty.mkdir()
        extra.symlink_to(empty, target_is_directory=True)
        with self.assertRaises(ValueError): installation.verify_installed(self.root,rows)
        extra.unlink()
        dependency.unlink()
        dependency.symlink_to(binary)
        with self.assertRaises(ValueError): installation.verify_installed(self.root,rows)

    def test_foreign_host_is_rejected_before_payload_or_output(self):
        output = self.root/'attempt'
        with patch.object(installation,'current_host_target',return_value='aarch64-apple-darwin'), \
             patch.object(installation,'payload') as payload:
            with self.assertRaises(ValueError):
                installation.rehearse(self.policy,archive=self.root/'archive',installer=self.root/'installer',
                    target='x86_64-unknown-linux-gnu',candidate_version='0.1.0-beta.1300',output=output)
        payload.assert_not_called()
        self.assertFalse(output.exists())

    def test_failed_installer_preserves_logs_receipt_and_zero_runtime_assertions(self):
        archive, installer = self.root/'archive', self.root/'installer'
        archive.write_bytes(b'archive');installer.write_bytes(b'installer')
        target = 'x86_64-unknown-linux-gnu'
        asset = installation.select(self.policy,target,'0.1.0-beta.1300')
        output = self.root/'attempt'
        failed = SimpleNamespace(cause='exit',returncode=71,truncated=False,elapsed_seconds=.1,stdout=b'',stderr=b'failed')
        with patch.object(installation,'current_host_target',return_value=target), \
             patch.object(installation,'payload',return_value=(asset,[])), \
             patch.object(installation,'installer_identity',return_value={'path':str(installer),'sha256':installation.digest(installer)}), \
             patch.object(installation,'discover',return_value=object()), \
             patch.object(installation,'admit',return_value={}), \
             patch.object(installation,'execute',return_value=failed):
            with self.assertRaises(ValueError):
                installation.rehearse(self.policy,archive=archive,installer=installer,target=target,
                    candidate_version='0.1.0-beta.1300',output=output)
        receipt = json.loads((output/'receipt.json').read_text())
        self.assertEqual(receipt['status'],'failed')
        self.assertEqual(receipt['runtime_assertions'],0)
        self.assertEqual((output/'install.stderr').read_bytes(),b'failed')
        self.assertEqual(receipt['commands'][0]['returncode'],71)


if __name__ == '__main__': unittest.main()
