use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

const NODE_RELEASE_BASE: &str = "https://nodejs.org/download/release/latest-v24.x";
const NODE_CACHE_NAME: &str = "node-lts-v24";
const NODE_DOWNLOAD_LIMIT: u64 = 200 * 1024 * 1024;
const ABP_CLI_CACHE_NAME: &str = "abp-cli";
const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "Furkan Cemal Caliskan";
const APPLICATION: &str = "banna";

#[derive(Debug)]
pub(super) struct ResolvedProgram {
    pub executable: PathBuf,
    pub prefix_args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub notices: Vec<String>,
}

impl ResolvedProgram {
    fn direct(executable: PathBuf) -> Self {
        Self {
            executable,
            prefix_args: Vec::new(),
            env: BTreeMap::new(),
            notices: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveFormat {
    TarGz,
    Zip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeArchive {
    target: &'static str,
    extension: &'static str,
    format: ArchiveFormat,
}

pub(super) struct Toolchain {
    root: PathBuf,
    os: &'static str,
    arch: &'static str,
}

impl Toolchain {
    pub(super) fn platform_default() -> Self {
        Self {
            root: toolchain_root(),
            os: env::consts::OS,
            arch: env::consts::ARCH,
        }
    }

    #[cfg(test)]
    fn at(root: PathBuf, os: &'static str, arch: &'static str) -> Self {
        Self { root, os, arch }
    }

    pub(super) fn resolve(&self, program: &str) -> Result<ResolvedProgram> {
        match program {
            "npm" => self.resolve_npm(),
            "abp" => self.resolve_abp(),
            "dotnet" => self.resolve_dotnet(),
            other => find_on_path(other)
                .map(ResolvedProgram::direct)
                .ok_or_else(|| missing_tool_error(other)),
        }
    }

    fn resolve_dotnet(&self) -> Result<ResolvedProgram> {
        find_on_path("dotnet")
            .map(ResolvedProgram::direct)
            .ok_or_else(|| missing_tool_error("dotnet"))
    }

    fn resolve_npm(&self) -> Result<ResolvedProgram> {
        if let Some(resolved) = system_npm() {
            return Ok(resolved);
        }
        if let Some(resolved) = self.cached_npm() {
            return Ok(resolved);
        }
        self.install_node()
    }

    fn resolve_abp(&self) -> Result<ResolvedProgram> {
        // ABP's install-libs command starts npm itself. Resolving npm first also
        // makes the portable Node bin directory available to that child process.
        let npm = self.resolve_npm()?;
        let mut notices = npm.notices;
        let node_env = npm.env;

        // Keep Banna's ABP CLI isolated from global tools. Omitting --version
        // lets dotnet resolve the latest stable release on first installation.
        let install_dir = self.root.join(ABP_CLI_CACHE_NAME);
        let executable = abp_executable(&install_dir);
        if executable.is_file() {
            return Ok(ResolvedProgram {
                executable,
                prefix_args: Vec::new(),
                env: node_env,
                notices,
            });
        }

        fs::create_dir_all(&self.root).with_context(|| {
            format!(
                "failed to create Banna toolchain directory {}",
                self.root.display()
            )
        })?;
        let _installation_lock = installation_lock(&self.root)?;
        let executable = abp_executable(&install_dir);
        if executable.is_file() {
            return Ok(ResolvedProgram {
                executable,
                prefix_args: Vec::new(),
                env: node_env,
                notices,
            });
        }
        let dotnet = self.resolve_dotnet()?;
        let temporary = tempfile::Builder::new()
            .prefix("abp-install-")
            .tempdir_in(&self.root)
            .context("failed to create temporary ABP CLI install directory")?;
        let output = Command::new(&dotnet.executable)
            .args(["tool", "install", "Volo.Abp.Cli", "--tool-path"])
            .arg(temporary.path())
            .env("DOTNET_NOLOGO", "1")
            .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
            .env("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("failed to start dotnet while installing the ABP CLI")?;
        if !output.status.success() {
            return Err(anyhow!(
                "failed to install the latest ABP CLI into Banna's private tool cache (exit status {}).\n{}",
                output.status.code().map_or_else(
                    || "terminated by signal".to_owned(),
                    |code| code.to_string()
                ),
                command_diagnostic(&output.stdout, &output.stderr)
            ));
        }

        let executable = abp_executable(&install_dir);
        if executable.is_file() {
            notices.push("ABP CLI was installed by another Banna process".into());
            return Ok(ResolvedProgram {
                executable,
                prefix_args: Vec::new(),
                env: node_env,
                notices,
            });
        }
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).with_context(|| {
                format!(
                    "failed to replace incomplete cache {}",
                    install_dir.display()
                )
            })?;
        }
        let temporary_path = temporary.keep();
        if let Err(error) = fs::rename(&temporary_path, &install_dir) {
            let concurrent_executable = abp_executable(&install_dir);
            if !concurrent_executable.is_file() {
                return Err(error).with_context(|| {
                    format!("failed to activate ABP CLI cache {}", install_dir.display())
                });
            }
        }
        let executable = abp_executable(&install_dir);
        if !executable.is_file() {
            return Err(anyhow!(
                "ABP CLI installation completed but the abp executable is missing"
            ));
        }
        notices.push(
            "ABP CLI was not found; installed the latest Volo.Abp.Cli in Banna's private tool cache"
                .into(),
        );
        Ok(ResolvedProgram {
            executable,
            prefix_args: Vec::new(),
            env: node_env,
            notices,
        })
    }

    fn cached_npm(&self) -> Option<ResolvedProgram> {
        npm_from_node_root(&self.root.join(NODE_CACHE_NAME), self.os)
    }

    fn install_node(&self) -> Result<ResolvedProgram> {
        let archive = node_archive(self.os, self.arch)?;
        fs::create_dir_all(&self.root).with_context(|| {
            format!(
                "failed to create Banna toolchain directory {}",
                self.root.display()
            )
        })?;
        let _installation_lock = installation_lock(&self.root)?;
        if let Some(resolved) = self.cached_npm() {
            return Ok(resolved);
        }
        let agent_config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10 * 60)))
            .build();
        let agent: ureq::Agent = agent_config.into();
        let checksums_url = format!("{NODE_RELEASE_BASE}/SHASUMS256.txt");
        let mut response = agent.get(&checksums_url).call().with_context(|| {
            format!("failed to download Node.js checksums from {checksums_url}")
        })?;
        let checksums = response
            .body_mut()
            .with_config()
            .limit(1024 * 1024)
            .read_to_string()
            .context("failed to read Node.js checksum manifest")?;
        let (expected_hash, filename) = select_archive_from_manifest(&checksums, archive)
            .ok_or_else(|| {
                anyhow!(
                    "Node.js LTS does not publish a {} archive for {}/{}",
                    archive.target,
                    self.os,
                    self.arch
                )
            })?;
        let archive_url = format!("{NODE_RELEASE_BASE}/{filename}");
        let mut response = agent
            .get(&archive_url)
            .call()
            .with_context(|| format!("failed to download portable Node.js from {archive_url}"))?;
        let mut archive_file = tempfile::Builder::new()
            .prefix("node-download-")
            .tempfile_in(&self.root)
            .context("failed to create temporary Node.js archive")?;
        let downloaded = io::copy(
            &mut response
                .body_mut()
                .as_reader()
                .take(NODE_DOWNLOAD_LIMIT + 1),
            &mut archive_file,
        )
        .context("failed while downloading portable Node.js")?;
        if downloaded > NODE_DOWNLOAD_LIMIT {
            return Err(anyhow!(
                "portable Node.js download exceeded the {} MiB safety limit",
                NODE_DOWNLOAD_LIMIT / 1024 / 1024
            ));
        }
        let actual_hash = sha256_file(archive_file.path())?;
        if !actual_hash.eq_ignore_ascii_case(expected_hash) {
            return Err(anyhow!(
                "portable Node.js checksum mismatch for {filename}: expected {expected_hash}, got {actual_hash}"
            ));
        }

        let extraction = tempfile::Builder::new()
            .prefix("node-extract-")
            .tempdir_in(&self.root)
            .context("failed to create temporary Node.js extraction directory")?;
        match archive.format {
            ArchiveFormat::TarGz => {
                let file = archive_file
                    .reopen()
                    .context("failed to reopen downloaded Node.js archive")?;
                let decoder = flate2::read::GzDecoder::new(BufReader::new(file));
                let mut archive = tar::Archive::new(decoder);
                archive
                    .unpack(extraction.path())
                    .context("failed to extract portable Node.js tar archive")?;
            }
            ArchiveFormat::Zip => {
                let file = archive_file
                    .reopen()
                    .context("failed to reopen downloaded Node.js archive")?;
                let mut archive =
                    zip::ZipArchive::new(file).context("invalid portable Node.js zip archive")?;
                archive
                    .extract(extraction.path())
                    .context("failed to extract portable Node.js zip archive")?;
            }
        }
        let top_level = archive_top_level(filename, archive.extension)
            .ok_or_else(|| anyhow!("unexpected Node.js archive filename: {filename}"))?;
        let extracted_root = extraction.path().join(top_level);
        if npm_from_node_root(&extracted_root, self.os).is_none() {
            return Err(anyhow!(
                "portable Node.js archive did not contain the expected node/npm layout"
            ));
        }

        let final_root = self.root.join(NODE_CACHE_NAME);
        if let Some(mut resolved) = npm_from_node_root(&final_root, self.os) {
            resolved
                .notices
                .push("portable Node.js was installed by another Banna process".to_owned());
            return Ok(resolved);
        }
        if final_root.exists() {
            fs::remove_dir_all(&final_root).with_context(|| {
                format!(
                    "failed to replace incomplete cache {}",
                    final_root.display()
                )
            })?;
        }
        if let Err(error) = fs::rename(&extracted_root, &final_root)
            && npm_from_node_root(&final_root, self.os).is_none()
        {
            return Err(error).with_context(|| {
                format!(
                    "failed to activate portable Node.js cache {}",
                    final_root.display()
                )
            });
        }
        let mut resolved = npm_from_node_root(&final_root, self.os)
            .ok_or_else(|| anyhow!("portable npm executable is missing after extraction"))?;
        let version = top_level
            .strip_prefix("node-")
            .and_then(|value| value.split('-').next())
            .unwrap_or("LTS");
        resolved.notices.push(format!(
            "npm was not found; downloaded and verified portable Node.js {version} for {}/{} into Banna's private tool cache",
            self.os, self.arch
        ));
        Ok(resolved)
    }
}

fn installation_lock(root: &Path) -> Result<File> {
    let path = root.join("install.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| format!("failed to open toolchain lock {}", path.display()))?;
    file.lock_exclusive()
        .with_context(|| format!("failed to lock toolchain cache {}", root.display()))?;
    Ok(file)
}

fn toolchain_root() -> PathBuf {
    if let Some(path) = env::var_os("BANNA_TOOLCHAIN_DIR") {
        return PathBuf::from(path);
    }
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).map_or_else(
        || PathBuf::from(".banna-toolchains"),
        |directories| directories.data_local_dir().join("toolchains"),
    )
}

fn node_archive(os: &str, arch: &str) -> Result<NodeArchive> {
    let archive = match (os, arch) {
        ("linux", "x86_64") => NodeArchive {
            target: "linux-x64",
            extension: ".tar.gz",
            format: ArchiveFormat::TarGz,
        },
        ("linux", "aarch64") => NodeArchive {
            target: "linux-arm64",
            extension: ".tar.gz",
            format: ArchiveFormat::TarGz,
        },
        ("macos", "x86_64") => NodeArchive {
            target: "darwin-x64",
            extension: ".tar.gz",
            format: ArchiveFormat::TarGz,
        },
        ("macos", "aarch64") => NodeArchive {
            target: "darwin-arm64",
            extension: ".tar.gz",
            format: ArchiveFormat::TarGz,
        },
        ("windows", "x86_64") => NodeArchive {
            target: "win-x64",
            extension: ".zip",
            format: ArchiveFormat::Zip,
        },
        ("windows", "aarch64") => NodeArchive {
            target: "win-arm64",
            extension: ".zip",
            format: ArchiveFormat::Zip,
        },
        _ => {
            return Err(anyhow!(
                "automatic npm bootstrap is not supported on {os}/{arch}; install Node.js 24 LTS from https://nodejs.org/download"
            ));
        }
    };
    Ok(archive)
}

fn select_archive_from_manifest(manifest: &str, archive: NodeArchive) -> Option<(&str, &str)> {
    manifest.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let filename = parts.next()?;
        if parts.next().is_none()
            && filename.starts_with("node-v")
            && filename.ends_with(&format!("-{}{}", archive.target, archive.extension))
            && hash.len() == 64
            && hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            Some((hash, filename))
        } else {
            None
        }
    })
}

fn archive_top_level<'a>(filename: &'a str, extension: &str) -> Option<&'a str> {
    filename
        .strip_suffix(extension)
        .filter(|name| !name.is_empty() && !name.contains(['/', '\\']))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("failed to open archive for checksum: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to checksum {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(encoded)
}

fn npm_from_node_root(root: &Path, os: &str) -> Option<ResolvedProgram> {
    let (node, npm_cli, bin_dir) = if os == "windows" {
        (
            root.join("node.exe"),
            root.join("node_modules/npm/bin/npm-cli.js"),
            root.to_path_buf(),
        )
    } else {
        (
            root.join("bin/node"),
            root.join("lib/node_modules/npm/bin/npm-cli.js"),
            root.join("bin"),
        )
    };
    if !node.is_file() || !npm_cli.is_file() {
        return None;
    }
    let mut env = BTreeMap::new();
    env.insert("PATH".into(), prepend_to_path(&bin_dir).ok()?);
    Some(ResolvedProgram {
        executable: node,
        prefix_args: vec![npm_cli.to_string_lossy().into_owned()],
        env,
        notices: Vec::new(),
    })
}

#[cfg(windows)]
fn system_npm() -> Option<ResolvedProgram> {
    let node = find_on_path("node")?;
    let npm_cli = node.parent()?.join("node_modules/npm/bin/npm-cli.js");
    npm_cli.is_file().then(|| ResolvedProgram {
        executable: node,
        prefix_args: vec![npm_cli.to_string_lossy().into_owned()],
        env: BTreeMap::new(),
        notices: Vec::new(),
    })
}

#[cfg(not(windows))]
fn system_npm() -> Option<ResolvedProgram> {
    find_on_path("npm").map(ResolvedProgram::direct)
}

fn abp_executable(root: &Path) -> PathBuf {
    let name = if cfg!(windows) { "abp.exe" } else { "abp" };
    root.join(name)
}

fn prepend_to_path(directory: &Path) -> Result<String> {
    let mut paths = vec![directory.to_path_buf()];
    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }
    env::join_paths(paths)
        .map(|path| path.to_string_lossy().into_owned())
        .context("failed to construct PATH for portable Node.js")
}

fn find_on_path(program: &str) -> Option<PathBuf> {
    let program_path = Path::new(program);
    if program_path.components().count() > 1 {
        return is_executable(program_path).then(|| program_path.to_path_buf());
    }
    let path = env::var_os("PATH")?;
    for directory in env::split_paths(&path) {
        for candidate_name in executable_names(program) {
            let candidate = directory.join(candidate_name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn executable_names(program: &str) -> Vec<OsString> {
    if cfg!(windows) && Path::new(program).extension().is_none() {
        vec![
            format!("{program}.exe").into(),
            format!("{program}.com").into(),
        ]
    } else {
        vec![program.into()]
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn missing_tool_error(program: &str) -> anyhow::Error {
    let guidance = match program {
        "dotnet" => {
            "Install the .NET SDK version required by the project's global.json from https://dotnet.microsoft.com/download, then restart Banna"
        }
        "npm" => {
            "Banna could not use or bootstrap Node.js 24 LTS; check network access to https://nodejs.org and BANNA_TOOLCHAIN_DIR permissions"
        }
        "abp" => {
            "Banna could not use or install the latest Volo.Abp.Cli; verify the .NET SDK and NuGet connectivity"
        }
        _ => "Install the required executable and make sure it is available on PATH",
    };
    anyhow!("required tool '{program}' was not found on PATH. {guidance}")
}

fn command_diagnostic(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr);
    if !stderr.trim().is_empty() {
        stderr.trim().to_owned()
    } else {
        String::from_utf8_lossy(stdout).trim().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_supported_platform_archives() {
        assert_eq!(
            node_archive("linux", "x86_64")
                .expect("linux archive")
                .target,
            "linux-x64"
        );
        assert_eq!(
            node_archive("macos", "aarch64")
                .expect("macOS archive")
                .target,
            "darwin-arm64"
        );
        assert_eq!(
            node_archive("windows", "x86_64")
                .expect("Windows archive")
                .format,
            ArchiveFormat::Zip
        );
        assert!(node_archive("freebsd", "x86_64").is_err());
    }

    #[test]
    fn selects_only_the_exact_platform_archive_and_hash() {
        let manifest = concat!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  node-v24.18.1-linux-arm64.tar.gz\n",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  node-v24.18.1-linux-x64.tar.gz\n",
        );
        let selected = select_archive_from_manifest(
            manifest,
            node_archive("linux", "x86_64").expect("archive"),
        )
        .expect("matching archive");
        assert_eq!(selected.0, "b".repeat(64));
        assert_eq!(selected.1, "node-v24.18.1-linux-x64.tar.gz");
    }

    #[test]
    fn resolves_a_cached_portable_node_layout_without_network() {
        let fixture = tempfile::tempdir().expect("fixture");
        let node_root = fixture.path().join(NODE_CACHE_NAME);
        let (node, npm_cli) = if cfg!(windows) {
            (
                node_root.join("node.exe"),
                node_root.join("node_modules/npm/bin/npm-cli.js"),
            )
        } else {
            (
                node_root.join("bin/node"),
                node_root.join("lib/node_modules/npm/bin/npm-cli.js"),
            )
        };
        fs::create_dir_all(node.parent().expect("node parent")).expect("node parent");
        fs::create_dir_all(npm_cli.parent().expect("npm parent")).expect("npm parent");
        fs::write(&node, b"node").expect("node");
        fs::write(&npm_cli, b"npm").expect("npm");

        let toolchain = Toolchain::at(
            fixture.path().to_path_buf(),
            env::consts::OS,
            env::consts::ARCH,
        );
        let resolved = toolchain.cached_npm().expect("cached npm");
        assert_eq!(resolved.executable, node);
        assert_eq!(
            resolved.prefix_args,
            [npm_cli.to_string_lossy().into_owned()]
        );
        assert!(resolved.env["PATH"].contains(node_root.to_string_lossy().as_ref()));
    }

    #[test]
    fn resolves_the_cached_unversioned_abp_cli_without_framework_overrides() {
        let fixture = tempfile::tempdir().expect("fixture");
        let node_root = fixture.path().join(NODE_CACHE_NAME);
        let (node, npm_cli) = if cfg!(windows) {
            (
                node_root.join("node.exe"),
                node_root.join("node_modules/npm/bin/npm-cli.js"),
            )
        } else {
            (
                node_root.join("bin/node"),
                node_root.join("lib/node_modules/npm/bin/npm-cli.js"),
            )
        };
        fs::create_dir_all(node.parent().expect("node parent")).expect("node parent");
        fs::create_dir_all(npm_cli.parent().expect("npm parent")).expect("npm parent");
        fs::write(&node, b"node").expect("node");
        fs::write(&npm_cli, b"npm").expect("npm");

        let abp = abp_executable(&fixture.path().join(ABP_CLI_CACHE_NAME));
        fs::create_dir_all(abp.parent().expect("abp parent")).expect("abp parent");
        fs::write(&abp, b"abp").expect("abp");

        let toolchain = Toolchain::at(
            fixture.path().to_path_buf(),
            env::consts::OS,
            env::consts::ARCH,
        );
        let resolved = toolchain.resolve_abp().expect("cached ABP CLI");

        assert_eq!(resolved.executable, abp);
        assert!(!resolved.env.contains_key("DOTNET_ROLL_FORWARD"));
    }

    #[test]
    #[ignore = "downloads and verifies the current official Node.js 24 LTS archive"]
    fn downloads_and_runs_portable_npm() {
        let fixture = tempfile::tempdir().expect("fixture");
        let toolchain = Toolchain::at(
            fixture.path().to_path_buf(),
            env::consts::OS,
            env::consts::ARCH,
        );
        let resolved = toolchain.install_node().expect("portable Node.js");
        let output = Command::new(&resolved.executable)
            .args(&resolved.prefix_args)
            .arg("--version")
            .envs(&resolved.env)
            .stdin(Stdio::null())
            .output()
            .expect("portable npm should start");

        assert!(
            output.status.success(),
            "portable npm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).trim().is_empty());
    }
}
