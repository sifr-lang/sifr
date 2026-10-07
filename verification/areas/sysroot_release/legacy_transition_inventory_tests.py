"""Synthetic residue inventory controls, never native qualification evidence."""
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import legacy_transition_inventory as inventory
from published_predecessor import digest


class LegacyInventory(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
        self.managed=Path(self.tmp.name);self.generations=self.managed/'.sifr-generations';self.generations.mkdir()
        self.rollback=self.generations/'legacy.failed';self.rollback.mkdir();(self.rollback/'bin').mkdir()
        (self.rollback/'install.json').write_text('published receipt')
        self.sha=digest(self.rollback/'install.json')
        self.complete=self.generations/'legacy.complete';self.complete.mkdir();(self.complete/'install.json').write_text('published receipt')

    def check(self):
        return inventory.predecessor(self.managed,self.rollback,self.sha,[])

    def test_exact_residue_and_one_intact_payload_use_independent_byte_verifier(self):
        with patch.object(inventory,'verify_installed') as verify:
            self.assertEqual(self.check(),self.complete);verify.assert_called_once_with(self.complete,[])

    def test_empty_or_additional_predecessors_never_qualify(self):
        (self.generations/'legacy.extra').mkdir()
        with self.assertRaisesRegex(ValueError,'exactly one'):self.check()
        (self.generations/'legacy.extra').rmdir();(self.complete/'install.json').unlink();self.complete.rmdir()
        with self.assertRaisesRegex(ValueError,'exactly one'):self.check()

    def test_residue_cannot_hide_partial_or_foreign_payload(self):
        for path in (self.rollback/'extra',self.rollback/'bin/sifr'):
            path.write_text('unexpected')
            with self.assertRaisesRegex(ValueError,'residue'):self.check()
            path.unlink()
        (self.rollback/'install.json').write_text('changed receipt')
        with self.assertRaisesRegex(ValueError,'residue'):self.check()

    def test_symlink_residue_or_receipt_is_rejected(self):
        (self.rollback/'bin').rmdir();(self.rollback/'bin').symlink_to(self.complete,target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'residue'):self.check()
        (self.rollback/'bin').unlink();(self.rollback/'bin').mkdir()
        (self.rollback/'install.json').unlink();(self.rollback/'install.json').symlink_to(self.complete/'install.json')
        with self.assertRaisesRegex(ValueError,'residue'):self.check()

    def test_predecessor_receipt_and_payload_must_be_exact(self):
        (self.complete/'install.json').write_text('other')
        with self.assertRaisesRegex(ValueError,'receipt'):self.check()
        (self.complete/'install.json').write_text('published receipt')
        with patch.object(inventory,'verify_installed',side_effect=ValueError('installed published payload differs')):
            with self.assertRaisesRegex(ValueError,'payload differs'):self.check()


if __name__=='__main__':unittest.main()
