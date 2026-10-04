"""Native artifact rehearsal capacity; Linux cgroup or dedicated Darwin VM."""
import os
import platform
import plistlib
import re
import shutil
import subprocess

from sifr_verify.resource_admission import Resources, discover


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
    devices=subprocess.check_output(['df','-P',str(path)],text=True,timeout=30).splitlines()
    if len(devices)<2 or not devices[-1].split():
        raise ValueError('native Darwin storage device is unavailable')
    device=devices[-1].split()[0]
    if not device.startswith('/dev/'):
        raise ValueError('native Darwin storage device is unavailable')
    storage=plistlib.loads(subprocess.check_output(['diskutil','info','-plist',device],timeout=30))
    physical_buses={'PCI-Express','SATA','ATA','USB','Thunderbolt','NVMe','Apple Fabric','SCSI'}
    if storage.get('FilesystemType') not in {'apfs','hfs'} or storage.get('BusProtocol') not in physical_buses:
        raise ValueError('unknown native Darwin build storage memory authority')
    return Resources(cpus,float(cpus),limit,available,shutil.disk_usage(path).free,{},[],
                     {'capacity_authority':'declared-dedicated-darwin-vm-stat','vm_stat':text,
                      'storage_device':device,'filesystem':storage['FilesystemType'],'bus_protocol':storage['BusProtocol']},False)
