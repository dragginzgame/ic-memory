//! Maintainer release workflow and portable artifact qualification.
//!
//! Source identity, selected dependencies, compiler identities and the verified
//! package travel together in one receipt. Publication checks that evidence
//! before dispatch; it never resolves a replacement lockfile.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const SURFACES: [&str; 3] = ["Cargo.toml", "README.md", "CHANGELOG.md"];
const PROBES: [(&str, u64); 5] = [
    ("core", 260_000),
    ("diagnostics", 315_000),
    ("key_only", 260_000),
    ("admission", 264_000),
    ("runtime_integration", 270_000),
];

///
/// ReleaseEvidence
///
/// The exact inputs and artifact verified during release preparation.
///
/// Receipts remain local under Cargo's configured target directory. Missing or
/// changed evidence requires preparation again; no earlier receipt is accepted.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseEvidence {
    source: String,
    version: String,
    package_head: String,
    lock_sha256: String,
    package_sha256: String,
    cargo: String,
    rustc: String,
    msrv_rustc: String,
    configuration: BTreeMap<String, String>,
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

fn next_version(version: &str, kind: &str) -> Result<String> {
    let [major, minor, patch] = version_parts(version)?;
    match kind {
        "patch" => Ok(format!(
            "{major}.{minor}.{}",
            patch.checked_add(1).ok_or("patch overflow")?
        )),
        "minor" => Ok(format!(
            "{major}.{}.0",
            minor.checked_add(1).ok_or("minor overflow")?
        )),
        _ => Err("release kind must be patch or minor".into()),
    }
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

fn release_changelog(text: &str, version: &str) -> Result<String> {
    let headings: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect();
    let first = headings.first().ok_or("missing changelog entry")?;
    require(
        *first == version || *first == "[Draft]",
        "top changelog entry must be [Draft] or the selected version",
    )?;
    require(
        headings
            .iter()
            .filter(|heading| **heading == *first)
            .count()
            == 1,
        "duplicate current changelog entry",
    )?;
    require(
        *first == version || !headings.contains(&version),
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
    Ok(text.replacen(&heading, &format!("## {version}\n"), 1))
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

    fn sha256(&self, path: &Path) -> Result<String> {
        require(
            fs::metadata(path)?.is_file(),
            "checksum input must be a regular file",
        )?;
        // shasum ships with macOS and is also provided by Ubuntu's Perl package.
        let output = self.output(
            "shasum",
            &["-a", "256", path.to_str().ok_or("non-UTF-8 checksum path")?],
        )?;
        let digest = output
            .split_whitespace()
            .next()
            .ok_or("missing SHA-256 output")?;
        require(
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "invalid SHA-256 output",
        )?;
        Ok(digest.to_owned())
    }

    fn identities(&self) -> Result<[String; 3]> {
        Ok([
            self.output("cargo", &[&format!("+{}", self.toolchain()?), "--version"])?,
            self.output("rustc", &[&format!("+{}", self.toolchain()?), "-Vv"])?,
            self.output("rustc", &[&format!("+{}", self.msrv()?), "-Vv"])?,
        ])
    }

    fn retained_package(&self, digest: &str) -> Result<PathBuf> {
        require(
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "invalid package digest",
        )?;
        Ok(self
            .target()?
            .join("release-validation/artifacts")
            .join(format!("{digest}.crate")))
    }

    fn record_package(&self, version: &str) -> Result<String> {
        let package = self.package_path(version)?;
        let digest = self.sha256(&package)?;
        let retained = self.retained_package(&digest)?;
        if !retained.exists() {
            fs::create_dir_all(retained.parent().ok_or("archive has no parent")?)?;
            let temporary = retained.with_extension("crate.tmp");
            fs::copy(package, &temporary)?;
            require(
                self.sha256(&temporary)? == digest,
                "package changed while retaining evidence",
            )?;
            fs::rename(temporary, &retained)?;
        }
        require(
            self.sha256(&retained)? == digest,
            "retained package differs from its digest",
        )?;
        Ok(digest)
    }

    fn configuration(&self) -> Result<BTreeMap<String, String>> {
        let mut configuration = BTreeMap::new();
        for (name, value) in env::vars_os() {
            let name = name.to_str().ok_or("non-UTF-8 environment variable name")?;
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
                    configuration.insert(format!("config:{}", path.display()), self.sha256(&path)?);
                }
            }
        }
        Ok(configuration)
    }

    fn surfaces(&self, source: &str, version: &str) -> Result<BTreeMap<String, String>> {
        let previous = self.version(Some(source))?;
        let readme = self.text("README.md", Some(source))?;
        let old = format!("ic-memory = \"{previous}\"");
        require(
            readme.lines().filter(|line| *line == old).count() == 1,
            "README must contain exactly one current dependency example",
        )?;
        let readme = readme
            .split_inclusive('\n')
            .map(|line| {
                if line.trim_end_matches('\n') == old {
                    format!("ic-memory = \"{version}\"\n")
                } else {
                    line.to_owned()
                }
            })
            .collect();
        Ok(BTreeMap::from([
            (
                "Cargo.toml".to_owned(),
                replace_package_version(
                    &self.text("Cargo.toml", Some(source))?,
                    &previous,
                    version,
                )?,
            ),
            ("README.md".to_owned(), readme),
            (
                "CHANGELOG.md".to_owned(),
                release_changelog(&self.text("CHANGELOG.md", Some(source))?, version)?,
            ),
        ]))
    }

    fn check_surfaces(&self, source: &str, version: &str, revision: Option<&str>) -> Result<()> {
        let mut expected_changes = Vec::new();
        for (path, expected) in self.surfaces(source, version)? {
            require(
                self.text(&path, revision)? == expected,
                &format!("{path} differs from the validated release edit"),
            )?;
            if self.text(&path, Some(source))? != expected {
                expected_changes.push(path);
            }
        }
        let mut args = vec!["diff", "--name-only", source];
        if let Some(revision) = revision {
            args.push(revision);
        }
        args.push("--");
        let changed = self.git(&args)?;
        require(
            changed
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

    fn remote_ready(&self, version: &str, before_version: bool) -> Result<String> {
        let branch = self.git(&["symbolic-ref", "--quiet", "--short", "HEAD"])?;
        let remote_ref = format!("refs/remotes/origin/{branch}");
        self.git(&[
            "fetch",
            "--quiet",
            "--no-tags",
            "origin",
            &format!("+refs/heads/{branch}:{remote_ref}"),
        ])?;
        self.git(&["merge-base", "--is-ancestor", &remote_ref, "HEAD"])?;
        let name = format!("refs/tags/v{version}");
        let refs = self.git(&["ls-remote", "--refs", "origin", &name])?;
        if !refs.is_empty() {
            let fields: Vec<_> = refs.split_whitespace().collect();
            require(
                fields.len() == 2 && fields[1] == name,
                "unexpected remote tag response",
            )?;
            require(
                !before_version && self.local_tag(version)?.as_deref() == Some(fields[0]),
                "remote release tag already exists or conflicts",
            )?;
        }
        Ok(branch)
    }

    fn verify_inputs(&self, evidence: &ReleaseEvidence, source: &str, version: &str) -> Result<()> {
        require(
            evidence.source == source && evidence.version == version,
            "release evidence identifies another source or version",
        )?;
        require(
            self.sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "selected lockfile differs from release validation",
        )?;
        let [cargo, rustc, msrv_rustc] = self.identities()?;
        require(
            cargo == evidence.cargo && rustc == evidence.rustc && msrv_rustc == evidence.msrv_rustc,
            "compiler identities differ from release validation",
        )?;
        require(
            self.configuration()? == evidence.configuration,
            "build configuration differs from release validation",
        )?;
        Ok(())
    }

    fn verify_evidence(
        &self,
        source: &str,
        version: &str,
        head: &str,
        prepared: bool,
    ) -> Result<ReleaseEvidence> {
        let evidence: ReleaseEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(version, prepared)?)?)?;
        self.verify_inputs(&evidence, source, version)?;
        require(
            evidence.package_head == head,
            "package evidence identifies another Git HEAD",
        )?;
        require(
            self.sha256(&self.package_path(version)?)? == evidence.package_sha256,
            "package differs from release validation",
        )?;
        require(
            self.sha256(&self.retained_package(&evidence.package_sha256)?)?
                == evidence.package_sha256,
            "retained archive differs from release validation",
        )?;
        let mut commands = self.preparation_commands()?;
        if !prepared {
            commands.push(self.final_package_command()?);
        }
        require(
            evidence.commands == commands,
            "release evidence has unexpected qualification commands",
        )?;
        Ok(evidence)
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

    fn write_evidence(&self, evidence: &ReleaseEvidence, prepared: bool) -> Result<()> {
        let receipt = self.receipt_path(&evidence.version, prepared)?;
        fs::create_dir_all(receipt.parent().ok_or("receipt has no parent")?)?;
        let temporary = receipt.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_vec_pretty(evidence)?)?;
        fs::rename(temporary, receipt)?;
        Ok(())
    }

    fn preparation_commands(&self) -> Result<Vec<Vec<String>>> {
        Ok(vec![
            vec![
                "make".to_owned(),
                "--no-print-directory".to_owned(),
                "validate".to_owned(),
                format!("VALIDATION_TOOLCHAIN={}", self.toolchain()?),
            ],
            vec![
                "cargo".to_owned(),
                format!("+{}", self.toolchain()?),
                "update".to_owned(),
                "--workspace".to_owned(),
                "--offline".to_owned(),
            ],
            vec![
                "cargo".to_owned(),
                format!("+{}", self.toolchain()?),
                "package".to_owned(),
                "--locked".to_owned(),
                "--offline".to_owned(),
                "--allow-dirty".to_owned(),
            ],
        ])
    }

    #[expect(
        clippy::too_many_lines,
        reason = "keep version mutation and its rollback in one reviewable transaction"
    )]
    fn prepare(&self, kind: &str) -> Result<()> {
        self.clean()?;
        let source = self.git(&["rev-parse", "HEAD"])?;
        let previous = self.version(None)?;
        let version = next_version(&previous, kind)?;
        let expected = self.surfaces(&source, &version)?;
        require(
            self.local_tag(&version)?.is_none(),
            "local release tag already exists",
        )?;
        // A cache/lockfile must already be selected. No implicit online retry.
        let lock = self.root.join("Cargo.lock");
        let selected_lock = self.sha256(&lock)?;
        let [cargo, rustc, msrv_rustc] = self.identities()?;
        let configuration = self.configuration()?;
        self.remote_ready(&version, true)?;
        let commands = self.preparation_commands()?;
        let validate = &commands[0];
        self.run(
            &validate[0],
            &validate[1..].iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        self.clean()?;
        require(
            self.sha256(&lock)? == selected_lock,
            "validation changed dependency selection",
        )?;
        require(
            self.identities()? == [cargo.clone(), rustc.clone(), msrv_rustc.clone()],
            "compiler identities changed during validation",
        )?;
        require(
            self.configuration()? == configuration,
            "build configuration changed during validation",
        )?;
        require(
            self.git(&["rev-parse", "HEAD"])? == source,
            "source HEAD changed during validation",
        )?;
        self.remote_ready(&version, true)?;
        self.clean()?;
        require(
            self.git(&["rev-parse", "HEAD"])? == source,
            "source HEAD changed before version edits",
        )?;
        let backups: BTreeMap<_, _> = SURFACES
            .iter()
            .chain(std::iter::once(&"Cargo.lock"))
            .map(|path| Ok((*path, fs::read(self.root.join(path))?)))
            .collect::<Result<_>>()?;
        let result = (|| -> Result<()> {
            for (path, text) in &expected {
                fs::write(self.root.join(path), text)?;
            }
            let update = &commands[1];
            self.run(
                &update[0],
                &update[1..].iter().map(String::as_str).collect::<Vec<_>>(),
            )?;
            check_lock_update(
                &backups["Cargo.lock"],
                &fs::read_to_string(&lock)?,
                &previous,
                &version,
            )?;
            let package = &commands[2];
            self.run(
                &package[0],
                &package[1..].iter().map(String::as_str).collect::<Vec<_>>(),
            )?;
            require(
                self.git(&["rev-parse", "HEAD"])? == source,
                "source HEAD changed during preparation",
            )?;
            self.check_surfaces(&source, &version, None)?;
            require(
                self.identities()? == [cargo.clone(), rustc.clone(), msrv_rustc.clone()],
                "compiler identities changed during preparation",
            )?;
            require(
                self.configuration()? == configuration,
                "build configuration changed during preparation",
            )?;
            let evidence = ReleaseEvidence {
                package_head: source.clone(),
                source,
                version: version.clone(),
                lock_sha256: self.sha256(&lock)?,
                package_sha256: self.record_package(&version)?,
                cargo,
                rustc,
                msrv_rustc,
                configuration,
                commands,
            };
            // Publish complete evidence only after all checks pass. The previous
            // receipt and build artifacts survive failures before this rename.
            self.write_evidence(&evidence, true)
        })();
        if result.is_err() {
            for (path, contents) in backups {
                fs::write(self.root.join(path), contents)?;
            }
        }
        result?;
        println!(
            "Prepared {version}. Review git diff; the maintainer runs release-stage, release-commit and release-push."
        );
        Ok(())
    }

    fn prepared(&self) -> Result<ReleaseEvidence> {
        let source = self.git(&["rev-parse", "HEAD"])?;
        let version = self.version(None)?;
        let previous = self.version(Some(&source))?;
        require(
            version == next_version(&previous, "patch")?
                || version == next_version(&previous, "minor")?,
            "candidate is not the next patch or minor",
        )?;
        self.check_surfaces(&source, &version, None)?;
        self.verify_evidence(&source, &version, &source, true)
    }

    fn stage(&self) -> Result<()> {
        self.prepared()?;
        self.run(
            "git",
            &["add", "--", "Cargo.toml", "README.md", "CHANGELOG.md"],
        )
    }

    fn inspect_release_commit(&self) -> Result<(String, String, String)> {
        self.clean()?;
        let version = self.version(None)?;
        let head = self.git(&["rev-parse", "HEAD"])?;
        let source = self.git(&["rev-parse", "HEAD^"])?;
        require(
            self.git(&["log", "-1", "--format=%B"])?
                == format!("Release {version}\n\nValidated-source: {source}"),
            "HEAD is not a validated release commit",
        )?;
        self.check_surfaces(&source, &version, Some("HEAD"))?;
        let previous = self.version(Some(&source))?;
        require(
            version == next_version(&previous, "patch")?
                || version == next_version(&previous, "minor")?,
            "release is not the next patch or minor",
        )?;
        Ok((version, head, source))
    }

    fn release_commit(&self) -> Result<(String, String, String)> {
        let (version, head, source) = self.inspect_release_commit()?;
        self.verify_evidence(&source, &version, &head, false)?;
        Ok((version, head, source))
    }

    fn qualify_release(&self) -> Result<()> {
        let (version, head, source) = self.inspect_release_commit()?;
        let mut evidence: ReleaseEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&version, true)?)?)?;
        self.verify_inputs(&evidence, &source, &version)?;
        require(
            evidence.commands == self.preparation_commands()?,
            "prepared evidence has unexpected commands",
        )?;
        let command = self.final_package_command()?;
        self.run(
            &command[0],
            &command[1..].iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        require(
            self.inspect_release_commit()? == (version.clone(), head.clone(), source.clone()),
            "release changed during final package qualification",
        )?;
        self.verify_inputs(&evidence, &source, &version)?;
        evidence.package_head = head;
        evidence.package_sha256 = self.record_package(&version)?;
        evidence.commands.push(command);
        self.write_evidence(&evidence, false)
    }

    fn check_tag(&self, version: &str, head: &str) -> Result<()> {
        require(self.local_tag(version)?.is_some(), "release tag is missing")?;
        require(
            self.git(&["cat-file", "-t", &format!("refs/tags/v{version}")])? == "tag",
            "release tag must be annotated",
        )?;
        require(
            self.git(&["rev-parse", &format!("refs/tags/v{version}^{{commit}}")])? == head,
            "release tag does not identify HEAD",
        )
    }

    fn commit(&self) -> Result<()> {
        if self.git(&["log", "-1", "--format=%s"])? != format!("Release {}", self.version(None)?) {
            let evidence = self.prepared()?;
            require(
                self.git(&["diff", "--name-only"])?.is_empty(),
                "release surfaces have unstaged changes",
            )?;
            let changed = self.git(&["diff", "--cached", "--name-only"])?;
            let expected = self.git(&["diff", "--name-only", &evidence.source])?;
            require(changed == expected, "stage exactly the release edits")?;
            require(
                self.local_tag(&evidence.version)?.is_none(),
                "release tag already exists",
            )?;
            self.run(
                "git",
                &[
                    "commit",
                    "-m",
                    &format!(
                        "Release {}\n\nValidated-source: {}",
                        evidence.version, evidence.source
                    ),
                ],
            )?;
        }
        self.qualify_release()?;
        let (version, head, _) = self.release_commit()?;
        if self.local_tag(&version)?.is_none() {
            self.run(
                "git",
                &[
                    "tag",
                    "-a",
                    &format!("v{version}"),
                    "-m",
                    &format!("Release {version}"),
                ],
            )?;
        }
        self.check_tag(&version, &head)
    }

    fn push(&self) -> Result<()> {
        let (version, head, source) = self.release_commit()?;
        self.check_tag(&version, &head)?;
        let branch = self.remote_ready(&version, false)?;
        require(
            self.release_commit()? == (version.clone(), head.clone(), source),
            "release changed during remote inspection",
        )?;
        self.check_tag(&version, &head)?;
        self.run(
            "git",
            &[
                "push",
                "--no-follow-tags",
                "--atomic",
                "origin",
                &format!("{head}:refs/heads/{branch}"),
                &format!("refs/tags/v{version}:refs/tags/v{version}"),
            ],
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
        "usage: repo-tool <version|msrv|toolchain|ensure-clean|wasm-size|patch|minor|stage|commit|qualify-release|push|publish> [--dry-run]",
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
        "ensure-clean" => repo.clean()?,
        "wasm-size" => check_wasm_artifacts(&repo.target()?)?,
        "patch" | "minor" => repo.prepare(command)?,
        "stage" => repo.stage()?,
        "commit" => repo.commit()?,
        "qualify-release" => repo.qualify_release()?,
        "push" => repo.push()?,
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
