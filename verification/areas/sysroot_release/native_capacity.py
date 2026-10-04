"""Native artifact rehearsal capacity; Linux cgroup or dedicated Darwin VM."""
import platform
import re
import shutil
import subprocess

from sifr_verify.resource_admission import Resources, discover


def resources(path):
    if platform.system() == 'Linux':
        return discover(disk_path=path)
    if platform.system() != 'Darwin':
        raise ValueError('native artifact qualification requires a supported Unix host')
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
    filesystem=subprocess.check_output(['stat','-f','%T',str(path)],text=True,timeout=30).strip()
    if filesystem not in {'apfs','hfs'}:
        raise ValueError('unknown native Darwin build storage memory authority')
    return Resources(cpus,float(cpus),limit,available,shutil.disk_usage(path).free,{},[],
                     {'capacity_authority':'native-darwin-vm-stat','vm_stat':text},False)
