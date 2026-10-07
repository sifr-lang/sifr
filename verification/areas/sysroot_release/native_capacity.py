"""Native artifact rehearsal capacity; Linux cgroup or dedicated Darwin VM."""
import os
import platform
import plistlib
import re
import shutil
import subprocess

from sifr_verify.resource_admission import Resources, discover


PHYSICAL_BUSES={'PCI-Express','SATA','ATA','USB','Thunderbolt','NVMe','Apple Fabric','SCSI'}


def known_apfs_store(info,device):
    if info.get('DeviceNode')!=device:
        return False
    if info.get('BusProtocol') in PHYSICAL_BUSES:
        return True
    # Apple's dedicated ARM guest exposes its block disk through VirtIO with
    # an empty bus field. Require the OS's identified writable APFS partition,
    # its parent disk and the specific storage driver; an empty bus alone is
    # never authority. Explicit unknown/image buses cannot use this route.
    identifier=device.removeprefix('/dev/')
    parent=re.fullmatch(r'(disk[0-9]+)s[0-9]+',identifier)
    tree=info.get('DeviceTreePath')
    return (info.get('BusProtocol')=='' and parent is not None
            and info.get('DeviceIdentifier')==identifier
            and info.get('ParentWholeDisk')==parent.group(1)
            and info.get('Content')=='Apple_APFS'
            and info.get('PartitionMapPartition') is True
            and info.get('Internal') is True and info.get('WritableMedia') is True
            and type(info.get('Size')) is int and info['Size']>0
            and isinstance(tree,str) and re.fullmatch(
                r'IODeviceTree:/arm-io/pcie@[0-9a-f]+/pci[0-9a-f]+,[0-9a-f]+@[0-9a-f]+/AppleVirtIOStorageDevice',tree) is not None)


def apfs_store_devices(storage):
    """APFS volumes inherit disk authority from every declared backing store."""
    stores=storage.get('APFSPhysicalStores')
    if not isinstance(stores,list) or not stores:
        raise ValueError('native Darwin APFS backing stores are unavailable')
    devices=[]
    for store in stores:
        identifier=store.get('APFSPhysicalStore') if isinstance(store,dict) else None
        if not isinstance(identifier,str) or not re.fullmatch(r'disk[0-9]+(?:s[0-9]+)?',identifier):
            raise ValueError('native Darwin APFS backing store identifier is invalid')
        device='/dev/'+identifier
        if device in devices:
            raise ValueError('native Darwin APFS backing stores are duplicated')
        devices.append(device)
    return devices


def storage(path):
    """Physical Darwin storage authority, reusable for each actual write root."""
    devices=subprocess.check_output(['df','-P',str(path)],text=True,timeout=30).splitlines()
    if len(devices)<2 or not devices[-1].split():
        raise ValueError('native Darwin storage device is unavailable')
    device=devices[-1].split()[0]
    if not device.startswith('/dev/'):
        raise ValueError('native Darwin storage device is unavailable')
    storage=plistlib.loads(subprocess.check_output(['diskutil','info','-plist',device],timeout=30))
    filesystem=storage.get('FilesystemType')
    backing=[]
    if filesystem=='apfs':
        for store_device in apfs_store_devices(storage):
            info=plistlib.loads(subprocess.check_output(['diskutil','info','-plist',store_device],timeout=30))
            if not known_apfs_store(info,store_device):
                raise ValueError('unknown native Darwin build storage memory authority')
            backing.append(info)
    elif filesystem!='hfs' or storage.get('BusProtocol') not in PHYSICAL_BUSES:
        raise ValueError('unknown native Darwin build storage memory authority')
    return {'storage_device':device,'filesystem':filesystem,
            'volume_bus_protocol':storage.get('BusProtocol'),'apfs_backing_stores':backing}


def resources(path):
    if platform.system() == 'Linux':
        return discover(disk_path=path)
    if platform.system() != 'Darwin':
        raise ValueError('native artifact qualification requires a supported Unix host')
    # Host ownership is an explicit operator declaration, not inferred from
    # available memory. Shared/unclassified Darwin hosts cannot use VM totals.
    if os.environ.get('SIFR_NATIVE_HOST_KIND')!='dedicated-darwin':
        raise ValueError('Darwin preparation requires an explicitly dedicated host')
    def sysctl(name):
        return int(subprocess.check_output(['sysctl','-n',name],text=True,timeout=30).strip())
    text=subprocess.check_output(['vm_stat'],text=True,timeout=30)
    match=re.search(r'page size of (\d+) bytes',text)
    if match is None: raise ValueError('native Darwin page size is unavailable')
    page=int(match[1]);rows={}
    for line in text.splitlines()[1:]:
        parsed=re.fullmatch(r'([^:]+):\s+(\d+)\.',line)
        if parsed: rows[parsed[1]]=int(parsed[2])
    available=page*sum(rows[name] for name in ('Pages free','Pages inactive','Pages speculative'))
    limit=sysctl('hw.memsize');cpus=sysctl('hw.logicalcpu')
    if page<=0 or limit<=0 or cpus<=0 or not 0<=available<=limit:
        raise ValueError('native Darwin capacity is invalid')
    disk=storage(path)
    return Resources(cpus,float(cpus),limit,available,shutil.disk_usage(path).free,{},[],
                     {'capacity_authority':'declared-dedicated-darwin-vm-stat','vm_stat':text,**disk},False)
