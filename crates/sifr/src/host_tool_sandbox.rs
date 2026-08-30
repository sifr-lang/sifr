use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const OUTPUT_LIMIT_BYTES: usize = 10 * 1024 * 1024;

struct SandboxTemp(PathBuf);

impl std::ops::Deref for SandboxTemp {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for SandboxTemp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) struct SandboxedToolOutput {
    pub(super) status: ExitStatus,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
    pub(super) output_exceeded_limit: bool,
}

pub(super) struct SandboxedToolRequest<'a> {
    pub(super) executable: &'a Path,
    pub(super) args: &'a [String],
    pub(super) workspace_root: &'a Path,
    pub(super) namespace: &'a str,
    pub(super) capabilities: &'a BTreeSet<String>,
    pub(super) package_checksum: &'a str,
    pub(super) lockfile_fingerprint: &'a str,
    pub(super) executable_hash: &'a str,
}

pub(super) fn run_sandboxed_tool(
    request: &SandboxedToolRequest<'_>,
) -> Result<SandboxedToolOutput, String> {
    let temp = sandbox_temp_dir()?;
    let mut command = sandbox_command(request, &temp)?;
    configure_environment(&mut command, request, &temp);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("cannot start confined host tool: {error}"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "confined host tool has no stdout pipe".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "confined host tool has no stderr pipe".to_string())?;
    let remaining = Arc::new(Mutex::new(OUTPUT_LIMIT_BYTES));
    let exceeded = Arc::new(AtomicBool::new(false));
    let stdout_limit = Arc::clone(&remaining);
    let stderr_limit = Arc::clone(&remaining);
    let stdout_exceeded = Arc::clone(&exceeded);
    let stderr_exceeded = Arc::clone(&exceeded);
    let stdout_reader =
        std::thread::spawn(move || read_bounded(stdout, &stdout_limit, &stdout_exceeded));
    let stderr_reader =
        std::thread::spawn(move || read_bounded(stderr, &stderr_limit, &stderr_exceeded));
    let status = loop {
        if exceeded.load(Ordering::Acquire) {
            if let Some(status) = child
                .try_wait()
                .map_err(|error| format!("cannot poll over-limit host tool: {error}"))?
            {
                break status;
            }
            child
                .kill()
                .map_err(|error| format!("cannot stop over-limit host tool: {error}"))?;
            break child
                .wait()
                .map_err(|error| format!("cannot wait for over-limit host tool: {error}"))?;
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("cannot poll confined host tool: {error}"))?
        {
            break status;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "host-tool stdout reader failed".to_string())??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "host-tool stderr reader failed".to_string())??;
    Ok(SandboxedToolOutput {
        status,
        stdout,
        stderr,
        output_exceeded_limit: exceeded.load(Ordering::Acquire),
    })
}

fn read_bounded(
    mut reader: impl Read,
    remaining: &Mutex<usize>,
    exceeded: &AtomicBool,
) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot read host-tool output: {error}"))?;
        if read == 0 {
            break;
        }
        let mut available = remaining
            .lock()
            .map_err(|_| "host-tool output counter failed".to_string())?;
        let retained = (*available).min(read);
        *available -= retained;
        output.extend_from_slice(&buffer[..retained]);
        if retained < read {
            exceeded.store(true, Ordering::Release);
        }
    }
    Ok(output)
}

fn configure_environment(command: &mut Command, request: &SandboxedToolRequest<'_>, temp: &Path) {
    command.env_clear();
    command.env("TMPDIR", temp);
    command.env("SIFR_TOOL_NAMESPACE", request.namespace);
    command.env(
        "SIFR_TOOL_CAPABILITIES",
        request
            .capabilities
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(","),
    );
    command.env("SIFR_TOOL_PACKAGE_CHECKSUM", request.package_checksum);
    command.env(
        "SIFR_TOOL_LOCKFILE_FINGERPRINT",
        request.lockfile_fingerprint,
    );
    command.env("SIFR_TOOL_EXECUTABLE_SHA256", request.executable_hash);
    if request.capabilities.contains("credentials") {
        if let Some(home) = std::env::var_os("HOME") {
            command.env("HOME", home);
        }
    } else {
        command.env("HOME", temp);
    }
    for (name, value) in std::env::vars_os() {
        let name_text = name.to_string_lossy();
        if matches!(name_text.as_ref(), "HOME" | "PATH" | "TMPDIR")
            || name_text.starts_with("SIFR_TOOL_")
        {
            continue;
        }
        let credential = credential_environment_name(&name_text);
        if (request.capabilities.contains("environment") && !credential)
            || (request.capabilities.contains("credentials") && credential)
        {
            command.env(name, value);
        }
    }
    if request.capabilities.contains("subprocess")
        && let Some(path) = std::env::var_os("PATH")
    {
        command.env("PATH", path);
    }
}

fn credential_environment_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    [
        "AUTH",
        "COOKIE",
        "CREDENTIAL",
        "DATABASE_URL",
        "KEY",
        "PASSWORD",
        "PGPASS",
        "SECRET",
        "SESSION",
        "TOKEN",
    ]
    .iter()
    .any(|marker| upper.contains(marker))
}

fn sandbox_temp_dir() -> Result<SandboxTemp, String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("system clock cannot create tool sandbox identity: {error}"))?
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("sifr-tool-sandbox-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&path)
        .map_err(|error| format!("cannot create tool sandbox '{}': {error}", path.display()))?;
    Ok(SandboxTemp(path))
}

#[cfg(target_os = "macos")]
fn sandbox_command(request: &SandboxedToolRequest<'_>, temp: &Path) -> Result<Command, String> {
    let sandbox = Path::new("/usr/bin/sandbox-exec");
    if !sandbox.is_file() {
        return Err("host-tool confinement requires /usr/bin/sandbox-exec on macOS".to_string());
    }
    let profile = macos_profile(request, temp)?;
    let mut command = Command::new(sandbox);
    command
        .arg("-p")
        .arg(profile)
        .arg(request.executable)
        .args(request.args)
        .current_dir(
            if request.capabilities.contains("project-read")
                || request.capabilities.contains("project-write")
            {
                request.workspace_root
            } else {
                temp
            },
        );
    Ok(command)
}

#[cfg(target_os = "macos")]
fn macos_profile(request: &SandboxedToolRequest<'_>, temp: &Path) -> Result<String, String> {
    let executable = sandbox_literal(request.executable)?;
    let executable_parent = request
        .executable
        .parent()
        .ok_or_else(|| "host-tool executable has no parent directory".to_string())?;
    let executable_parent = sandbox_literal(executable_parent)?;
    let temp = sandbox_literal(temp)?;
    let project = sandbox_literal(request.workspace_root)?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|path| sandbox_literal(&path))
        .transpose()?;
    let mut rules = vec![
        "(version 1)".to_string(),
        "(deny default)".to_string(),
        "(allow process-info*)".to_string(),
        "(allow sysctl-read)".to_string(),
        "(allow mach-lookup)".to_string(),
        format!("(allow process-exec (literal {executable}))"),
        format!("(allow file-read-data (literal {executable}))"),
        "(allow file-read-data (literal \"/\"))".to_string(),
        format!("(allow file-read-metadata (subpath {executable_parent}))"),
        "(allow file-read* (subpath \"/System\") (subpath \"/usr/lib\") (subpath \"/private/etc\"))".to_string(),
        format!("(allow file-read* file-write* (subpath {temp}))"),
        "(allow file-read* file-write* (literal \"/dev/null\"))".to_string(),
    ];
    if request.capabilities.contains("project-read")
        || request.capabilities.contains("project-write")
    {
        rules.push(format!("(allow file-read* (subpath {project}))"));
    }
    if request.capabilities.contains("project-write") {
        rules.push(format!("(allow file-write* (subpath {project}))"));
    }
    if request.capabilities.contains("credentials")
        && let Some(home) = home
    {
        rules.push(format!("(allow file-read* (subpath {home}))"));
    }
    if request.capabilities.contains("network") {
        rules.push("(allow network*)".to_string());
    }
    if request.capabilities.contains("subprocess") {
        rules.push("(allow process*)".to_string());
        rules.push("(allow file-read* (subpath \"/usr/bin\") (subpath \"/bin\"))".to_string());
    }
    Ok(rules.join("\n"))
}

#[cfg(target_os = "macos")]
fn sandbox_literal(path: &Path) -> Result<String, String> {
    let value = path
        .to_str()
        .ok_or_else(|| format!("sandbox path '{}' is not UTF-8", path.display()))?;
    if value.chars().any(char::is_control) {
        return Err(format!(
            "sandbox path '{}' contains a control character",
            path.display()
        ));
    }
    Ok(format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    ))
}

#[cfg(target_os = "linux")]
fn sandbox_command(request: &SandboxedToolRequest<'_>, temp: &Path) -> Result<Command, String> {
    let bwrap = ["/usr/bin/bwrap", "/bin/bwrap"]
        .iter()
        .map(Path::new)
        .find(|path| path.is_file())
        .ok_or_else(|| "host-tool confinement requires bubblewrap on Linux".to_string())?;
    let mut command = Command::new(bwrap);
    command.args(["--die-with-parent", "--new-session", "--unshare-all"]);
    if request.capabilities.contains("network") {
        command.arg("--share-net");
    }
    let mut mount_dirs = BTreeSet::new();
    mount_dirs.insert(PathBuf::from("/usr"));
    if !request.capabilities.contains("subprocess") {
        mount_dirs.insert(PathBuf::from("/usr/bin"));
    }
    for path in [temp, request.executable, request.workspace_root] {
        collect_bwrap_parent_dirs(&mut mount_dirs, path);
    }
    if request.capabilities.contains("credentials")
        && let Some(home) = std::env::var_os("HOME")
    {
        collect_bwrap_parent_dirs(&mut mount_dirs, Path::new(&home));
    }
    for directory in mount_dirs {
        command.arg("--dir").arg(directory);
    }
    for system in ["/etc", "/lib", "/lib64", "/usr/lib", "/usr/lib64"] {
        if Path::new(system).is_dir() {
            command.args(["--ro-bind", system, system]);
        }
    }
    command
        .args(["--dev", "/dev"])
        .args(["--proc", "/proc"])
        .arg("--bind")
        .arg(temp)
        .arg(temp)
        .arg("--ro-bind")
        .arg(request.executable)
        .arg(request.executable);
    if request.capabilities.contains("project-write") {
        command
            .arg("--bind")
            .arg(request.workspace_root)
            .arg(request.workspace_root);
    } else if request.capabilities.contains("project-read") {
        command
            .arg("--ro-bind")
            .arg(request.workspace_root)
            .arg(request.workspace_root);
    }
    if request.capabilities.contains("credentials")
        && let Some(home) = std::env::var_os("HOME")
    {
        command.arg("--ro-bind").arg(&home).arg(&home);
    }
    if request.capabilities.contains("subprocess") {
        for binaries in ["/bin", "/usr/bin"] {
            if Path::new(binaries).is_dir() {
                command.args(["--ro-bind", binaries, binaries]);
            }
        }
    } else {
        let prlimit = Path::new("/usr/bin/prlimit");
        if !prlimit.is_file() {
            return Err(
                "host-tool subprocess confinement requires /usr/bin/prlimit on Linux".to_string(),
            );
        }
        command.arg("--ro-bind").arg(prlimit).arg(prlimit);
    }
    command
        .args(["--chdir"])
        .arg(
            if request.capabilities.contains("project-read")
                || request.capabilities.contains("project-write")
            {
                request.workspace_root
            } else {
                temp
            },
        )
        .arg("--");
    if request.capabilities.contains("subprocess") {
        command.arg(request.executable).args(request.args);
    } else {
        command
            .arg("/usr/bin/prlimit")
            .arg("--nproc=0")
            .arg("--")
            .arg(request.executable)
            .args(request.args);
    }
    Ok(command)
}

#[cfg(target_os = "linux")]
fn collect_bwrap_parent_dirs(directories: &mut BTreeSet<PathBuf>, path: &Path) {
    let parent = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    for ancestor in parent.ancestors().collect::<Vec<_>>().into_iter().rev() {
        if ancestor != Path::new("/") {
            directories.insert(ancestor.to_path_buf());
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn sandbox_command(_request: &SandboxedToolRequest<'_>, _temp: &Path) -> Result<Command, String> {
    Err("host-tool confinement is not implemented on this host operating system".to_string())
}
