//! Maintainer release workflow and portable artifact qualification.
//!
//! Source identity, selected dependencies, compiler identities and the verified
//! package travel together in one receipt. Publication checks that evidence
//! before dispatch; it never resolves a replacement lockfile.

use ic_host_tools::artifact::{Sha256Digest, hash_file};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PROBES: [(&str, u64); 5] = [
    ("core", 260_000),
    ("diagnostics", 315_000),
    ("key_only", 260_000),
    ("admission", 264_000),
    ("runtime_integration", 270_000),
];

///
/// ReleaseSelection
///
/// Exact release intent supplied by the common runner.
///
/// Adapters consume this selection without choosing another increment or target.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseSelection {
    kind: String,
    previous: String,
    source: String,
    version: String,
    date: String,
    remote: String,
    branch: String,
}

///
/// ValidationEvidence
///
/// Successful full-gate inputs, saved before release metadata may change.
///
/// The original selected lock bytes permit exact root-version refresh checks
/// and recovery after an interrupted refresh without selecting dependencies.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ValidationEvidence {
    selection: ReleaseSelection,
    lock: Vec<u8>,
    identities: [String; 3],
    configuration: BTreeMap<String, String>,
    command: Vec<String>,
}

///
/// PackageEvidence
///
/// Qualified package bound to the saved validation and current root lockfile.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackageEvidence {
    validation: ValidationEvidence,
    package_head: String,
    lock_sha256: String,
    package_sha256: String,
    commands: Vec<Vec<String>>,
}

///
/// Repository
///
/// Explicit working directory for external commands and release files.
///
/// Command execution is owned here so tests can substitute external effects
/// without creating Git commits, tags, pushes or package publications.
///

struct Repository<E> {
    root: PathBuf,
    exec: E,
}

///
/// Execute
///
/// Process boundary for real commands and deterministic test substitutes.
///

trait Execute {
    fn run(&self, root: &Path, program: &str, args: &[&str], capture: bool) -> Result<String>;
}

///
/// Processes
///
/// Native process execution, preserving argument boundaries and failure status.
///

struct Processes;

impl Execute for Processes {
    fn run(&self, root: &Path, program: &str, args: &[&str], capture: bool) -> Result<String> {
        let mut command = Command::new(program);
        command.current_dir(root).args(args);
        if capture {
            let output = command.output()?;
            require(
                output.status.success(),
                &format!(
                    "{program} {args:?} failed ({}): {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                ),
            )?;
            Ok(String::from_utf8(output.stdout)?)
        } else {
            let status = command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?;
            require(
                status.success(),
                &format!("{program} {args:?} failed ({status})"),
            )?;
            Ok(String::new())
        }
    }
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

impl ReleaseSelection {
    fn from_env() -> Result<Self> {
        let selection = Self {
            kind: env::var("RELEASE_KIND")?,
            previous: env::var("RELEASE_PREVIOUS")?,
            version: env::var("RELEASE_VERSION")?,
            date: env::var("RELEASE_DATE")?,
            source: env::var("RELEASE_SOURCE")?,
            remote: env::var("RELEASE_REMOTE")?,
            branch: env::var("RELEASE_BRANCH")?,
        };
        require(
            matches!(selection.kind.as_str(), "patch" | "minor" | "major"),
            "invalid release kind",
        )?;
        version_parts(&selection.previous)?;
        version_parts(&selection.version)?;
        require(valid_date(&selection.date), "invalid UTC release date")?;
        require(
            valid_commit_identity(&selection.source),
            "invalid source commit identity",
        )?;
        require(
            !selection.remote.is_empty()
                && selection
                    .remote
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)),
            "invalid selected remote",
        )?;
        require(
            !selection.branch.is_empty()
                && !selection.branch.bytes().any(|byte| byte.is_ascii_control()),
            "invalid selected branch",
        )?;
        Ok(selection)
    }
}

fn valid_commit_identity(commit: &str) -> bool {
    matches!(commit.len(), 40 | 64)
        && commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn release_commit_from_env() -> Result<String> {
    let commit = env::var("RELEASE_COMMIT")?;
    require(
        valid_commit_identity(&commit),
        "invalid selected release commit",
    )?;
    Ok(commit)
}

fn valid_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return false;
    }
    let year = date[..4].parse::<u32>().unwrap_or(0);
    let month = date[5..7].parse::<u32>().unwrap_or(0);
    let day = date[8..].parse::<u32>().unwrap_or(0);
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(400) || year.is_multiple_of(4) && !year.is_multiple_of(100) => 29,
        2 => 28,
        _ => 0,
    };
    year != 0 && day != 0 && day <= maximum
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or("metadata has no parent")?;
    fs::create_dir_all(parent)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let temporary = parent.join(format!(
        ".ic-memory-release.{}.{nonce}.tmp",
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        fs::remove_file(temporary)?;
    }
    result
}

fn version_parts(version: &str) -> Result<[u64; 3]> {
    let parts: Vec<_> = version.split('.').collect();
    require(
        parts.len() == 3,
        "expected a stable major.minor.patch version",
    )?;
    let mut numbers = [0; 3];
    for (number, part) in numbers.iter_mut().zip(parts) {
        require(
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part == "0" || !part.starts_with('0')),
            "invalid release version",
        )?;
        *number = part.parse()?;
    }
    Ok(numbers)
}

fn toml_string(text: &str, section: &str, field: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(text)?;
    value
        .get(section)
        .and_then(|value| value.get(field))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing {section}.{field}").into())
}

fn replace_package_version(text: &str, previous: &str, version: &str) -> Result<String> {
    require(
        toml_string(text, "package", "version")? == previous,
        "unexpected package version",
    )?;
    let mut in_package = false;
    let mut count = 0;
    let mut result = String::new();
    for line in text.split_inclusive('\n') {
        if line.trim_start().starts_with('[') {
            in_package = line.trim() == "[package]";
        }
        if in_package
            && line
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "version")
        {
            let start = line
                .find('"')
                .ok_or("package.version must use a quoted string")?
                + 1;
            let end = start
                + line[start..]
                    .find('"')
                    .ok_or("missing version closing quote")?;
            require(
                &line[start..end] == previous,
                "unexpected package version syntax",
            )?;
            result.push_str(&line[..start]);
            result.push_str(version);
            result.push_str(&line[end..]);
            count += 1;
        } else {
            result.push_str(line);
        }
    }
    require(count == 1, "expected one package.version assignment")?;
    Ok(result)
}

fn release_changelog(text: &str, version: &str, date: &str) -> Result<String> {
    require(valid_date(date), "invalid UTC release date")?;
    let headings: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect();
    let first = headings.first().ok_or("missing changelog entry")?;
    require(
        *first == format!("[{version}]"),
        "top numbered pending entry must match the selected version",
    )?;
    require(
        headings
            .iter()
            .filter(|heading| **heading == *first)
            .count()
            == 1,
        "duplicate current changelog entry",
    )?;
    // Historical headings can retain their original bare or dated style.
    // Detect the same release identity without rewriting any of their bytes.
    require(
        !headings.iter().skip(1).any(|heading| {
            *heading == version
                || *heading == format!("[{version}]")
                || heading.starts_with(&format!("[{version}] - "))
        }),
        "selected version already exists",
    )?;
    let heading = format!("## {first}\n");
    let entry = text
        .split_once(&heading)
        .ok_or("invalid changelog heading")?
        .1
        .split("\n## ")
        .next()
        .unwrap_or_default();
    require(!entry.trim().is_empty(), "empty changelog entry")?;
    Ok(text.replacen(&heading, &format!("## [{version}] - {date}\n"), 1))
}

fn check_lock_update(before: &[u8], after: &str, previous: &str, version: &str) -> Result<()> {
    let mut expected: toml::Value = toml::from_str(std::str::from_utf8(before)?)?;
    let packages = expected
        .get_mut("package")
        .and_then(toml::Value::as_array_mut)
        .ok_or("lockfile has no packages")?;
    let mut count = 0;
    for package in packages {
        if package.get("name").and_then(toml::Value::as_str) == Some("ic-memory") {
            require(
                package.get("version").and_then(toml::Value::as_str) == Some(previous),
                "unexpected locked root package version",
            )?;
            package["version"] = toml::Value::String(version.to_owned());
            count += 1;
        }
    }
    require(count == 1, "expected one locked ic-memory package")?;
    require(
        expected == toml::from_str::<toml::Value>(after)?,
        "workspace version refresh changed dependency selection",
    )
}

// Cargo's canonical lockfile places name and version together. Preserve every
// other byte and independently verify that only the root package identity moved.
fn replace_lock_version(text: &str, previous: &str, version: &str) -> Result<String> {
    let identity = format!("name = \"ic-memory\"\nversion = \"{previous}\"");
    require(
        text.matches(&identity).count() == 1,
        "expected one canonical locked root package",
    )?;
    let updated = text.replacen(
        &identity,
        &format!("name = \"ic-memory\"\nversion = \"{version}\""),
        1,
    );
    check_lock_update(text.as_bytes(), &updated, previous, version)?;
    Ok(updated)
}

fn target_directory(metadata: &str) -> Result<PathBuf> {
    let value: serde_json::Value = serde_json::from_str(metadata)?;
    let target = value
        .get("target_directory")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .ok_or("Cargo metadata has no target_directory")?;
    require(
        target.is_absolute(),
        "Cargo target_directory must be absolute",
    )?;
    Ok(target)
}

fn check_wasm_artifacts(target: &Path) -> Result<()> {
    let artifacts = target.join("wasm32-unknown-unknown/wasm-size/examples");
    for (probe, budget) in PROBES {
        let path = artifacts.join(format!("wasm_{probe}_size_probe.wasm"));
        let metadata = fs::metadata(&path)?;
        require(metadata.is_file(), "Wasm artifact is not a regular file")?;
        let bytes = metadata.len();
        println!("{probe} raw Wasm: {bytes} bytes (budget: {budget} bytes)");
        require(
            bytes <= budget,
            &format!("{} exceeds its raw Wasm budget", path.display()),
        )?;
    }
    Ok(())
}

impl<E: Execute> Repository<E> {
    fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        self.exec.run(&self.root, program, args, false)?;
        Ok(())
    }

    fn output(&self, program: &str, args: &[&str]) -> Result<String> {
        self.exec.run(&self.root, program, args, true)
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        Ok(self.output("git", args)?.trim().to_owned())
    }

    fn text(&self, path: &str, revision: Option<&str>) -> Result<String> {
        match revision {
            Some(revision) => self.output("git", &["show", &format!("{revision}:{path}")]),
            None => Ok(fs::read_to_string(self.root.join(path))?),
        }
    }

    fn version(&self, revision: Option<&str>) -> Result<String> {
        let text = self.text("Cargo.toml", revision)?;
        require(
            toml_string(&text, "package", "name")? == "ic-memory",
            "expected the ic-memory package",
        )?;
        let version = toml_string(&text, "package", "version")?;
        version_parts(&version)?;
        Ok(version)
    }

    fn toolchain(&self) -> Result<String> {
        toml_string(
            &self.text("rust-toolchain.toml", None)?,
            "toolchain",
            "channel",
        )
    }

    fn msrv(&self) -> Result<String> {
        toml_string(&self.text("Cargo.toml", None)?, "package", "rust-version")
    }

    fn clean(&self) -> Result<()> {
        require(
            self.git(&["status", "--porcelain", "--untracked-files=all"])?
                .is_empty(),
            "commit source and changelog first; working tree/index must be clean",
        )
    }

    fn target(&self) -> Result<PathBuf> {
        target_directory(&self.output(
            "cargo",
            &[
                "metadata",
                "--locked",
                "--offline",
                "--no-deps",
                "--format-version",
                "1",
            ],
        )?)
    }

    fn receipt_path(&self, version: &str, prepared: bool) -> Result<PathBuf> {
        version_parts(version)?;
        let suffix = if prepared { "-prepared" } else { "" };
        Ok(self
            .target()?
            .join("release-validation")
            .join(format!("{version}{suffix}.json")))
    }

    fn package_path(&self, version: &str) -> Result<PathBuf> {
        version_parts(version)?;
        Ok(self
            .target()?
            .join("package")
            .join(format!("ic-memory-{version}.crate")))
    }

    fn sha256(path: &Path) -> Result<String> {
        // Preserve the existing file-size contract; hashing uses constant memory.
        Ok(hash_file(path, u64::MAX)?.sha256.to_string())
    }

    fn identities(&self) -> Result<[String; 3]> {
        Ok([
            self.output("cargo", &[&format!("+{}", self.toolchain()?), "--version"])?,
            self.output("rustc", &[&format!("+{}", self.toolchain()?), "-Vv"])?,
            self.output("rustc", &[&format!("+{}", self.msrv()?), "-Vv"])?,
        ])
    }

    fn retained_package(&self, digest: &str) -> Result<PathBuf> {
        let digest: Sha256Digest = digest.parse()?;
        Ok(self
            .target()?
            .join("release-validation/artifacts")
            .join(format!("{digest}.crate")))
    }

    fn record_package(&self, version: &str) -> Result<String> {
        let package = self.package_path(version)?;
        let digest = Self::sha256(&package)?;
        let retained = self.retained_package(&digest)?;
        if !retained.exists() {
            fs::create_dir_all(retained.parent().ok_or("archive has no parent")?)?;
            let temporary = retained.with_extension("crate.tmp");
            fs::copy(package, &temporary)?;
            require(
                Self::sha256(&temporary)? == digest,
                "package changed while retaining evidence",
            )?;
            fs::rename(temporary, &retained)?;
        }
        require(
            Self::sha256(&retained)? == digest,
            "retained package differs from its digest",
        )?;
        Ok(digest)
    }

    fn configuration(&self) -> Result<BTreeMap<String, String>> {
        let mut configuration = BTreeMap::new();
        for (name, value) in env::vars_os() {
            let name = name.to_str().ok_or("non-UTF-8 environment variable name")?;
            // The receipts identify the declared toolchains. Cargo compiler
            // replacements or wrappers would execute outside those identities.
            require(
                !matches!(
                    name,
                    "RUSTC"
                        | "RUSTC_WRAPPER"
                        | "RUSTC_WORKSPACE_WRAPPER"
                        | "RUSTDOC"
                        | "CARGO_BUILD_RUSTC"
                        | "CARGO_BUILD_RUSTC_WRAPPER"
                        | "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER"
                        | "CARGO_BUILD_RUSTDOC"
                ),
                &format!("unset {name} before release qualification; use the declared toolchains"),
            )?;
            if matches!(
                name,
                "RUSTFLAGS"
                    | "RUSTDOCFLAGS"
                    | "CARGO_ENCODED_RUSTFLAGS"
                    | "CARGO_ENCODED_RUSTDOCFLAGS"
            ) || name.starts_with("CARGO_BUILD_")
                || name.starts_with("CARGO_PROFILE_")
                || name.starts_with("CARGO_TARGET_") && name != "CARGO_TARGET_DIR"
            {
                configuration.insert(
                    format!("env:{name}"),
                    value
                        .into_string()
                        .map_err(|_| "non-UTF-8 build environment")?,
                );
            }
        }
        // Cargo reads configuration from checkout ancestors and Cargo home.
        // Record digests rather than file contents, which may include secrets.
        let cargo_home = env::var_os("CARGO_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
            .ok_or("set CARGO_HOME or HOME")?;
        let directories = self
            .root
            .ancestors()
            .map(|path| path.join(".cargo"))
            .chain(std::iter::once(cargo_home));
        for directory in directories {
            for name in ["config", "config.toml"] {
                let path = directory.join(name);
                if path.exists() {
                    let value: toml::Value =
                        toml::from_str(&fs::read_to_string(&path)?).map_err(|_| {
                            format!("invalid TOML Cargo configuration: {}", path.display())
                        })?;
                    for field in [
                        "rustc",
                        "rustc-wrapper",
                        "rustc-workspace-wrapper",
                        "rustdoc",
                    ] {
                        require(
                            value
                                .get("build")
                                .and_then(|build| build.get(field))
                                .is_none(),
                            &format!(
                                "remove build.{field} from {} before release qualification; use the declared toolchains",
                                path.display()
                            ),
                        )?;
                    }
                    configuration
                        .insert(format!("config:{}", path.display()), Self::sha256(&path)?);
                }
            }
        }
        Ok(configuration)
    }

    fn check_selection(&self, selection: &ReleaseSelection) -> Result<()> {
        version_parts(&selection.previous)?;
        version_parts(&selection.version)?;
        require(
            valid_date(&selection.date),
            "invalid saved UTC release date",
        )?;
        require(
            self.output(
                "bash",
                &[
                    "scripts/ci/next-release-version.sh",
                    &selection.previous,
                    &selection.kind,
                ],
            )?
            .trim()
                == selection.version,
            "candidate differs from the common version selection",
        )?;
        require(
            self.version(Some(&selection.source))? == selection.previous,
            "saved source version differs from the release selection",
        )
    }

    fn surfaces(&self, selection: &ReleaseSelection) -> Result<BTreeMap<String, String>> {
        self.check_selection(selection)?;
        let source = &selection.source;
        let previous = &selection.previous;
        let version = &selection.version;
        let readme = self.text("README.md", Some(source))?;
        let old = format!("ic-memory = \"{previous}\"");
        require(
            readme.lines().filter(|line| *line == old).count() == 1,
            "README must contain exactly one current dependency example",
        )?;
        let [major, minor, _] = version_parts(version)?;
        let detail = format!("docs/changelog/{major}.{minor}.md");
        Ok(BTreeMap::from([
            (
                "Cargo.lock".to_owned(),
                replace_lock_version(&self.text("Cargo.lock", Some(source))?, previous, version)?,
            ),
            (
                "Cargo.toml".to_owned(),
                replace_package_version(
                    &self.text("Cargo.toml", Some(source))?,
                    previous,
                    version,
                )?,
            ),
            (
                "README.md".to_owned(),
                readme.replacen(&old, &format!("ic-memory = \"{version}\""), 1),
            ),
            (
                "CHANGELOG.md".to_owned(),
                release_changelog(
                    &self.text("CHANGELOG.md", Some(source))?,
                    version,
                    &selection.date,
                )?,
            ),
            (
                detail.clone(),
                release_changelog(&self.text(&detail, Some(source))?, version, &selection.date)?,
            ),
        ]))
    }

    fn check_surfaces(
        &self,
        selection: &ReleaseSelection,
        revision: Option<&str>,
        partial: bool,
    ) -> Result<()> {
        let mut expected_changes = Vec::new();
        for (path, expected) in self.surfaces(selection)? {
            let original = self.text(&path, Some(&selection.source))?;
            let actual = self.text(&path, revision)?;
            require(
                actual == expected || partial && actual == original,
                &format!("{path} differs from the saved release edit"),
            )?;
            if actual != original {
                expected_changes.push(path);
            }
        }
        let mut args = vec!["diff", "--name-only", &selection.source];
        if let Some(revision) = revision {
            args.push(revision);
        }
        args.push("--");
        require(
            self.git(&args)?
                .lines()
                .eq(expected_changes.iter().map(String::as_str)),
            "release candidate contains changes outside its version surfaces",
        )?;
        require(
            self.git(&["ls-files", "--others", "--exclude-standard"])?
                .is_empty(),
            "release candidate has untracked files",
        )
    }

    fn check_index(
        &self,
        selection: &ReleaseSelection,
        surfaces: &BTreeMap<String, String>,
    ) -> Result<()> {
        for path in self
            .git(&["diff", "--cached", "--name-only", &selection.source, "--"])?
            .lines()
        {
            let expected = surfaces
                .get(path)
                .ok_or("index contains unrelated release changes")?;
            require(
                self.git(&[
                    "diff",
                    "--cached",
                    "--summary",
                    &selection.source,
                    "--",
                    path,
                ])?
                .is_empty(),
                "staged release metadata changes file mode or identity",
            )?;
            let actual = self.output("git", &["show", &format!(":{path}")])?;
            require(
                actual == *expected || actual == self.text(path, Some(&selection.source))?,
                "staged release metadata differs from saved intent",
            )?;
        }
        Ok(())
    }
    fn local_tag(&self, version: &str) -> Result<Option<String>> {
        let refs = self.git(&[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/tags",
        ])?;
        let name = format!("refs/tags/v{version}");
        Ok(refs
            .lines()
            .filter_map(|line| line.split_once(' '))
            .find(|(_, reference)| *reference == name)
            .map(|(hash, _)| hash.to_owned()))
    }

    fn remote_ready(&self, selection: &ReleaseSelection) -> Result<()> {
        require(
            self.git(&["symbolic-ref", "--quiet", "--short", "HEAD"])? == selection.branch,
            "selected release branch is not checked out",
        )?;
        let remote_ref = format!("refs/remotes/{}/{}", selection.remote, selection.branch);
        self.git(&[
            "fetch",
            "--quiet",
            "--no-tags",
            &selection.remote,
            &format!("+refs/heads/{}:{remote_ref}", selection.branch),
        ])?;
        self.git(&["merge-base", "--is-ancestor", &remote_ref, "HEAD"])?;
        Ok(())
    }
    fn validation_path(&self, version: &str) -> Result<PathBuf> {
        version_parts(version)?;
        Ok(self
            .target()?
            .join("release-validation")
            .join(format!("{version}-validated.json")))
    }

    fn validation_command(&self) -> Result<Vec<String>> {
        Ok(vec![
            "make".to_owned(),
            "--no-print-directory".to_owned(),
            "validate".to_owned(),
            format!("VALIDATION_TOOLCHAIN={}", self.toolchain()?),
        ])
    }

    fn run_command(&self, command: &[String]) -> Result<()> {
        self.run(
            &command[0],
            &command[1..].iter().map(String::as_str).collect::<Vec<_>>(),
        )
    }

    fn preflight(&self, selection: &ReleaseSelection) -> Result<()> {
        self.configuration()?;
        self.check_selection(selection)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source HEAD differs from the saved release",
        )?;
        require(
            self.version(None)? == selection.previous,
            "preflight requires the saved base version",
        )?;
        self.check_surfaces(selection, None, true)?;
        self.check_index(selection, &self.surfaces(selection)?)?;
        let lock = fs::read(self.root.join("Cargo.lock"))?;
        let interrupted = lock != self.text("Cargo.lock", Some(&selection.source))?.as_bytes();
        if interrupted {
            // The exact candidate lock can precede the manifest after an
            // interrupted write. Reuse source-bound successful validation;
            // locked Cargo fetching cannot reconcile that temporary mismatch.
            self.validated(selection)?;
        } else {
            check_lock_update(
                &lock,
                std::str::from_utf8(&lock)?,
                &selection.previous,
                &selection.previous,
            )?;
        }
        self.remote_ready(selection)?;
        // Remote inspection and offline cache verification cannot change the
        // tracked lockfile whose selection will qualify this attempt.
        if !interrupted {
            self.run(
                "cargo",
                &[
                    &format!("+{}", self.toolchain()?),
                    "fetch",
                    "--locked",
                    "--offline",
                ],
            )?;
        }
        require(
            fs::read(self.root.join("Cargo.lock"))? == lock,
            "dependency selection changed during preflight",
        )?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during preflight",
        )?;
        self.check_surfaces(selection, None, true)?;
        self.check_index(selection, &self.surfaces(selection)?)
    }

    fn verify(&self, selection: &ReleaseSelection) -> Result<()> {
        self.configuration()?;
        self.check_selection(selection)?;
        self.clean()?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source
                && self.version(None)? == selection.previous,
            "validation requires the saved source and base version",
        )?;
        let lock = fs::read(self.root.join("Cargo.lock"))?;
        check_lock_update(
            &lock,
            std::str::from_utf8(&lock)?,
            &selection.previous,
            &selection.previous,
        )?;
        let evidence = ValidationEvidence {
            selection: selection.clone(),
            lock,
            identities: self.identities()?,
            configuration: self.configuration()?,
            command: self.validation_command()?,
        };
        self.run_command(&evidence.command)?;
        self.clean()?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during validation",
        )?;
        self.verify_inputs(&evidence, false)?;
        let path = self.validation_path(&selection.version)?;
        if path.exists() {
            let retained = path
                .parent()
                .ok_or("validation path has no parent")?
                .join("attempts")
                .join(format!(
                    "{}-validated.{}.{}.json",
                    selection.version,
                    std::process::id(),
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
                ));
            write_atomic(&retained, &fs::read(&path)?)?;
        }
        write_atomic(&path, &serde_json::to_vec_pretty(&evidence)?)
    }

    fn verify_inputs(&self, evidence: &ValidationEvidence, updated: bool) -> Result<()> {
        require(
            evidence.lock
                == self
                    .text("Cargo.lock", Some(&evidence.selection.source))?
                    .as_bytes(),
            "validation lockfile differs from the saved source",
        )?;
        require(
            self.configuration()? == evidence.configuration,
            "build configuration differs from release validation",
        )?;
        require(
            self.identities()? == evidence.identities,
            "compiler identities differ from release validation",
        )?;
        require(
            evidence.command == self.validation_command()?,
            "validation evidence has unexpected commands",
        )?;
        self.check_selection(&evidence.selection)?;
        let current = fs::read(self.root.join("Cargo.lock"))?;
        if current != evidence.lock {
            require(updated, "selected lockfile differs from release validation")?;
            check_lock_update(
                &evidence.lock,
                std::str::from_utf8(&current)?,
                &evidence.selection.previous,
                &evidence.selection.version,
            )?;
        }
        Ok(())
    }

    fn validated(&self, selection: &ReleaseSelection) -> Result<ValidationEvidence> {
        let evidence: ValidationEvidence =
            serde_json::from_slice(&fs::read(self.validation_path(&selection.version)?)?)?;
        require(
            evidence.selection == *selection,
            "validation identifies another release intent",
        )?;
        self.verify_inputs(&evidence, true)?;
        Ok(evidence)
    }

    fn preparation_commands(&self) -> Result<Vec<Vec<String>>> {
        let pin = format!("+{}", self.toolchain()?);
        Ok(vec![
            vec![
                "cargo".to_owned(),
                pin.clone(),
                "update".to_owned(),
                "--workspace".to_owned(),
                "--offline".to_owned(),
            ],
            vec![
                "make".to_owned(),
                "--no-print-directory".to_owned(),
                "fmt-check".to_owned(),
                format!("VALIDATION_TOOLCHAIN={}", self.toolchain()?),
            ],
            vec![
                "cargo".to_owned(),
                pin,
                "package".to_owned(),
                "--locked".to_owned(),
                "--offline".to_owned(),
                "--allow-dirty".to_owned(),
            ],
        ])
    }

    fn final_package_command(&self) -> Result<Vec<String>> {
        Ok(vec![
            "cargo".to_owned(),
            format!("+{}", self.toolchain()?),
            "package".to_owned(),
            "--locked".to_owned(),
            "--offline".to_owned(),
        ])
    }

    fn write_evidence(&self, evidence: &PackageEvidence, prepared: bool) -> Result<()> {
        write_atomic(
            &self.receipt_path(&evidence.validation.selection.version, prepared)?,
            &serde_json::to_vec_pretty(evidence)?,
        )
    }

    fn prepare_selected(&self, selection: &ReleaseSelection) -> Result<()> {
        self.validated(selection)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed before preparation",
        )?;
        self.check_surfaces(selection, None, true)?;
        let expected = self.surfaces(selection)?;
        let mut backups = BTreeMap::new();
        for path in expected.keys() {
            backups.insert(path.clone(), fs::read(self.root.join(path))?);
        }
        let result = (|| -> Result<()> {
            // Cargo.toml is last: observing the candidate means all other source
            // metadata, including the selected root lock, is complete. The
            // prepared check can then finish offline workspace verification and
            // package qualification after an interrupted operation.
            for (path, text) in expected
                .iter()
                .filter(|(path, _)| path.as_str() != "Cargo.toml")
            {
                write_atomic(&self.root.join(path), text.as_bytes())?;
            }
            write_atomic(
                &self.root.join("Cargo.toml"),
                expected["Cargo.toml"].as_bytes(),
            )?;
            self.prepared(selection)?;
            Ok(())
        })();
        if result.is_err() {
            for (path, bytes) in backups {
                let current = fs::read(self.root.join(&path))?;
                let owned = if path == "Cargo.lock" {
                    current == bytes
                        || std::str::from_utf8(&current).is_ok_and(|text| {
                            check_lock_update(&bytes, text, &selection.previous, &selection.version)
                                .is_ok()
                        })
                } else {
                    current == expected[&path].as_bytes() || current == bytes
                };
                if owned {
                    write_atomic(&self.root.join(path), &bytes)?;
                }
            }
        }
        result
    }

    fn verify_evidence(
        &self,
        selection: &ReleaseSelection,
        head: &str,
        prepared: bool,
        working: bool,
    ) -> Result<PackageEvidence> {
        let evidence: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&selection.version, prepared)?)?)?;
        require(
            evidence.validation.selection == *selection,
            "package evidence identifies another release intent",
        )?;
        self.verify_inputs(&evidence.validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "selected lockfile differs from package qualification",
        )?;
        require(
            evidence.package_head == head,
            "package evidence identifies another Git HEAD",
        )?;
        require(
            Self::sha256(&self.retained_package(&evidence.package_sha256)?)?
                == evidence.package_sha256,
            "retained archive differs from release validation",
        )?;
        if working {
            require(
                Self::sha256(&self.package_path(&selection.version)?)? == evidence.package_sha256,
                "working package differs from release validation",
            )?;
        }
        let mut commands = self.preparation_commands()?;
        if !prepared {
            commands.push(self.final_package_command()?);
        }
        require(
            evidence.commands == commands,
            "package evidence has unexpected commands",
        )?;
        Ok(evidence)
    }

    fn prepared(&self, selection: &ReleaseSelection) -> Result<PackageEvidence> {
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "prepared source changed",
        )?;
        let validation = self.validated(selection)?;
        self.check_surfaces(selection, None, false)?;
        if self.receipt_path(&selection.version, true)?.exists() {
            // Existing evidence is checked, never silently repaired or replaced.
            return self.verify_evidence(selection, &selection.source, true, true);
        }
        let commands = self.preparation_commands()?;
        self.run_command(&commands[0])?;
        check_lock_update(
            &validation.lock,
            &fs::read_to_string(self.root.join("Cargo.lock"))?,
            &selection.previous,
            &selection.version,
        )?;
        self.run_command(&commands[1])?;
        let lock_sha256 = Self::sha256(&self.root.join("Cargo.lock"))?;
        self.run_command(&commands[2])?;
        self.check_surfaces(selection, None, false)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during preparation",
        )?;
        self.verify_inputs(&validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == lock_sha256,
            "packaging changed selected dependencies",
        )?;
        let evidence = PackageEvidence {
            validation,
            package_head: selection.source.clone(),
            lock_sha256,
            package_sha256: self.record_package(&selection.version)?,
            commands,
        };
        self.write_evidence(&evidence, true)?;
        Ok(evidence)
    }

    fn commit_check(&self, selection: &ReleaseSelection) -> Result<()> {
        self.prepared(selection)?;
        self.check_index(selection, &self.surfaces(selection)?)?;
        require(
            self.git(&["diff", "--name-only"])?.is_empty(),
            "release metadata has unstaged changes",
        )?;
        require(
            self.git(&["diff", "--cached", "--name-only"])?
                == self.git(&["diff", "--name-only", &selection.source])?,
            "stage exactly the release metadata",
        )
    }

    fn inspect_release_commit(&self, commit: &str) -> Result<(String, String, ReleaseSelection)> {
        self.clean()?;
        let version = self.version(Some(commit))?;
        let prepared: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&version, true)?)?)?;
        let selection = prepared.validation.selection;
        require(
            selection.version == version,
            "release metadata differs from saved intent",
        )?;
        require(
            self.git(&["log", "-1", "--format=%P", commit])? == selection.source,
            "release parent differs from validated source",
        )?;
        require(
            self.git(&["log", "-1", "--format=%s", commit])? == format!("Release {version}"),
            "selected commit is not the release commit",
        )?;
        self.git(&["merge-base", "--is-ancestor", commit, "HEAD"])?;
        self.check_surfaces(&selection, Some(commit), false)?;
        Ok((version, commit.to_owned(), selection))
    }

    fn release_commit(&self) -> Result<(String, String, ReleaseSelection)> {
        let release = self.inspect_release_commit(&self.git(&["rev-parse", "HEAD"])?)?;
        self.verify_evidence(&release.2, &release.1, false, true)?;
        Ok(release)
    }

    fn qualify_release(&self) -> Result<()> {
        self.qualify_release_at(&self.git(&["rev-parse", "HEAD"])?)
    }

    fn qualify_release_at(&self, commit: &str) -> Result<()> {
        let (version, head, selection) = self.inspect_release_commit(commit)?;
        let mut evidence = self.verify_evidence(&selection, &selection.source, true, false)?;
        let current = self.git(&["rev-parse", "HEAD"])? == commit;
        if self.receipt_path(&version, false)?.exists() {
            // Recovery verifies the retained archive for the selected commit.
            // Cargo's working archive may belong to the newer source at HEAD.
            self.verify_evidence(&selection, &head, false, current)?;
            return Ok(());
        }
        require(
            current,
            "selected older release has no final qualification receipt; qualify that exact commit before recovery",
        )?;
        let command = self.final_package_command()?;
        self.run_command(&command)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == commit
                && self.inspect_release_commit(commit)?
                    == (version.clone(), head.clone(), selection),
            "release changed during final qualification",
        )?;
        self.verify_inputs(&evidence.validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "final packaging changed selected dependencies",
        )?;
        evidence.package_head = head;
        evidence.package_sha256 = self.record_package(&version)?;
        evidence.commands.push(command);
        self.write_evidence(&evidence, false)
    }

    fn selected_committed_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<()> {
        require(
            self.inspect_release_commit(commit)?.2 == *selection,
            "commit identifies another release intent",
        )?;
        self.qualify_release_at(commit)
    }

    fn tagged_check(
        &self,
        selection: &ReleaseSelection,
        commit: &str,
    ) -> Result<(String, String, ReleaseSelection)> {
        let release = self.inspect_release_commit(commit)?;
        require(
            release.2 == *selection,
            "tagged release identifies another intent",
        )?;
        self.verify_evidence(
            selection,
            commit,
            false,
            self.git(&["rev-parse", "HEAD"])? == commit,
        )?;
        self.check_tag(&release.0, &release.1)?;
        Ok(release)
    }

    fn push_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<()> {
        let release = self.tagged_check(selection, commit)?;
        self.remote_ready(selection)?;
        require(
            self.tagged_check(selection, commit)? == release,
            "release changed during remote inspection",
        )?;
        Ok(())
    }
    fn check_tag(&self, version: &str, head: &str) -> Result<()> {
        require(self.local_tag(version)?.is_some(), "release tag is missing")?;
        require(
            self.git(&["cat-file", "-t", &format!("refs/tags/v{version}")])? == "tag",
            "release tag must be annotated",
        )?;
        require(
            self.git(&["rev-parse", &format!("refs/tags/v{version}^{{commit}}")])? == head,
            "release tag does not identify the selected commit",
        )
    }

    fn publish(&self, dry_run: bool) -> Result<()> {
        let (version, head, source) = self.release_commit()?;
        self.check_tag(&version, &head)?;
        let mut args = vec![
            format!("+{}", self.toolchain()?),
            "publish".to_owned(),
            "--locked".to_owned(),
            "--registry".to_owned(),
            "crates-io".to_owned(),
        ];
        if dry_run {
            args.push("--dry-run".to_owned());
        }
        require(
            self.release_commit()? == (version, head, source),
            "release changed before publication",
        )?;
        // Cargo uploads the crate from the package directory. Evidence is checked
        // before dispatch and rechecked afterwards so a rebuild cannot replace it
        // silently. Publication itself is a network effect, never an offline retry.
        self.run(
            "cargo",
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        self.release_commit()?;
        Ok(())
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "only the binary entrypoint calls this module"
)]
pub(super) fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    require(
        !args.is_empty() && args.len() <= 2,
        "usage: repo-tool <version|msrv|toolchain|target|ensure-clean|wasm-size|release-preflight|release-verify|release-prepare-version|release-prepared-check|release-files|release-commit-check|release-committed-check|release-tagged-check|release-push-check|qualify-release|publish> [--dry-run]",
    )?;
    let command = args[0].as_str();
    require(
        args.len() == 1 || (command == "publish" && args[1] == "--dry-run"),
        "--dry-run applies only to publish",
    )?;
    let repo = Repository {
        root: env::current_dir()?,
        exec: Processes,
    };
    match command {
        "version" => println!("{}", repo.version(None)?),
        "msrv" => println!("{}", repo.msrv()?),
        "toolchain" => println!("{}", repo.toolchain()?),
        "target" => println!("{}", repo.target()?.display()),
        "ensure-clean" => repo.clean()?,
        "wasm-size" => check_wasm_artifacts(&repo.target()?)?,
        "release-preflight" => repo.preflight(&ReleaseSelection::from_env()?)?,
        "release-verify" => repo.verify(&ReleaseSelection::from_env()?)?,
        "release-prepare-version" => repo.prepare_selected(&ReleaseSelection::from_env()?)?,
        "release-prepared-check" => {
            repo.prepared(&ReleaseSelection::from_env()?)?;
        }
        "release-files" => {
            for path in repo.surfaces(&ReleaseSelection::from_env()?)?.keys() {
                print!("{path}\0");
            }
        }
        "release-commit-check" => repo.commit_check(&ReleaseSelection::from_env()?)?,
        "release-committed-check" => {
            repo.selected_committed_check(
                &ReleaseSelection::from_env()?,
                &release_commit_from_env()?,
            )?;
        }
        "release-tagged-check" => {
            repo.tagged_check(&ReleaseSelection::from_env()?, &release_commit_from_env()?)?;
        }
        "release-push-check" => {
            repo.push_check(&ReleaseSelection::from_env()?, &release_commit_from_env()?)?;
        }
        "qualify-release" => repo.qualify_release()?,
        "publish" => {
            let flag = match env::var("PUBLISH_DRY_RUN") {
                Ok(flag) => flag,
                Err(env::VarError::NotPresent) => "0".to_owned(),
                Err(error) => return Err(error.into()),
            };
            require(flag == "0" || flag == "1", "PUBLISH_DRY_RUN must be 0 or 1")?;
            repo.publish(args.len() == 2 || flag == "1")?;
        }
        _ => return Err("unknown repo-tool command".into()),
    }
    Ok(())
}
