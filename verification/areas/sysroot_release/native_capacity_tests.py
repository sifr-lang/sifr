"""Darwin VM and storage authorities use BSD-native interfaces; no native pass claimed."""
import os
from pathlib import Path
import plistlib
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import native_candidate  # initializes the canonical runner import path
import native_capacity as capacity


class CapacityChecks(unittest.TestCase):
    def observe(self,storage=None,*,declared=True,backing=None):
        storage=storage or {'FilesystemType':'apfs','BusProtocol':'Apple Fabric',
                           'APFSPhysicalStores':[{'APFSPhysicalStore':'disk0s2'}]}
        backing=backing or {'/dev/disk0s2':{'DeviceNode':'/dev/disk0s2','BusProtocol':'Apple Fabric'}}
        calls=[]
        def command(argv,**kwargs):
            calls.append(argv)
            if argv==['sysctl','-n','hw.memsize']: return str(16*1024**3)+'\n'
            if argv==['sysctl','-n','hw.logicalcpu']: return '8\n'
            if argv==['vm_stat']:
                return 'Mach Virtual Memory Statistics: (page size of 16384 bytes)\nPages free: 100000.\nPages inactive: 200000.\nPages speculative: 10000.\n'
            if argv==['df','-P','/native/output']: return 'Filesystem 512-blocks Used Available Capacity Mounted on\n/dev/disk3s1 1000 10 990 1% /System/Volumes/Data\n'
            if argv==['diskutil','info','-plist','/dev/disk3s1']: return plistlib.dumps(storage)
            if len(argv)==4 and argv[:3]==['diskutil','info','-plist'] and argv[-1] in backing:
                return plistlib.dumps(backing[argv[-1]])
            raise AssertionError('unexpected capacity command: '+repr(argv))
        with patch.dict(os.environ,{'SIFR_NATIVE_HOST_KIND':'dedicated-darwin'} if declared else {},clear=True), \
             patch.object(capacity.platform,'system',return_value='Darwin'), \
             patch.object(capacity.subprocess,'check_output',side_effect=command), \
             patch.object(capacity.shutil,'disk_usage',return_value=SimpleNamespace(free=30*1024**3)):
            value=capacity.resources(Path('/native/output'))
        return value,calls

    def test_native_bsd_device_authority_and_measured_available_pages(self):
        observed,calls=self.observe()
        self.assertEqual(observed.memory_available_bytes,310000*16384)
        self.assertEqual(observed.memory_limit_bytes,16*1024**3)
        self.assertEqual(observed.effective_cpus,8)
        self.assertFalse(observed.disk_memory_backed)
        self.assertIn(['diskutil','info','-plist','/dev/disk3s1'],calls)
        self.assertIn(['diskutil','info','-plist','/dev/disk0s2'],calls)

    def test_apfs_volume_bus_is_not_its_backing_disk_authority(self):
        storage={'FilesystemType':'apfs','BusProtocol':'',
                 'APFSPhysicalStores':[{'APFSPhysicalStore':'disk0s2'}]}
        observed,_=self.observe(storage)
        self.assertFalse(observed.disk_memory_backed)
        self.assertEqual(observed.diagnostics['apfs_backing_stores'][0]['BusProtocol'],'Apple Fabric')
        for bus in ('','Disk Image','Unknown'):
            with self.subTest(bus=bus),self.assertRaisesRegex(ValueError,'storage memory authority'):
                self.observe(storage,backing={'/dev/disk0s2':{'DeviceNode':'/dev/disk0s2','BusProtocol':bus}})
        with self.assertRaisesRegex(ValueError,'storage memory authority'):
            self.observe(storage,backing={'/dev/disk0s2':{'DeviceNode':'/dev/disk9','BusProtocol':'SATA'}})

    def test_every_apfs_backing_store_must_be_known_and_identified(self):
        stores=[{'APFSPhysicalStore':'disk0s2'},{'APFSPhysicalStore':'disk1s2'}]
        storage={'FilesystemType':'apfs','BusProtocol':'SATA','APFSPhysicalStores':stores}
        backing={'/dev/disk0s2':{'DeviceNode':'/dev/disk0s2','BusProtocol':'SATA'},
                 '/dev/disk1s2':{'DeviceNode':'/dev/disk1s2','BusProtocol':'Disk Image'}}
        with self.assertRaisesRegex(ValueError,'storage memory authority'):
            self.observe(storage,backing=backing)
        with self.assertRaisesRegex(ValueError,'backing stores are unavailable'):
            capacity.apfs_store_devices({'APFSPhysicalStores':None})
        for stores in ('unknown',[],[{}],[{'APFSPhysicalStore':'../disk0'}],
                       [{'APFSPhysicalStore':'disk0s2'}]*2):
            with self.subTest(stores=stores),self.assertRaises(ValueError):
                self.observe({'FilesystemType':'apfs','BusProtocol':'SATA','APFSPhysicalStores':stores})

    def test_shared_or_unknown_host_and_ram_disk_are_rejected(self):
        with self.assertRaisesRegex(ValueError,'dedicated'): self.observe(declared=False)
        for bus in ('Disk Image','DiskImage','Unknown'):
            with self.subTest(bus=bus),self.assertRaisesRegex(ValueError,'storage memory authority'):
                self.observe({'FilesystemType':'hfs','BusProtocol':bus})

    def test_linux_uses_actual_cgroup_and_ram_backed_mount_authority(self):
        expected=object()
        with patch.object(capacity.platform,'system',return_value='Linux'),patch.object(capacity,'discover',return_value=expected) as discover:
            self.assertIs(capacity.resources(Path('/owned')),expected)
            discover.assert_called_once_with(disk_path=Path('/owned'))


if __name__=='__main__': unittest.main()
