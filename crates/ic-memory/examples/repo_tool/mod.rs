//! Maintainer release workflow and portable artifact qualification.
//!
//! Source identity, selected dependencies, compiler identities and the verified
//! package travel together in one receipt. Publication checks that evidence
//! before dispatch; it never resolves a replacement lockfile.

use ic_build::cargo::{
    environment::{
        configuration_directories, is_build_variable, is_compiler_override, observe_configurations,
    },
    lock_projection::{WorkspaceVersionProjection, verify_workspace_version_projection},
    observation::schema::{Metadata, MetadataCommand},
};
use ic_host_artifacts::artifact::{Sha256Digest, copy_reader};
use ic_host_fs::durable::{PublicationMode, WriteOptions};
use ic_host_fs::read::{hash_file, open_regular_file};
use ic_host_process::tool::{
    CompletedExecution, OutputLimit, OutputLimits, ToolError, capture_command,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    io::{BufWriter, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

///
/// PreparationError
///
/// Original preparation failure together with failed metadata restorations.
///
/// Successful restorations and retained evidence remain available for recovery.
///

#[derive(Debug)]
struct PreparationError {
    source: Box<dyn Error>,
    rollback_errors: Vec<(String, Box<dyn Error>)>,
}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.source)?;
        for (path, error) in &self.rollback_errors {
            write!(formatter, "; could not restore {path}: {error}")?;
        }
        Ok(())
    }
}

impl Error for PreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

const PACKAGE_MANIFEST: &str = "crates/ic-memory/Cargo.toml";
const PACKAGE_README: &str = "crates/ic-memory/README.md";
const CHANGELOG_FINALIZER: &str = "scripts/ci/finalize-release-changelog.awk";
const LOCK_REWRITER: &str = "scripts/ci/rewrite-local-lock-versions.pl";
const WORKSPACE_VERSION_RENDERER: &str = "scripts/ci/render-cargo-workspace-version.pl";
const RELEASE_INPUT_ADMISSION: &str = "scripts/ci/validate-release-inputs.pl";
const TAG_CHECKER: &str = "scripts/ci/check-release-tag.sh";
const REPLACE_FILE_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::Replace,
    permissions: 0o666,
};
const PROBES: [(&str, u64); 4] = [
    ("core", 260_000),
    ("diagnostics", 315_000),
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
/// Process and file-publication boundaries for real effects and test substitutes.
///

trait Execute {
    fn run(&self, root: &Path, program: &str, args: &[&str], capture: bool) -> Result<String>;

    fn cargo_target(&self, root: &Path) -> Result<PathBuf>;

    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(ic_host_fs::durable::write_bytes(
            path,
            REPLACE_FILE_OPTIONS,
            bytes,
        )?)
    }

    fn write_json(&self, path: &Path, value: &impl Serialize) -> Result<()> {
        // Keep receipt encoding local; the shared engine owns staging and
        // publication, including original producer and cleanup failures.
        Ok(ic_host_fs::durable::write_with(
            path,
            REPLACE_FILE_OPTIONS,
            |file| {
                let mut writer = BufWriter::new(file);
                serde_json::to_writer_pretty(&mut writer, value)?;
                // Flush the fixed-size buffer before the engine may publish.
                writer.flush().map_err(serde_json::Error::io)
            },
        )?)
    }

    fn write_with(
        &self,
        path: &Path,
        write: impl FnOnce(&mut fs::File) -> std::io::Result<()>,
    ) -> Result<()> {
        Ok(ic_host_fs::durable::write_with(
            path,
            REPLACE_FILE_OPTIONS,
            write,
        )?)
    }
}

///
/// Processes
///
/// Native process execution, preserving argument boundaries and failure status.
///

struct Processes;

impl Execute for Processes {
    fn cargo_target(&self, root: &Path) -> Result<PathBuf> {
        // Cargo owns command construction and its schema. Execute with this
        // consumer's Host so no process types cross IC Build's Host boundary.
        let mut command = MetadataCommand::new()
            .cargo_path("cargo")
            .manifest_path(root.join("Cargo.toml"))
            .current_dir(root)
            .no_deps()
            .other_options(vec!["--locked".into(), "--offline".into()])
            .cargo_command();
        let output = capture_command(
            &mut command,
            OutputLimits {
                stdout: OutputLimit::Terminate(4 * 1024 * 1024),
                stderr: OutputLimit::Terminate(64 * 1024),
                timeout: None,
            },
        )
        .and_then(CompletedExecution::require_success)
        .and_then(CompletedExecution::require_complete)
        .inspect_err(|error| report_tool_error("cargo", error))?;
        let metadata: Metadata = serde_json::from_slice(&output.stdout)?;
        Ok(metadata.target_directory.into_std_path_buf())
    }

    fn run(&self, root: &Path, program: &str, args: &[&str], capture: bool) -> Result<String> {
        let mut command = Command::new(program);
        command.current_dir(root).args(args);
        if capture {
            // Preserve complete output and the existing absence of a deadline.
            // Host owns fair pipe draining, failure evidence and child cleanup.
            let output = capture_command(
                &mut command,
                OutputLimits {
                    stdout: OutputLimit::Terminate(usize::MAX),
                    stderr: OutputLimit::Terminate(usize::MAX),
                    timeout: None,
                },
            )
            .and_then(CompletedExecution::require_success)
            .and_then(CompletedExecution::require_complete)
            .inspect_err(|error| report_tool_error(program, error))?;
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
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

fn report_tool_error(program: &str, error: &ToolError) {
    // Host formatting excludes bytes; preserve stderr before returning its cause.
    if let ToolError::Execution(failure) = error {
        if let Some(status) = failure.evidence.status {
            eprintln!("{program} failed ({status})");
        }
        eprint!("{}", String::from_utf8_lossy(&failure.evidence.stderr));
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
    fn from_env<E: Execute>(repo: &Repository<E>) -> Result<Self> {
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
        repo.version_parts(&selection.previous)?;
        repo.version_parts(&selection.version)?;
        repo.validate_date(&selection.date)?;
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

fn toml_string(text: &str, section: &str, field: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(text)?;
    section
        .split('.')
        .try_fold(&value, |value, key| value.get(key))
        .and_then(|value| value.get(field))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing {section}.{field}").into())
}

fn check_lock_update(before: &[u8], after: &str, previous: &str, version: &str) -> Result<()> {
    verify_workspace_version_projection(
        std::str::from_utf8(before)?,
        after,
        &[WorkspaceVersionProjection {
            package: "ic-memory",
            previous,
            next: version,
        }],
    )?;
    Ok(())
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
    fn validate_date(&self, date: &str) -> Result<()> {
        require(
            self.output("perl", &[RELEASE_INPUT_ADMISSION, "date", date])? == format!("{date}\n"),
            "release date admission returned unexpected output",
        )
    }

    fn version_parts(&self, version: &str) -> Result<[u64; 3]> {
        let output = self.output("perl", &[RELEASE_INPUT_ADMISSION, "version", version])?;
        let mut parts = output.trim_end_matches('\n').split('\t');
        let mut numbers = [0; 3];
        for number in &mut numbers {
            *number = parts
                .next()
                .ok_or("release version admission omitted a component")?
                .parse()?;
        }
        require(
            parts.next().is_none(),
            "release version admission returned extra components",
        )?;
        Ok(numbers)
    }

    // Keep exact helper inputs for retries and diagnosis. Both transforms use
    // Host capture instead of writing to one pipe before draining the others.
    fn transform_text(
        &self,
        program: &str,
        prefix: &[&str],
        text: &str,
        suffix: &[&str],
    ) -> Result<String> {
        let digest = Sha256Digest::compute(text.as_bytes());
        let input = self
            .target()?
            .join("release-validation/inputs")
            .join(format!("{digest}.input"));
        match fs::symlink_metadata(&input) {
            Ok(metadata) => require(metadata.is_file(), "transform input is not a regular file")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ic_host_fs::durable::write_with(
                    &input,
                    WriteOptions {
                        mode: PublicationMode::CreateNew,
                        ..REPLACE_FILE_OPTIONS
                    },
                    |file| file.write_all(text.as_bytes()),
                )?;
            }
            Err(error) => return Err(error.into()),
        }
        let limit = u64::try_from(text.len())?;
        require(
            hash_file(&input, limit)?.sha256 == digest,
            "transform input changed",
        )?;
        let arguments = prefix
            .iter()
            .copied()
            .chain(std::iter::once(
                input.to_str().ok_or("non-UTF-8 transform input")?,
            ))
            .chain(suffix.iter().copied())
            .collect::<Vec<_>>();
        let output = self.output(program, &arguments)?;
        require(
            hash_file(&input, limit)?.sha256 == digest,
            "transform input changed",
        )?;
        Ok(output)
    }

    fn replace_workspace_version(
        &self,
        text: &str,
        previous: &str,
        version: &str,
    ) -> Result<String> {
        self.version_parts(previous)?;
        self.version_parts(version)?;
        let mut expected: toml::Value = toml::from_str(text)?;
        require(
            expected
                .get("workspace")
                .and_then(|value| value.get("package"))
                .and_then(|value| value.get("version"))
                .and_then(toml::Value::as_str)
                == Some(previous),
            "unexpected package version",
        )?;
        expected["workspace"]["package"]["version"] = toml::Value::String(version.to_owned());
        let updated = self.transform_text(
            "perl",
            &[WORKSPACE_VERSION_RENDERER],
            text,
            &[previous, version],
        )?;
        require(
            expected == toml::from_str::<toml::Value>(&updated)?,
            "workspace version rendering changed other metadata",
        )?;
        Ok(updated)
    }

    fn replace_lock_version(&self, text: &str, previous: &str, version: &str) -> Result<String> {
        let updated = self.transform_text(
            "perl",
            &[LOCK_REWRITER],
            text,
            &[previous, version, "ic-memory"],
        )?;
        check_lock_update(text.as_bytes(), &updated, previous, version)?;
        Ok(updated)
    }

    fn release_changelog(
        &self,
        finalizer: &Path,
        text: &str,
        previous: &str,
        version: &str,
        date: &str,
    ) -> Result<String> {
        self.version_parts(previous)?;
        self.version_parts(version)?;
        self.validate_date(date)?;
        // Published history includes bare version headings. Keep that refusal;
        // the shared helper owns candidate classification and finalization.
        require(
            !text.lines().any(|line| {
                let mut words = line.split_whitespace();
                words.next() == Some("##") && words.next() == Some(version)
            }),
            "selected version already exists",
        )?;
        self.transform_text(
            "awk",
            &[
                "-v",
                &format!("version={version}"),
                "-v",
                &format!("previous={previous}"),
                "-v",
                &format!("date={date}"),
                "-f",
                finalizer.to_str().ok_or("non-UTF-8 changelog finalizer")?,
            ],
            text,
            &[],
        )
    }

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
        let package_text = self.text(PACKAGE_MANIFEST, revision)?;
        let package: toml::Value = toml::from_str(&package_text)?;
        require(
            toml_string(&package_text, "package", "name")? == "ic-memory"
                && package
                    .get("package")
                    .and_then(|value| value.get("version"))
                    .and_then(|value| value.get("workspace"))
                    .and_then(toml::Value::as_bool)
                    == Some(true),
            "expected the ic-memory package with an inherited workspace version",
        )?;
        let version = toml_string(&text, "workspace.package", "version")?;
        self.version_parts(&version)?;
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
        toml_string(
            &self.text("Cargo.toml", None)?,
            "workspace.package",
            "rust-version",
        )
    }

    fn clean(&self) -> Result<()> {
        let status = self.output(
            "git",
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )?;
        require(
            status.trim().is_empty(),
            &format!(
                "working tree/index must be clean; review these Git status entries:\n{}",
                status.trim_end()
            ),
        )
    }

    fn target(&self) -> Result<PathBuf> {
        let target = self.exec.cargo_target(&self.root)?;
        require(
            target.is_absolute(),
            "Cargo target_directory must be absolute",
        )?;
        Ok(target)
    }

    fn receipt_path(&self, version: &str, prepared: bool) -> Result<PathBuf> {
        self.version_parts(version)?;
        let suffix = if prepared { "-prepared" } else { "" };
        Ok(self
            .target()?
            .join("release-validation")
            .join(format!("{version}{suffix}.json")))
    }

    fn package_path(&self, version: &str) -> Result<PathBuf> {
        self.version_parts(version)?;
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
            self.exec.write_with(&retained, |staging| {
                let source = open_regular_file(&package).map_err(std::io::Error::from)?;
                let copied = copy_reader(source, staging, u64::MAX)?;
                if copied.sha256.to_string() != digest {
                    return Err(std::io::Error::other(
                        "package changed while retaining evidence",
                    ));
                }
                Ok(())
            })?;
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
                !is_compiler_override(name),
                &format!("unset {name} before release qualification; use the declared toolchains"),
            )?;
            if is_build_variable(name) {
                configuration.insert(
                    format!("env:{name}"),
                    value
                        .into_string()
                        .map_err(|_| "non-UTF-8 build environment")?,
                );
            }
        }
        let cargo_home = env::var_os("CARGO_HOME");
        let home = env::var_os("HOME");
        require(
            cargo_home.is_some() || home.is_some(),
            "set CARGO_HOME or HOME",
        )?;
        // Keep conservative custody of both root names, even when Cargo shadows
        // one. Build owns discovery, include traversal and same-read raw hashes.
        let roots: Vec<_> =
            configuration_directories(&self.root, cargo_home.as_deref(), home.as_deref())?
                .iter()
                .flat_map(|directory| [directory.join("config"), directory.join("config.toml")])
                .filter(|path| path.exists())
                .collect();
        for file in observe_configurations(&roots)? {
            if let Some(field) = file.compiler_overrides.first() {
                return Err(format!(
                    "remove build.{field} from {} before release qualification; use the declared toolchains",
                    file.path.display()
                ).into());
            }
            // Contents may contain secrets; receipts retain only lookup labels
            // and the hash of the exact bytes parsed by the observer.
            configuration.insert(
                format!("config:{}", file.path.display()),
                Sha256Digest::from_bytes(file.sha256).to_string(),
            );
        }
        Ok(configuration)
    }

    fn check_selection(&self, selection: &ReleaseSelection) -> Result<()> {
        self.version_parts(&selection.previous)?;
        self.version_parts(&selection.version)?;
        self.validate_date(&selection.date)?;
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
        let readme = self.text(PACKAGE_README, Some(source))?;
        let [major, minor, _] = self.version_parts(version)?;
        let detail = format!("docs/changelog/{major}.{minor}.md");
        let finalizer = self.root.join(CHANGELOG_FINALIZER);
        Ok(BTreeMap::from([
            (
                "Cargo.lock".to_owned(),
                self.replace_lock_version(
                    &self.text("Cargo.lock", Some(source))?,
                    previous,
                    version,
                )?,
            ),
            (
                "Cargo.toml".to_owned(),
                self.replace_workspace_version(
                    &self.text("Cargo.toml", Some(source))?,
                    previous,
                    version,
                )?,
            ),
            (PACKAGE_README.to_owned(), readme),
            (
                "CHANGELOG.md".to_owned(),
                self.release_changelog(
                    &finalizer,
                    &self.text("CHANGELOG.md", Some(source))?,
                    previous,
                    version,
                    &selection.date,
                )?,
            ),
            (
                detail.clone(),
                self.release_changelog(
                    &finalizer,
                    &self.text(&detail, Some(source))?,
                    previous,
                    version,
                    &selection.date,
                )?,
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
            self.output("git", &args)?
                .lines()
                .eq(expected_changes.iter().map(String::as_str)),
            "release candidate contains changes outside its version surfaces",
        )?;
        require(
            self.output("git", &["ls-files", "--others", "--exclude-standard"])?
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
            .output(
                "git",
                &["diff", "--cached", "--name-only", &selection.source, "--"],
            )?
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
        self.version_parts(version)?;
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
            // Only coherent, admitted release sources may prepare tools. The
            // complete common selection is required by the offline full gate.
            self.run("make", &["--no-print-directory", "install-tools"])?;
        }
        self.run(
            "make",
            &["--no-print-directory", "tools-check", "format-tools-check"],
        )?;
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
        self.run("bash", &["scripts/ci/check-make-execution.sh"])?;
        let configuration = self.configuration()?;
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
            configuration,
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
        let prepared_path = self.receipt_path(&selection.version, true)?;
        let stale_prepared = if prepared_path.exists() {
            let prepared: PackageEvidence = serde_json::from_slice(&fs::read(&prepared_path)?)?;
            let previous = &prepared.validation.selection;
            if previous.source == selection.source {
                None
            } else {
                require(
                    previous.kind == selection.kind
                        && previous.previous == selection.previous
                        && previous.version == selection.version
                        && previous.remote == selection.remote
                        && previous.branch == selection.branch,
                    "superseded preparation identifies another release or destination",
                )?;
                require(
                    prepared.package_head == previous.source,
                    "superseded package evidence differs from its preparation source",
                )?;
                self.git(&[
                    "merge-base",
                    "--is-ancestor",
                    &previous.source,
                    &selection.source,
                ])?;
                Some(prepared_path)
            }
        } else {
            None
        };
        for (receipt, kind) in std::iter::once((&path, "validated"))
            .chain(stale_prepared.as_ref().map(|receipt| (receipt, "prepared")))
            .filter(|(receipt, _)| receipt.exists())
        {
            let retained = receipt
                .parent()
                .ok_or("validation path has no parent")?
                .join("attempts")
                .join(format!(
                    "{}-{kind}.{}.{}.json",
                    selection.version,
                    std::process::id(),
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
                ));
            self.exec.write_bytes(&retained, &fs::read(receipt)?)?;
        }
        // Fresh successful validation supersedes only an admitted older source.
        // Preserve both receipts first; unchanged-source recovery keeps its active
        // prepared receipt and existing integrity checks.
        if let Some(receipt) = stale_prepared {
            fs::remove_file(receipt)?;
        }
        self.exec.write_json(&path, &evidence)
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
        self.exec.write_json(
            &self.receipt_path(&evidence.validation.selection.version, prepared)?,
            evidence,
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
                self.exec
                    .write_bytes(&self.root.join(path), text.as_bytes())?;
            }
            self.exec.write_bytes(
                &self.root.join("Cargo.toml"),
                expected["Cargo.toml"].as_bytes(),
            )?;
            self.prepared(selection)?;
            Ok(())
        })();
        if let Err(source) = result {
            // Durable publication may report a parent-sync error after rename.
            // Inspect current bytes before restoring only owned replacements.
            // A failed restoration must not hide the preparation error or stop
            // restoration of other independent, owned metadata files.
            let mut rollback_errors = Vec::new();
            for (path, bytes) in backups {
                let restored = (|| -> Result<()> {
                    let current = fs::read(self.root.join(&path))?;
                    if current == bytes {
                        // No replacement needs undoing. Preserve the existing
                        // file identity and permissions without another write.
                        return Ok(());
                    }
                    let owned = if path == "Cargo.lock" {
                        std::str::from_utf8(&current).is_ok_and(|text| {
                            check_lock_update(&bytes, text, &selection.previous, &selection.version)
                                .is_ok()
                        })
                    } else {
                        current == expected[&path].as_bytes()
                    };
                    if owned {
                        self.exec.write_bytes(&self.root.join(&path), &bytes)?;
                    }
                    Ok(())
                })();
                if let Err(error) = restored {
                    rollback_errors.push((path, error));
                }
            }
            if rollback_errors.is_empty() {
                Err(source)
            } else {
                Err(PreparationError {
                    source,
                    rollback_errors,
                }
                .into())
            }
        } else {
            result
        }
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
    fn check_tag(&self, version: &str, head: &str) -> Result<String> {
        let tag = self.local_tag(version)?.ok_or("release tag is missing")?;
        self.output("bash", &[TAG_CHECKER, head, version])?;
        require(
            self.local_tag(version)?.as_deref() == Some(tag.as_str()),
            "release tag changed during admission",
        )?;
        Ok(tag)
    }

    fn publish(&self, dry_run: bool) -> Result<()> {
        let release = self.release_commit()?;
        let (version, head, _) = &release;
        let tag = self.check_tag(version, head)?;
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
            self.release_commit()? == release,
            "release changed before publication",
        )?;
        require(
            self.check_tag(version, head)? == tag,
            "release tag changed before publication",
        )?;
        // Cargo uploads the crate from the package directory. Evidence is checked
        // before dispatch and rechecked afterwards so a rebuild cannot replace it
        // silently. Publication itself is a network effect, never an offline retry.
        self.run(
            "cargo",
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        require(
            self.release_commit()? == release,
            "release changed during publication",
        )?;
        require(
            self.check_tag(version, head)? == tag,
            "release tag changed during publication",
        )
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
        "release-preflight" => repo.preflight(&ReleaseSelection::from_env(&repo)?)?,
        "release-verify" => repo.verify(&ReleaseSelection::from_env(&repo)?)?,
        "release-prepare-version" => repo.prepare_selected(&ReleaseSelection::from_env(&repo)?)?,
        "release-prepared-check" => {
            repo.prepared(&ReleaseSelection::from_env(&repo)?)?;
        }
        "release-files" => {
            for path in repo.surfaces(&ReleaseSelection::from_env(&repo)?)?.keys() {
                print!("{path}\0");
            }
        }
        "release-commit-check" => repo.commit_check(&ReleaseSelection::from_env(&repo)?)?,
        "release-committed-check" => {
            repo.selected_committed_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
        }
        "release-tagged-check" => {
            repo.tagged_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
        }
        "release-push-check" => {
            repo.push_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
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
