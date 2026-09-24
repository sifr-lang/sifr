//! Owner-only Windows storage and reparse-point checks.
#![allow(unsafe_code)]
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use std::path::{Component, Path, PathBuf};
use windows_sys::Win32::Foundation::{
    CloseHandle, GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
    GetNamedSecurityInfoW, SDDL_REVISION_1, SE_FILE_OBJECT, SetNamedSecurityInfoW,
};
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION, EqualSid, GetAce,
    GetSecurityDescriptorDacl, GetTokenInformation, IsWellKnownSid, OWNER_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES,
    TOKEN_OWNER, TOKEN_QUERY, TOKEN_USER, TokenOwner, TokenUser, WinBuiltinAdministratorsSid,
    WinLocalSystemSid,
};
use windows_sys::Win32::Storage::FileSystem::{
    CREATE_NEW, CreateDirectoryW, CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
    GetDiskFreeSpaceExW,
};
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

fn denied(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message.into())
}
pub fn wide(path: &OsStr) -> Vec<u16> {
    path.encode_wide().chain(std::iter::once(0)).collect()
}

/// Raw Win32 file APIs need the extended-length form for deep cache keys.
/// Callers validate components before passing paths to these APIs.
fn wide_path(path: &Path) -> Vec<u16> {
    // Verbatim Win32 paths do not translate forward slashes to separators.
    let units: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .map(|unit| {
            if unit == u16::from(b'/') {
                u16::from(b'\\')
            } else {
                unit
            }
        })
        .collect();
    let mut result: Vec<u16> = if !path.is_absolute() || units.starts_with(&[92, 92, 63, 92]) {
        Vec::new()
    } else if units.starts_with(&[92, 92]) {
        // UNC server/share paths use the extended UNC prefix.
        r"\\?\UNC\".encode_utf16().collect()
    } else {
        r"\\?\".encode_utf16().collect()
    };
    result.extend_from_slice(
        if path.is_absolute()
            && units.starts_with(&[92, 92])
            && !units.starts_with(&[92, 92, 63, 92])
        {
            &units[2..]
        } else {
            &units
        },
    );
    result.push(0);
    result
}

struct Owner {
    token: windows_sys::Win32::Foundation::HANDLE,
    bytes: Vec<u8>,
}
impl Owner {
    fn sid(&self) -> PSID {
        // SAFETY: GetTokenInformation initialized this buffer as TOKEN_USER.
        unsafe {
            self.bytes
                .as_ptr()
                .cast::<TOKEN_USER>()
                .read_unaligned()
                .User
                .Sid
        }
    }
}
impl Drop for Owner {
    fn drop(&mut self) {
        // SAFETY: OpenProcessToken returned this owned handle.
        unsafe { CloseHandle(self.token) };
    }
}
fn owner() -> io::Result<Owner> {
    // SAFETY: the pseudo process handle is valid in this process; the token
    // handle and output buffer have valid storage.
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0;
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &raw mut len);
        if len < std::mem::size_of::<TOKEN_USER>() as u32 {
            CloseHandle(token);
            return Err(io::Error::last_os_error());
        }
        let mut bytes = vec![0; len as usize];
        if GetTokenInformation(
            token,
            TokenUser,
            bytes.as_mut_ptr().cast(),
            len,
            &raw mut len,
        ) == 0
        {
            let error = io::Error::last_os_error();
            CloseHandle(token);
            return Err(error);
        }
        Ok(Owner { token, bytes })
    }
}
#[cfg(test)]
pub fn token_default_owner_is_administrators() -> io::Result<bool> {
    let owner = owner()?;
    let mut len = 0;
    // SAFETY: the first call requests the required buffer size; the second
    // writes TOKEN_OWNER into the allocated buffer while the token is live.
    unsafe {
        GetTokenInformation(
            owner.token,
            TokenOwner,
            std::ptr::null_mut(),
            0,
            &raw mut len,
        );
        if len < std::mem::size_of::<TOKEN_OWNER>() as u32 {
            return Err(io::Error::last_os_error());
        }
        let mut bytes = vec![0; len as usize];
        if GetTokenInformation(
            owner.token,
            TokenOwner,
            bytes.as_mut_ptr().cast(),
            len,
            &raw mut len,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        let sid = bytes.as_ptr().cast::<TOKEN_OWNER>().read_unaligned().Owner;
        Ok(IsWellKnownSid(sid, WinBuiltinAdministratorsSid) != 0)
    }
}

struct Descriptor(PSECURITY_DESCRIPTOR);
impl Drop for Descriptor {
    fn drop(&mut self) {
        // SAFETY: Windows allocated this security descriptor with LocalAlloc.
        unsafe { LocalFree(self.0) };
    }
}
fn private_descriptor() -> io::Result<Descriptor> {
    let owner = owner()?;
    // SAFETY: ConvertSidToStringSidW allocates a null-terminated string and
    // LocalFree releases it after copying into Rust-owned UTF-16 text.
    let sid = unsafe {
        let mut ptr = std::ptr::null_mut();
        if ConvertSidToStringSidW(owner.sid(), &raw mut ptr) == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }
        let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
        LocalFree(ptr.cast());
        text
    };
    // Protected DACL: only the actual token owner has full access. Children
    // inherit the owner-only grant; no ambient Users/Everyone ACL is retained.
    let sddl = format!("O:{sid}D:P(A;OICI;FA;;;{sid})");
    let wide = wide(OsStr::new(&sddl));
    // SAFETY: the SDDL string is null terminated; Windows allocates the result.
    let mut descriptor = std::ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide.as_ptr(),
            SDDL_REVISION_1,
            &raw mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(Descriptor(descriptor))
}
fn attributes(descriptor: &Descriptor) -> SECURITY_ATTRIBUTES {
    SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    }
}
pub fn create_directory(path: &Path) -> io::Result<()> {
    no_reparse(
        path.parent()
            .ok_or_else(|| denied("missing directory parent"))?,
    )?;
    let descriptor = private_descriptor()?;
    let attrs = attributes(&descriptor);
    let path = wide_path(path);
    // SAFETY: both pointers remain live for the duration of CreateDirectoryW.
    if unsafe { CreateDirectoryW(path.as_ptr(), &raw const attrs) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
pub fn create_file(path: &Path) -> io::Result<File> {
    no_reparse(path.parent().ok_or_else(|| denied("missing file parent"))?)?;
    let descriptor = private_descriptor()?;
    let attrs = attributes(&descriptor);
    let path = wide_path(path);
    // SAFETY: CreateFileW returns a uniquely owned handle or INVALID_HANDLE_VALUE.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            &raw const attrs,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: this transfers the uniquely owned handle to File.
    Ok(unsafe { File::from_raw_handle(handle) })
}
pub fn no_reparse(path: &Path) -> io::Result<()> {
    if !path.is_absolute() {
        return Err(denied("storage path must be absolute"));
    }
    let mut current = PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::CurDir | Component::ParentDir) {
            return Err(denied("storage path traversal"));
        }
        current.push(component);
        if matches!(component, Component::Prefix(_)) {
            continue;
        }
        match fs::symlink_metadata(&current) {
            Ok(meta) => {
                use std::os::windows::fs::MetadataExt;
                if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return Err(denied(format!(
                        "reparse-point storage path {}",
                        current.display()
                    )));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
pub fn check(path: &Path) -> io::Result<()> {
    no_reparse(path)?;
    let meta = fs::symlink_metadata(path)?;
    use std::os::windows::fs::MetadataExt;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(denied("reparse-point storage entry"));
    }
    let owner = owner()?;
    let wide = wide_path(path);
    let mut actual_owner: PSID = std::ptr::null_mut();
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SAFETY: the requested owner/DACL pointers refer to the returned descriptor.
    let result = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &raw mut actual_owner,
            std::ptr::null_mut(),
            &raw mut dacl,
            std::ptr::null_mut(),
            &raw mut descriptor,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let descriptor = Descriptor(descriptor);
    // SAFETY: all SID/ACE pointers refer to the live descriptor. Deny ACEs
    // cannot grant access; every allow ACE must name the current token owner.
    unsafe {
        if actual_owner.is_null() || dacl.is_null() || EqualSid(owner.sid(), actual_owner) == 0 {
            return Err(denied("foreign owner or null storage DACL"));
        }
        for index in 0..(*dacl).AceCount {
            let mut ace = std::ptr::null_mut();
            if GetAce(dacl, u32::from(index), &raw mut ace) == 0 {
                return Err(io::Error::last_os_error());
            }
            let header = &*ace.cast::<ACE_HEADER>();
            if header.AceType == 0 {
                let allow = &*ace.cast::<ACCESS_ALLOWED_ACE>();
                let sid = (&raw const allow.SidStart).cast_mut().cast();
                if EqualSid(owner.sid(), sid) == 0 {
                    return Err(denied("storage ACL grants another principal access"));
                }
            } else if header.AceType != 1 {
                return Err(denied("unrecognized storage ACL grant"));
            }
        }
    }
    let _ = descriptor;
    Ok(())
}
pub fn available_bytes(path: &Path) -> io::Result<u64> {
    let wide = wide_path(path);
    let mut free = 0;
    // SAFETY: the path and output pointer are valid.
    if unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &raw mut free,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(free)
}

pub fn seal(path: &Path) -> io::Result<()> {
    no_reparse(path)?;
    let owner = owner()?;
    let wide = wide_path(path);
    let mut actual_owner: PSID = std::ptr::null_mut();
    let mut actual_descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SAFETY: the owner pointer is valid until the returned descriptor is freed.
    let result = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            &raw mut actual_owner,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &raw mut actual_descriptor,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let actual_descriptor = Descriptor(actual_descriptor);
    // SAFETY: both SID pointers remain valid for this comparison.
    let mut owner_len = 0;
    // SAFETY: the token is live. This first call obtains the required size.
    unsafe {
        GetTokenInformation(
            owner.token,
            TokenOwner,
            std::ptr::null_mut(),
            0,
            &raw mut owner_len,
        )
    };
    if owner_len < std::mem::size_of::<TOKEN_OWNER>() as u32 {
        return Err(io::Error::last_os_error());
    }
    let mut owner_bytes = vec![0; owner_len as usize];
    // SAFETY: the buffer is large enough to hold the token default owner.
    if unsafe {
        GetTokenInformation(
            owner.token,
            TokenOwner,
            owner_bytes.as_mut_ptr().cast(),
            owner_len,
            &raw mut owner_len,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful token query initialized TOKEN_OWNER in this buffer.
    let default_sid = unsafe {
        owner_bytes
            .as_ptr()
            .cast::<TOKEN_OWNER>()
            .read_unaligned()
            .Owner
    };
    if actual_owner.is_null()
        || unsafe { EqualSid(owner.sid(), actual_owner) } == 0
            && unsafe { EqualSid(default_sid, actual_owner) } == 0
    {
        return Err(denied("foreign-owned staged payload"));
    }
    let descriptor = private_descriptor()?;
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl: *mut ACL = std::ptr::null_mut();
    // SAFETY: the descriptor is valid; the requested DACL remains live for the
    // following SetNamedSecurityInfoW call.
    if unsafe {
        GetSecurityDescriptorDacl(
            descriptor.0,
            &raw mut present,
            &raw mut dacl,
            &raw mut defaulted,
        )
    } == 0
        || present == 0
        || dacl.is_null()
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: only the DACL of the owned path is changed, and the DACL points
    // into the live private descriptor.
    let result = unsafe {
        SetNamedSecurityInfoW(
            wide.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION
                | DACL_SECURITY_INFORMATION
                | PROTECTED_DACL_SECURITY_INFORMATION,
            owner.sid(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null_mut(),
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let _ = actual_descriptor;
    check(path)
}

pub fn open_read(path: &Path) -> io::Result<File> {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    check(path)?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    use std::os::windows::fs::MetadataExt;
    if file.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(denied("reparse-point storage file"));
    }
    Ok(file)
}

pub fn durable_rename(source: &Path, destination: &Path) -> io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    };
    check(source)?;
    let parent = destination
        .parent()
        .ok_or_else(|| denied("missing publication parent"))?;
    no_reparse(parent)?;
    if fs::symlink_metadata(destination).is_ok() {
        check(destination)?;
    }
    let directory = fs::symlink_metadata(source)?.is_dir();
    let source = wide_path(source);
    let destination_name = wide_path(destination);
    // Directory winners must never be replaced. A competing complete winner
    // is validated by the caller after this reports AlreadyExists.
    let flags = if directory {
        MOVEFILE_WRITE_THROUGH
    } else {
        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH
    };
    // SAFETY: both names are terminated and live through the synchronous move.
    if unsafe { MoveFileExW(source.as_ptr(), destination_name.as_ptr(), flags) } == 0 {
        let error = io::Error::last_os_error();
        if directory && fs::symlink_metadata(destination).is_ok() {
            check(destination)?;
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, error));
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
pub fn test_grant_world(path: &Path) -> io::Result<()> {
    let sddl = wide(OsStr::new("D:P(A;;FA;;;WD)"));
    let mut descriptor = std::ptr::null_mut();
    // SAFETY: test-only descriptor uses fixed, valid SDDL.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            SDDL_REVISION_1,
            &raw mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let descriptor = Descriptor(descriptor);
    let mut present = 0;
    let mut defaulted = 0;
    let mut dacl = std::ptr::null_mut();
    // SAFETY: descriptor and returned DACL remain live through mutation.
    if unsafe {
        GetSecurityDescriptorDacl(
            descriptor.0,
            &raw mut present,
            &raw mut dacl,
            &raw mut defaulted,
        )
    } == 0
        || present == 0
        || dacl.is_null()
    {
        return Err(io::Error::last_os_error());
    }
    let path = wide_path(path);
    // SAFETY: test deliberately installs an unsafe DACL to verify rejection.
    let result = unsafe {
        SetNamedSecurityInfoW(
            path.as_ptr().cast_mut(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            dacl,
            std::ptr::null_mut(),
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    Ok(())
}

pub fn file_identity(file: &File) -> io::Result<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    let mut information = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::uninit();
    // SAFETY: the File owns a live handle and the output has the required size.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: a successful call initialized every field.
    let information = unsafe { information.assume_init() };
    let index =
        (u64::from(information.nFileIndexHigh) << 32) | u64::from(information.nFileIndexLow);
    Ok((u64::from(information.dwVolumeSerialNumber), index))
}

pub fn path_identity(path: &Path) -> io::Result<(u64, u64)> {
    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
    no_reparse(path)?;
    let wide = wide_path(path);
    // SAFETY: the path is terminated and the returned handle is transferred to File.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null(),
            windows_sys::Win32::Storage::FileSystem::OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            std::ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: CreateFileW returned a unique owned handle.
    let file = unsafe { File::from_raw_handle(handle) };
    use std::os::windows::fs::MetadataExt;
    if file.metadata()?.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(denied("reparse-point workspace identity"));
    }
    file_identity(&file)
}

/// Caller-owned workspaces keep ordinary Windows ACLs, including privileged
/// SYSTEM/Administrators grants. Reject writable grants to other principals.
pub fn safe_workspace(path: &Path) -> io::Result<()> {
    use windows_sys::Win32::Foundation::{GENERIC_ALL, GENERIC_WRITE};
    use windows_sys::Win32::Storage::FileSystem::{
        DELETE, FILE_ADD_FILE, FILE_APPEND_DATA, FILE_DELETE_CHILD, FILE_WRITE_ATTRIBUTES,
        FILE_WRITE_DATA, WRITE_DAC, WRITE_OWNER,
    };
    no_reparse(path)?;
    if !fs::symlink_metadata(path)?.is_dir() {
        return Err(denied("workspace is not a directory"));
    }
    let owner = owner()?;
    let wide = wide_path(path);
    let mut actual_owner: PSID = std::ptr::null_mut();
    let mut dacl: *mut ACL = std::ptr::null_mut();
    let mut descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
    // SAFETY: Windows fills the requested pointers within the returned descriptor.
    let result = unsafe {
        GetNamedSecurityInfoW(
            wide.as_ptr(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &raw mut actual_owner,
            std::ptr::null_mut(),
            &raw mut dacl,
            std::ptr::null_mut(),
            &raw mut descriptor,
        )
    };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result as i32));
    }
    let _descriptor = Descriptor(descriptor);
    // SAFETY: all SID and ACE pointers belong to the live security descriptor.
    unsafe {
        if actual_owner.is_null()
            || EqualSid(owner.sid(), actual_owner) == 0
                && IsWellKnownSid(actual_owner, WinBuiltinAdministratorsSid) == 0
        {
            return Err(denied("foreign workspace owner"));
        }
        if dacl.is_null() {
            return Err(denied("unrestricted workspace DACL"));
        }
        let write_mask = GENERIC_ALL
            | GENERIC_WRITE
            | DELETE
            | FILE_ADD_FILE
            | FILE_APPEND_DATA
            | FILE_DELETE_CHILD
            | FILE_WRITE_ATTRIBUTES
            | FILE_WRITE_DATA
            | WRITE_DAC
            | WRITE_OWNER;
        for index in 0..(*dacl).AceCount {
            let mut ace = std::ptr::null_mut();
            if GetAce(dacl, u32::from(index), &raw mut ace) == 0 {
                return Err(io::Error::last_os_error());
            }
            let header = &*ace.cast::<ACE_HEADER>();
            if header.AceFlags & 0x08 != 0 {
                continue; // INHERIT_ONLY_ACE does not grant access to this workspace.
            }
            if header.AceType == 0 {
                let allow = &*ace.cast::<ACCESS_ALLOWED_ACE>();
                let sid = (&raw const allow.SidStart).cast_mut().cast();
                if allow.Mask & write_mask != 0
                    && EqualSid(owner.sid(), sid) == 0
                    && IsWellKnownSid(sid, WinBuiltinAdministratorsSid) == 0
                    && IsWellKnownSid(sid, WinLocalSystemSid) == 0
                {
                    return Err(denied(
                        "workspace ACL grants another principal write access",
                    ));
                }
            } else if header.AceType != 1 {
                return Err(denied("unrecognized workspace ACL grant"));
            }
        }
    }
    Ok(())
}
