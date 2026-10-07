"""Prospective physical temporary storage for the trusted native Darwin recipe.

This closes known preparation paths, not arbitrary build-script filesystem access.
Resident memory remains an estimate; Darwin retains session-group process custody.
"""
from contextlib import contextmanager
import hashlib
import os
from pathlib import Path
import stat
import sys
import tempfile
import tomllib

from native_capacity import storage
from sifr_verify.resource_admission import Resources, admit

GIB=1024**3
TEMP_VARIABLES=('TMPDIR','TMP','TEMP','RUSTC_TMPDIR')
# These are either overwritten by this producer or presentation-only. Unknown
# Cargo configuration environment variables cannot redirect a known write path.
CARGO_ENV={'CARGO_HOME','CARGO_TARGET_DIR','CARGO_BUILD_JOBS','CARGO_INCREMENTAL',
           'CARGO_NET_OFFLINE','CARGO_TERM_COLOR'}
TOOL_OVERRIDES=('CC','CXX','CPP','AR','RANLIB','LD','CFLAGS','CXXFLAGS','CPPFLAGS',
                'LDFLAGS','RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','RUSTFLAGS',
                'CARGO_ENCODED_RUSTFLAGS','CARGO_TARGET_TMPDIR','RUSTC_BOOTSTRAP',
                'CCACHE_DIR','SCCACHE_DIR','MODULE_CACHE_DIR','GCC_EXEC_PREFIX',
                'COMPILER_PATH','PYTHONPATH','PYTHONHOME','PYTHONUSERBASE')
STORE_FIELDS=('DeviceNode','BusProtocol','DeviceIdentifier','ParentWholeDisk','Content',
              'PartitionMapPartition','Internal','WritableMedia','Size','DeviceTreePath')
CACHE_PATHS=('registry','git','.global-cache','.global-cache-shm','.global-cache-wal','.global-cache-journal',
             '.package-cache','.package-cache-mutate')


def requirements(*,physical=False,recovery=False):
    return dict(disk_growth_bytes=(GIB if recovery else 5*GIB//2)+(GIB if physical else 0),
                retained_copy_bytes=GIB//4,disk_reserve_bytes=2*GIB,
                memory_peak_bytes=6*GIB,tmpfs_growth_bytes=0 if physical else GIB,
                memory_reserve_bytes=2*GIB)


def canonical(path):
    path=Path(path)
    if (not path.is_absolute() or path.resolve(strict=True)!=path
            or any(p.is_symlink() for p in (path,*path.parents))):
        raise ValueError('native preparation storage requires canonical paths without links')
    return path


def topology(diagnostics):
    return {key:diagnostics[key] for key in ('storage_device','filesystem','volume_bus_protocol')}|{
        'apfs_backing_stores':[{key:row[key] for key in STORE_FIELDS if key in row}
                               for row in diagnostics['apfs_backing_stores']]}


def inventory(root,output,cargo_home):
    return {'source':root,'output':output,'temporary':output/'temporary','sifr_cache':output/'sifr-cache',
            'clang_cache':output/'clang-cache','cargo_home':cargo_home}


def environment(paths):
    return {name:str(paths['temporary']) for name in TEMP_VARIABLES}|{
        'SIFR_CACHE_DIR':str(paths['sifr_cache']),
        'CLANG_MODULE_CACHE_PATH':str(paths['clang_cache']),'CARGO_HOME':str(paths['cargo_home']),
        'CARGO_TARGET_DIR':str(paths['output']/'target/sysroot_release/cargo-target'),
        'CARGO_NET_OFFLINE':'true','CARGO_INCREMENTAL':'0','PYTHONDONTWRITEBYTECODE':'1'}


def configuration(root,cargo_home,env):
    for name,value in env.items():
        if not value: continue
        # cc-rs accepts target-qualified and HOST_/TARGET_ forms as well.
        if ((name.startswith('CARGO_') and name not in CARGO_ENV)
                or name in TOOL_OVERRIDES
                or any(name.startswith(prefix+'_') or name.endswith('_'+prefix)
                       for prefix in ('CC','CXX','AR','RANLIB','CFLAGS','CXXFLAGS','CPPFLAGS','LDFLAGS'))
                or name.startswith(('CCACHE_','SCCACHE_'))):
            raise ValueError('uncontrolled native preparation tool configuration: '+name)
    expected=root/'.cargo/config.toml'
    candidates={parent/'.cargo'/name for parent in (root,*root.parents) for name in ('config','config.toml')}
    candidates.update(cargo_home/name for name in ('config','config.toml'))
    for path in candidates:
        if path!=expected and (path.exists() or path.is_symlink()):
            raise ValueError('uncontrolled native Cargo configuration: '+str(path))
    canonical(expected)
    data=expected.read_bytes()
    # This is the exact governed source configuration. In particular, [env]
    # must not override TMPDIR, rustc temporary output, or the private caches.
    if tomllib.loads(data.decode())!={'env':{'SYNTAQLITE_SQLITE_VERSION':{'value':'3053004','force':True}}}:
        raise ValueError('unknown native source Cargo configuration')
    return {'path':str(expected),'sha256':hashlib.sha256(data).hexdigest()}


def same_volume_tree(path,device):
    """Reject known cached write paths through links or nested foreign mounts."""
    if not path.exists() and not path.is_symlink(): return
    info=path.lstat()
    if not (stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode)) or info.st_dev!=device:
        raise ValueError('native write path escapes its admitted physical volume: '+str(path))
    if stat.S_ISREG(info.st_mode): return
    def unavailable(error):
        raise ValueError('native write path inventory is unavailable: '+str(error)) from error
    for current,dirs,files in os.walk(path,followlinks=False,onerror=unavailable):
        for child in (Path(current),*(Path(current)/name for name in dirs+files)):
            info=child.lstat()
            if not (stat.S_ISREG(info.st_mode) or stat.S_ISDIR(info.st_mode)) or info.st_dev!=device:
                raise ValueError('native write path escapes its admitted physical volume: '+str(child))


def observe(root,output,cargo_home,env):
    root=canonical(root);output=canonical(output);cargo_home=canonical(cargo_home)
    paths=inventory(root,output,cargo_home)
    config=configuration(root,cargo_home,env)
    device=output.stat().st_dev
    rows={}
    for name,path in paths.items():
        canonical(path);info=path.stat()
        if (not path.is_dir() or info.st_uid!=os.getuid() or info.st_dev!=device
                or info.st_mode & 0o022 or (name not in ('source','cargo_home') and info.st_mode & 0o077)):
            raise ValueError('native write roots require owned physical storage on one volume')
        rows[name]={'path':str(path),'device':info.st_dev,'inode':info.st_ino,'uid':info.st_uid,
                    'mode':stat.S_IMODE(info.st_mode),'storage':topology(storage(path))}
    if any(row['storage']!=rows['output']['storage'] for row in rows.values()):
        raise ValueError('native write roots have different physical storage authority')
    for name in CACHE_PATHS: same_volume_tree(cargo_home/name,device)
    same_volume_tree(output,device)
    return {'protocol':'native-physical-temporary-storage-v1','paths':rows,
            'environment':environment(paths),'cargo_configuration':config,
            'cargo_write_paths':[str(cargo_home/name) for name in CACHE_PATHS],
            'temporary_growth_paths':[str(paths[name]) for name in ('temporary','sifr_cache','clang_cache','cargo_home')],
            'process_memory':'prospective-aggregate-estimate','process_custody':'darwin-session-group'}


def prepare(root,output,env):
    cargo_home=canonical(Path(env.get('CARGO_HOME',str(Path.home()/'.cargo'))))
    for name in ('temporary','sifr-cache','clang-cache'):
        (output/name).mkdir(mode=0o700,exist_ok=False)
    return observe(root,output,cargo_home,env)


@contextmanager
def parent_temporary(record):
    # tempfile caches its first choice independently of later child environments.
    # Restore both that cache and the parent's selected variables on every exit.
    selected={name:record['environment'][name] for name in TEMP_VARIABLES}
    previous={name:os.environ.get(name) for name in selected}
    cached=tempfile.tempdir
    bytecode=sys.dont_write_bytecode
    try:
        os.environ.update(selected);tempfile.tempdir=selected['TMPDIR'];sys.dont_write_bytecode=True
        yield
    finally:
        tempfile.tempdir=cached
        sys.dont_write_bytecode=bytecode
        for name,value in previous.items():
            if value is None: os.environ.pop(name,None)
            else: os.environ[name]=value


def check(record,admission,root,output,env):
    cargo_home=canonical(Path(env.get('CARGO_HOME',str(Path.home()/'.cargo'))))
    if record!=observe(root,output,cargo_home,env):
        raise ValueError('native physical temporary storage evidence differs')
    observed=Resources(**admission['resources'])
    if (observed.disk_memory_backed is not False
            or observed.diagnostics.get('capacity_authority')!='declared-dedicated-darwin-vm-stat'
            or topology(observed.diagnostics)!=record['paths']['output']['storage']
            or admission!=admit(observed,requirements(physical=True))):
        raise ValueError('native physical temporary storage admission differs')
