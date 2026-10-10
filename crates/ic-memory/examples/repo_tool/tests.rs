//! Release effects are substituted; native Git checks use disposable indexes.
//! No fixture commits, tags, pushes or live publication occur.

use super::*;
use std::cell::RefCell;

const MANIFEST: &str = "[workspace]\nmembers = [\"crates/ic-memory\"]\n[workspace.package]\nversion = \"0.12.3\"\nrust-version = \"1.88.0\"\n[workspace.dependencies]\nother = \"0.12.3\"\n";
const PACKAGE: &str = "[package]\nname = \"ic-memory\"\nversion.workspace = true\n";
const LOCK: &str = "version = 4\n[[package]]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n[[package]]\nname = \"other\"\nversion = \"0.12.3\"\n";

#[test]
fn release_version_requires_the_named_package_to_inherit_the_workspace_identity() {
    let fixture = Fixture::new();
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for package in [
        "[package]\nname = \"another-package\"\nversion.workspace = true\n",
        "[package]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n",
    ] {
        fs::write(fixture.repo.root.join(PACKAGE_MANIFEST), package).unwrap();
        assert!(fixture.repo.version(None).is_err());
        assert_eq!(fixture.repo.version(Some("source")).unwrap(), "0.12.3");
    }
}

struct Fixture {
    repo: Repository<Substitute>,
    selection: ReleaseSelection,
}

impl Fixture {
    fn new() -> Self {
        Self::new_in(&env::temp_dir())
    }

    fn new_in(parent: &Path) -> Self {
        // Exclusive creation owns uniqueness, including across PID reuse.
        // Never reuse or remove an occupied path: it may hold failed evidence.
        let mut sequence = 0_u64;
        let root = loop {
            let candidate = parent.join(format!(
                "ic-memory-tooling-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => sequence += 1,
                Err(error) => panic!("could not create fixture {}: {error}", candidate.display()),
            }
        };
        let source = BTreeMap::from([
            ("Cargo.lock".to_owned(), LOCK.to_owned()),
            ("Cargo.toml".to_owned(), MANIFEST.to_owned()),
            (PACKAGE_MANIFEST.to_owned(), PACKAGE.to_owned()),
            (
                PACKAGE_README.to_owned(),
                "# Fixture\n\nic-memory = \"0.12.3\"\n".to_owned(),
            ),
            (
                "CHANGELOG.md".to_owned(),
                "# Changelog\n\n## [0.13.0]\n\nFixture changes.\n\n## 0.12.3\n\nHistory.\n"
                    .to_owned(),
            ),
            ("source.rs".to_owned(), "// committed source\n".to_owned()),
            (
                "docs/changelog/0.13.md".to_owned(),
                "# Detail\n\n## [0.13.0]\n\nFixture changes.\n".to_owned(),
            ),
        ]);
        for (path, text) in &source {
            fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            fs::write(root.join(path), text).unwrap();
        }
        fs::write(
            root.join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.99.0\"\n",
        )
        .unwrap();
        let exec = Substitute {
            state: RefCell::new(State {
                source,
                release: None,
                staged: false,
                calls: Vec::new(),
                fail: None,
                archive_change: None,
                tag: false,
                descendant: None,
            }),
        };
        Self {
            repo: Repository { root, exec },
            selection: ReleaseSelection {
                kind: "minor".to_owned(),
                previous: "0.12.3".to_owned(),
                version: "0.13.0".to_owned(),
                date: "2026-10-05".to_owned(),
                source: "source".to_owned(),
                remote: "origin".to_owned(),
                branch: "main".to_owned(),
            },
        }
    }

    fn prepare(&self) -> Result<()> {
        self.repo.preflight(&self.selection)?;
        self.repo.verify(&self.selection)?;
        self.repo.preflight(&self.selection)?;
        self.repo.prepare_selected(&self.selection)
    }

    fn stage(&self) -> Result<()> {
        self.repo.prepared(&self.selection)?;
        self.repo.exec.state.borrow_mut().staged = true;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        if self.repo.exec.state.borrow().release.is_none() {
            self.repo.commit_check(&self.selection)?;
            let mut state = self.repo.exec.state.borrow_mut();
            let release = state
                .source
                .keys()
                .map(|path| {
                    (
                        path.clone(),
                        fs::read_to_string(self.repo.root.join(path)).unwrap(),
                    )
                })
                .collect();
            state.release = Some(release);
            state
                .calls
                .push(vec!["shared-runner".to_owned(), "commit".to_owned()]);
        }
        self.repo
            .selected_committed_check(&self.selection, "release")?;
        self.repo.exec.state.borrow_mut().tag = true;
        Ok(())
    }

    fn release(&self) {
        self.prepare().unwrap();
        self.stage().unwrap();
        self.commit().unwrap();
    }

    fn fail(&self, effect: &str) {
        self.repo.exec.state.borrow_mut().fail = Some(effect.to_owned());
    }

    fn assert_original(&self) {
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.toml")).unwrap(),
            MANIFEST
        );
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.lock")).unwrap(),
            LOCK
        );
        assert!(self.repo.exec.state.borrow().release.is_none());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let saved = (|| -> Result<()> {
                let state = self.repo.exec.state.try_borrow()?;
                fs::write(
                    self.repo.root.join("fixture-state.json"),
                    serde_json::to_vec_pretty(&*state)?,
                )?;
                Ok(())
            })();
            if let Err(error) = saved {
                eprintln!("Could not save fixture command state: {error}");
            }
            eprintln!("Release fixture retained: {}", self.repo.root.display());
            return;
        }
        fs::remove_dir_all(&self.repo.root).unwrap();
    }
}

#[test]
fn failed_release_fixture_retains_receipts_and_command_state() {
    let mut retained = PathBuf::new();
    let mut receipt = PathBuf::new();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let fixture = Fixture::new();
        retained.clone_from(&fixture.repo.root);
        fixture.repo.verify(&fixture.selection).unwrap();
        receipt = fixture
            .repo
            .validation_path(&fixture.selection.version)
            .unwrap();
        panic!("simulate a failed assertion after validation");
    }));
    assert!(failure.is_err());
    let state: State =
        serde_json::from_slice(&fs::read(retained.join("fixture-state.json")).unwrap()).unwrap();
    assert!(
        state
            .calls
            .iter()
            .any(|call| call.get(2).is_some_and(|arg| arg == "validate"))
    );
    assert_eq!(
        fs::read_to_string(retained.join("Cargo.lock")).unwrap(),
        LOCK
    );
    let evidence: ValidationEvidence = serde_json::from_slice(&fs::read(receipt).unwrap()).unwrap();
    assert_eq!(evidence.selection.source, "source");
    fs::remove_dir_all(retained).unwrap();

    let successful = Fixture::new();
    let root = successful.repo.root.clone();
    drop(successful);
    assert!(!root.exists());
}

#[test]
fn fixture_creation_preserves_existing_directories_and_files() {
    let parent = Fixture::new();
    let retained = parent
        .repo
        .root
        .join(format!("ic-memory-tooling-{}-0", std::process::id()));
    fs::create_dir(&retained).unwrap();
    fs::write(retained.join("evidence"), b"retained failure").unwrap();
    let foreign = parent
        .repo
        .root
        .join(format!("ic-memory-tooling-{}-1", std::process::id()));
    fs::write(&foreign, b"unrelated file").unwrap();

    let next = Fixture::new_in(&parent.repo.root);
    assert_eq!(next.repo.version(None).unwrap(), "0.12.3");
    assert_ne!(next.repo.root, retained);
    assert_ne!(next.repo.root, foreign);
    drop(next);
    assert_eq!(
        fs::read(retained.join("evidence")).unwrap(),
        b"retained failure"
    );
    assert_eq!(fs::read(foreign).unwrap(), b"unrelated file");
}

#[test]
fn durable_publication_replaces_complete_files_and_preserves_rejected_targets() {
    let fixture = Fixture::new();
    let target = fixture.repo.root.join("durable/release.json");
    Processes.write_bytes(&target, b"original receipt").unwrap();
    let mut original = fs::File::open(&target).unwrap();
    Processes
        .write_with(&target, |file| {
            std::io::Write::write_all(file, b"complete replacement")
        })
        .unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    let mut original_bytes = Vec::new();
    std::io::Read::read_to_end(&mut original, &mut original_bytes).unwrap();
    assert_eq!(original_bytes, b"original receipt");

    let rejected = fixture.repo.root.join("durable/retained-directory");
    fs::create_dir(&rejected).unwrap();
    fs::write(rejected.join("evidence"), "keep").unwrap();
    let entries = || {
        fs::read_dir(target.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let before = entries();
    let error = Processes
        .write_with(&target, |file| {
            std::io::Write::write_all(file, b"partial replacement")?;
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        })
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<std::io::Error>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } if source.kind() == std::io::ErrorKind::PermissionDenied
    ));
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    assert_eq!(entries(), before);

    let error = Processes
        .write_bytes(&rejected, b"invalid replacement")
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<std::io::Error>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(rejected.join("evidence")).unwrap(),
        "keep"
    );
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    assert_eq!(entries(), before);
}

#[test]
fn streamed_receipts_preserve_json_and_reject_partial_serialization() {
    struct RejectedValue;
    impl Serialize for RejectedValue {
        fn serialize<S: serde::Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("reject after the receipt prefix"))
        }
    }

    let fixture = Fixture::new();
    fixture.release();
    let path = fixture.repo.validation_path("0.13.0").unwrap();
    let original = fs::read(&path).unwrap();
    let validation: ValidationEvidence = serde_json::from_slice(&original).unwrap();
    Processes.write_json(&path, &validation).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    for prepared in [true, false] {
        let path = fixture.repo.receipt_path("0.13.0", prepared).unwrap();
        let original = fs::read(&path).unwrap();
        let evidence: PackageEvidence = serde_json::from_slice(&original).unwrap();
        Processes.write_json(&path, &evidence).unwrap();
        assert_eq!(fs::read(path).unwrap(), original);
    }

    let entries = || fs::read_dir(path.parent().unwrap()).unwrap().count();
    let before = entries();
    let error = Processes
        .write_json(&path, &(&validation, RejectedValue))
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<serde_json::Error>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } if source.is_data()
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(entries(), before);
}

#[derive(Deserialize, Serialize)]
struct State {
    source: BTreeMap<String, String>,
    release: Option<BTreeMap<String, String>>,
    staged: bool,
    calls: Vec<Vec<String>>,
    fail: Option<String>,
    archive_change: Option<PathBuf>,
    tag: bool,
    descendant: Option<BTreeMap<String, String>>,
}

struct Substitute {
    state: RefCell<State>,
}

impl Execute for Substitute {
    fn write_json(&self, path: &Path, value: &impl Serialize) -> Result<()> {
        // Reuse the fixture's before/after-publication fault injection. The
        // native streaming boundary is exercised separately below.
        self.write_bytes(path, &serde_json::to_vec_pretty(value)?)
    }

    fn write_with(
        &self,
        path: &Path,
        write: impl FnOnce(&mut fs::File) -> std::io::Result<()>,
    ) -> Result<()> {
        let phase = {
            let mut state = self.state.borrow_mut();
            state
                .calls
                .push(vec!["stream-write".to_owned(), path.display().to_string()]);
            let phase = state.fail.clone();
            if let Some(source) = state.archive_change.take() {
                if phase.as_deref() == Some("archive-read") {
                    fs::remove_file(&source)?;
                    fs::create_dir(source)?;
                } else {
                    fs::write(source, b"changed after digest admission")?;
                }
            }
            phase
        };
        if phase.as_deref() == Some("archive-before") {
            return Err("fixture failure before archive publication".into());
        }
        Processes.write_with(path, write)?;
        if phase.as_deref() == Some("archive-after") {
            return Err("fixture failure after archive publication".into());
        }
        Ok(())
    }

    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        let failure = self.state.borrow().fail.clone();
        let phase = failure
            .as_deref()
            .and_then(|failure| failure.split_once(':'))
            .filter(|(_, target)| path.ends_with(target))
            .map(|(phase, _)| phase);
        if phase.is_some() {
            self.state.borrow_mut().fail = None;
        }
        if phase == Some("write-before") {
            return Err(std::io::Error::other("fixture failure before publication").into());
        }
        Processes.write_bytes(path, bytes)?;
        if phase == Some("write-after") {
            // Model the real writer's error-after-rename contract. The consumer
            // must inspect the visible bytes before rollback or retry.
            assert_eq!(fs::read(path)?, bytes);
            self.state.borrow_mut().calls.push(vec![
                "published-write-error".to_owned(),
                path.display().to_string(),
            ]);
            return Err(std::io::Error::other("fixture failure after publication").into());
        }
        Ok(())
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one explicit process substitute records every permitted effect"
    )]
    fn run(&self, root: &Path, program: &str, args: &[&str], _: bool) -> Result<String> {
        let mut state = self.state.borrow_mut();
        state.calls.push(
            std::iter::once(program)
                .chain(args.iter().copied())
                .map(str::to_owned)
                .collect(),
        );
        let target = root.join("configured output");
        let version = || {
            toml_string(
                &fs::read_to_string(root.join("Cargo.toml"))?,
                "workspace.package",
                "version",
            )
        };
        let changed = |reference: &BTreeMap<String, String>| -> Result<String> {
            Ok(reference
                .iter()
                .filter_map(|(path, text)| match fs::read_to_string(root.join(path)) {
                    Ok(current) if current == *text => None,
                    _ => Some(path.as_str()),
                })
                .collect::<Vec<_>>()
                .join("\n"))
        };
        match program {
            "bash"
                if args == ["scripts/ci/check-make-execution.sh"]
                    && state.fail.as_deref() == Some("make-execution") =>
            {
                Err("fixture Make execution refusal".into())
            }
            "bash" => Processes.run(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
                program,
                args,
                true,
            ),
            "rustc" => Ok(format!("fixture-rustc {}\n", args[0])),
            "cargo"
                if args
                    == [
                        "metadata",
                        "--locked",
                        "--offline",
                        "--no-deps",
                        "--format-version",
                        "1",
                    ] =>
            {
                if state.fail.as_deref() == Some("metadata") {
                    return Err("fixture metadata failure".into());
                }
                Ok(serde_json::json!({"target_directory": target}).to_string())
            }
            "cargo" if args.get(1) == Some(&"--version") => {
                Ok(format!("fixture-cargo {}\n", args[0]))
            }
            "cargo" if args.get(1) == Some(&"update") => {
                let mut lock = fs::read_to_string(root.join("Cargo.lock"))?.replacen(
                    "name = \"ic-memory\"\nversion = \"0.12.3\"",
                    &format!("name = \"ic-memory\"\nversion = \"{}\"", version()?),
                    1,
                );
                if state.fail.as_deref() == Some("dependency-update") {
                    lock = lock.replace(
                        "name = \"other\"\nversion = \"0.12.3\"",
                        "name = \"other\"\nversion = \"0.12.4\"",
                    );
                }
                fs::write(root.join("Cargo.lock"), lock)?;
                Ok(String::new())
            }
            "cargo" if args.get(1) == Some(&"fetch") => Ok(String::new()),
            "cargo" if args.get(1) == Some(&"package") => {
                let directory = target.join("package");
                fs::create_dir_all(&directory)?;
                if state.fail.as_deref() == Some("package-head") {
                    let mut descendant = state.release.clone().ok_or("missing release")?;
                    descendant.insert(
                        "source.rs".to_owned(),
                        "// fix during packaging\n".to_owned(),
                    );
                    for (path, text) in &descendant {
                        fs::write(root.join(path), text)?;
                    }
                    state.descendant = Some(descendant);
                }
                // Model Cargo's Git metadata changing after the release commit.
                let bytes = if state.descendant.is_some() {
                    "descendant-archive"
                } else if state.release.is_some() {
                    "release-archive"
                } else {
                    "prepared-archive"
                };
                fs::write(
                    directory.join(format!("ic-memory-{}.crate", version()?)),
                    bytes,
                )?;
                if state.fail.as_deref() == Some("package") {
                    return Err("fixture package failure".into());
                }
                Ok(String::new())
            }
            "cargo" if args.get(1) == Some(&"publish") => {
                match state.fail.as_deref() {
                    Some("publish-tag-missing") => state.tag = false,
                    Some("publish-tag-commit") => state.fail = Some("tag-commit".to_owned()),
                    Some("publish-tag-object") => state.fail = Some("tag-object".to_owned()),
                    _ => {}
                }
                Ok(String::new())
            }
            "make" if args == ["--no-print-directory", "install-tools"] => {
                if state.fail.as_deref() == Some("tool-setup") {
                    return Err("fixture tool setup failure".into());
                }
                Ok(String::new())
            }
            "make" if args == ["--no-print-directory", "tools-check", "check-format-tools"] => {
                if state.fail.as_deref() == Some("tool-check") {
                    return Err("fixture offline tool admission failure".into());
                }
                Ok(String::new())
            }
            "make"
                if args
                    == [
                        "--no-print-directory",
                        "validate",
                        "VALIDATION_TOOLCHAIN=1.99.0",
                    ] =>
            {
                if state.fail.as_deref() == Some("validate") {
                    return Err("fixture validation failure".into());
                }
                if state.fail.as_deref() == Some("validation-lock") {
                    fs::write(root.join("Cargo.lock"), "unexpected dependency selection")?;
                }
                Ok(String::new())
            }
            "make"
                if args
                    == [
                        "--no-print-directory",
                        "fmt-check",
                        "VALIDATION_TOOLCHAIN=1.99.0",
                    ] =>
            {
                if state.fail.as_deref() == Some("formatting") {
                    return Err("fixture formatting failure".into());
                }
                Ok(String::new())
            }
            "git" => match args {
                ["--no-optional-locks", "status", ..]
                    if state.fail.as_deref() == Some("status") =>
                {
                    Err("fixture Git status observation failure".into())
                }
                ["--no-optional-locks", "status", ..] => changed(
                    state
                        .descendant
                        .as_ref()
                        .or(state.release.as_ref())
                        .unwrap_or(&state.source),
                ),
                ["rev-parse", "HEAD"] => {
                    if state.fail.as_deref() == Some("publish-before-tag")
                        && state
                            .calls
                            .iter()
                            .any(|call| call.get(1).is_some_and(|arg| arg == "cat-file"))
                    {
                        state.tag = false;
                    }
                    Ok(if state.descendant.is_some() {
                        "fix"
                    } else if state.release.is_some() {
                        "release"
                    } else {
                        "source"
                    }
                    .to_owned())
                }
                ["rev-parse", "HEAD^"] => Ok("source".to_owned()),
                ["show", object] => {
                    let (revision, path) =
                        object.split_once(':').ok_or("invalid fixture object")?;
                    match revision {
                        "" if state.staged => Ok(fs::read_to_string(root.join(path))?),
                        "source" => state
                            .source
                            .get(path)
                            .cloned()
                            .ok_or_else(|| "missing fixture source".into()),
                        "release" => state
                            .release
                            .as_ref()
                            .and_then(|files| files.get(path))
                            .cloned()
                            .ok_or_else(|| "missing fixture release".into()),
                        "fix" => state
                            .descendant
                            .as_ref()
                            .and_then(|files| files.get(path))
                            .cloned()
                            .ok_or_else(|| "missing fixture descendant".into()),
                        _ => Err("unexpected fixture revision".into()),
                    }
                }
                ["for-each-ref", ..] => Ok(if state.tag
                    && state.fail.as_deref() == Some("tag-object")
                {
                    "changed-tag-object refs/tags/v0.13.0"
                } else if state.tag {
                    "tag-object refs/tags/v0.13.0"
                } else {
                    ""
                }
                .to_owned()),
                ["symbolic-ref", ..] => Ok("main".to_owned()),
                ["merge-base", ..] if state.fail.as_deref() == Some("remote-ahead") => {
                    Err("fixture remote is ahead".into())
                }
                ["fetch", ..] if state.fail.as_deref() == Some("remote-lock") => {
                    if state
                        .calls
                        .iter()
                        .filter(|call| call.get(1).is_some_and(|arg| arg == "fetch"))
                        .count()
                        == 2
                    {
                        fs::write(
                            root.join("Cargo.lock"),
                            LOCK.replace(
                                "name = \"other\"\nversion = \"0.12.3\"",
                                "name = \"other\"\nversion = \"0.12.4\"",
                            ),
                        )?;
                    }
                    Ok(String::new())
                }
                ["merge-base", "--is-ancestor", "release", "HEAD"]
                    if state.fail.as_deref() == Some("history") =>
                {
                    Err("fixture unrelated history".into())
                }
                ["fetch" | "merge-base" | "ls-files", ..]
                | ["diff", "--cached", "--summary", "source", "--", _] => Ok(String::new()),
                ["diff", "--cached", "--name-only", "source", "--"] => {
                    if state.staged && state.release.is_none() {
                        changed(&state.source)
                    } else {
                        Ok(String::new())
                    }
                }
                ["diff", "--name-only", "source", "release", "--"] => {
                    let release = state.release.as_ref().ok_or("missing release")?;
                    Ok(state
                        .source
                        .iter()
                        .filter_map(|(path, original)| {
                            (release.get(path) != Some(original)).then_some(path.as_str())
                        })
                        .collect::<Vec<_>>()
                        .join("\n"))
                }
                ["diff", "--name-only", "source", ..] => changed(&state.source),
                ["diff", "--name-only"] => {
                    if state.staged {
                        Ok(String::new())
                    } else {
                        changed(&state.source)
                    }
                }
                ["diff", "--cached", "--name-only"] => {
                    if state.staged {
                        changed(&state.source)
                    } else {
                        Ok(String::new())
                    }
                }
                ["log", "-1", "--format=%P", "release"] => {
                    Ok(if state.fail.as_deref() == Some("parent") {
                        "wrong-source"
                    } else {
                        "source"
                    }
                    .to_owned())
                }
                ["log", "-1", "--format=%s", "fix"] => Ok("Fix callback".to_owned()),
                ["log", "-1", "--format=%s", "release"] => Ok(if state.release.is_some() {
                    "Release 0.13.0"
                } else {
                    "Fixture source"
                }
                .to_owned()),
                ["cat-file", "-t", ..] => Ok("tag".to_owned()),
                ["rev-parse", reference] if reference.ends_with("^{commit}") => {
                    Ok(if state.fail.as_deref() == Some("tag-commit") {
                        "fix"
                    } else {
                        "release"
                    }
                    .to_owned())
                }
                _ => Err(format!("unexpected fixture git command: {args:?}").into()),
            },
            _ => Err(format!("unexpected fixture process: {program} {args:?}").into()),
        }
    }
}

#[test]
fn retained_package_paths_require_digest_authority_before_target_discovery() {
    let fixture = Fixture::new();
    for digest in ["", "../other", &"A".repeat(64), &"0".repeat(63)] {
        assert!(fixture.repo.retained_package(digest).is_err());
    }
    assert_eq!(fixture.repo.exec.state.borrow().calls.len(), 0);
    let digest = "0".repeat(64);
    assert_eq!(
        fixture.repo.retained_package(&digest).unwrap(),
        fixture
            .repo
            .root
            .join("configured output/release-validation/artifacts")
            .join(format!("{digest}.crate"))
    );
}

#[test]
fn compiler_environment_overrides_refuse_preparation_and_release_effects() {
    const ROOT: &str = "IC_MEMORY_TEST_COMPILER_ROOT";
    const STATE: &str = "IC_MEMORY_TEST_COMPILER_STATE";
    const OVERRIDES: [&str; 8] = [
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTDOC",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CARGO_BUILD_RUSTDOC",
    ];
    if let Some(state) = env::var_os(STATE) {
        // Separate processes exercise real inherited environment without
        // unsafe mutation of the concurrent Rust test harness environment.
        let fixture = Fixture::new();
        let receipt = fixture.repo.receipt_path("0.13.0", true).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&receipt, "previous evidence").unwrap();
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
        assert_eq!(fs::read(receipt).unwrap(), b"previous evidence");
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            call[0] == "make"
                || call
                    .get(2)
                    .is_some_and(|arg| matches!(arg.as_str(), "update" | "package"))
        }));

        let repo = Repository {
            root: PathBuf::from(env::var_os(ROOT).unwrap()),
            exec: Substitute {
                state: RefCell::new(
                    serde_json::from_str::<State>(state.to_str().unwrap()).unwrap(),
                ),
            },
        };
        let call_start = repo.exec.state.borrow().calls.len();
        assert!(repo.qualify_release().is_err());
        assert!(repo.push_check(&fixture.selection, "release").is_err());
        assert!(repo.publish(true).is_err());
        assert!(
            !repo.exec.state.borrow().calls[call_start..]
                .iter()
                .any(|call| {
                    call.get(1).is_some_and(|arg| arg == "push")
                        || call
                            .get(2)
                            .is_some_and(|arg| matches!(arg.as_str(), "package" | "publish"))
                })
        );
        return;
    }

    let fixture = Fixture::new();
    fixture.release();
    let state = serde_json::to_string(&*fixture.repo.exec.state.borrow()).unwrap();
    let evidence = fixture
        .repo
        .verify_evidence(&fixture.selection, "release", false, true)
        .unwrap();
    let paths = [
        fixture.repo.receipt_path("0.13.0", true).unwrap(),
        fixture.repo.receipt_path("0.13.0", false).unwrap(),
        fixture.repo.package_path("0.13.0").unwrap(),
        fixture
            .repo
            .retained_package(&evidence.package_sha256)
            .unwrap(),
    ];
    let before: Vec<_> = paths.iter().map(|path| fs::read(path).unwrap()).collect();
    for name in OVERRIDES {
        for value in ["", "alternate-tool"] {
            let mut child = Command::new(env::current_exe().unwrap());
            child.args(["--exact", "repo_tool::tests::compiler_environment_overrides_refuse_preparation_and_release_effects"]);
            for variable in OVERRIDES {
                child.env_remove(variable);
            }
            let output = child
                .env(ROOT, &fixture.repo.root)
                .env(STATE, &state)
                .env(name, value)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{name}={value:?}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            for (path, bytes) in paths.iter().zip(&before) {
                assert_eq!(fs::read(path).unwrap(), *bytes);
            }
        }
    }
}

#[test]
fn compiler_configuration_overrides_refuse_preparation() {
    for filename in ["config", "config.toml"] {
        for field in [
            "rustc",
            "rustc-wrapper",
            "rustc-workspace-wrapper",
            "rustdoc",
        ] {
            for value in ["", "alternate-tool"] {
                let fixture = Fixture::new();
                let directory = fixture.repo.root.join(".cargo");
                fs::create_dir(&directory).unwrap();
                let path = directory.join(filename);
                let text = format!("[build]\n{field} = {value:?}\n");
                fs::write(&path, &text).unwrap();
                assert!(fixture.prepare().is_err());
                fixture.assert_original();
                assert_eq!(fs::read_to_string(path).unwrap(), text);
                assert!(
                    !fixture
                        .repo
                        .exec
                        .state
                        .borrow()
                        .calls
                        .iter()
                        .any(|call| call[0] == "make")
                );
            }
        }
    }
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "[build]\njobs = 1\n[profile.dev]\nopt-level = 1\n",
    )
    .unwrap();
    fixture.release();
    fixture.repo.publish(true).unwrap();

    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "token = 'private-test-value' trailing\n",
    )
    .unwrap();
    let error = fixture.repo.configuration().unwrap_err().to_string();
    assert!(!error.contains("private-test-value"));
}

#[test]
fn inherited_compiler_configuration_refuses_preparation() {
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "[build]\nrustc-wrapper = 'alternate-tool'\n",
    )
    .unwrap();
    let root = fixture.repo.root.join("nested");
    fs::create_dir(&root).unwrap();
    let source = fixture.repo.exec.state.borrow().source.clone();
    for path in source
        .keys()
        .map(String::as_str)
        .chain(["rust-toolchain.toml"])
    {
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        fs::copy(fixture.repo.root.join(path), root.join(path)).unwrap();
    }
    let repo = Repository {
        root,
        exec: Substitute {
            state: RefCell::new(State {
                source,
                release: None,
                staged: false,
                calls: Vec::new(),
                fail: None,
                archive_change: None,
                tag: false,
                descendant: None,
            }),
        },
    };
    assert!(repo.preflight(&fixture.selection).is_err());
    assert_eq!(repo.version(None).unwrap(), "0.12.3");
    assert!(
        !repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .any(|call| call[0] == "make")
    );
}

#[test]
fn release_tool_setup_and_offline_admission_precede_gates_and_version_mutation() {
    for phase in ["tool-setup", "tool-check"] {
        let fixture = Fixture::new();
        fixture.fail(phase);
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            call.iter()
                .any(|argument| argument == "validate" || argument == "update")
        }));
    }
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    let state = fixture.repo.exec.state.borrow();
    let position = |argument: &str| {
        state
            .calls
            .iter()
            .position(|call| call.iter().any(|value| value == argument))
            .unwrap()
    };
    assert!(position("fetch") < position("install-tools"));
    assert!(position("install-tools") < position("tools-check"));
    assert!(position("tools-check") < position("validate"));
    assert!(position("validate") < position("update"));
}

#[test]
fn release_normalizes_dependency_examples_without_requiring_the_previous_version() {
    for (original, expected) in [
        ("ic-memory = \"0.34\"\n", "ic-memory = \"0.13.0\"\n"),
        (
            "# café\r\n  ic-memory='0.1' # retain \"comment\"\r\n",
            "# café\r\n  ic-memory=\"0.13.0\" # retain \"comment\"\r\n",
        ),
        (
            "ic-memory = { features = [\"example\"], version = '0.13', default-features = false }\n",
            "ic-memory = { features = [\"example\"], version = \"0.13.0\", default-features = false }\n",
        ),
        ("ic-memory = \"0.13.0\"", "ic-memory = \"0.13.0\""),
    ] {
        let fixture = Fixture::new();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .source
            .insert(PACKAGE_README.to_owned(), original.to_owned());
        fs::write(fixture.repo.root.join(PACKAGE_README), original).unwrap();
        fixture.release();
        fixture.repo.publish(true).unwrap();
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(PACKAGE_README)).unwrap(),
            expected
        );
    }
}

#[test]
fn missing_ambiguous_or_unsupported_dependency_examples_do_not_block_release() {
    for readme in [
        "# Guide without a dependency example\n",
        "ic-memory = \"0.12\"\nic-memory = \"0.13\"\n",
        "ic-memory = \"unterminated\n",
        "ic-memory = { path = \"../ic-memory\" }\n",
    ] {
        let fixture = Fixture::new();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .source
            .insert(PACKAGE_README.to_owned(), readme.to_owned());
        fs::write(fixture.repo.root.join(PACKAGE_README), readme).unwrap();
        fixture.release();
        fixture.repo.publish(true).unwrap();
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(PACKAGE_README)).unwrap(),
            readme
        );
    }
}

#[test]
fn release_versions_and_pending_notes_preserve_dependency_versions_and_history() {
    for invalid in ["0.01.3", "0.12", "0.12.3-dev", "0.12.+3"] {
        assert!(version_parts(invalid).is_err());
    }
    let updated = replace_workspace_version(MANIFEST, "0.12.3", "0.13.0").unwrap();
    assert!(updated.contains("other = \"0.12.3\""));
    let current = "# Changelog\n\n## [0.13.0]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n";
    assert_eq!(
        release_changelog(current, "0.13.0", "2026-10-05").unwrap(),
        current.replacen("## [0.13.0]", "## [0.13.0] - 2026-10-05", 1)
    );
    assert!(release_changelog(current, "0.12.4", "2026-10-05").is_err());
    assert_eq!(
        release_changelog("## [0.13.0]\n\n## 0.12.3\nHistory", "0.13.0", "2026-10-05").unwrap(),
        "## [0.13.0] - 2026-10-05\n\n## 0.12.3\nHistory"
    );
    assert!(release_changelog("## [0.13.0]", "0.13.0", "2026-10-05").is_err());
    for historical in ["0.13.0", "[0.13.0]", "[0.13.0] - 2026-10-05"] {
        let duplicate = format!("{current}\n## {historical}\n\nAlready released.\n");
        assert!(release_changelog(&duplicate, "0.13.0", "2026-10-05").is_err());
    }
    for date in [
        "2026-02-29",
        "2026-04-31",
        "2026-13-01",
        "2026-10-05\n",
        "0000-01-01",
    ] {
        assert!(!valid_date(date));
    }
    assert!(valid_date("2024-02-29"));
}

#[test]
fn empty_numbered_notes_prepare_without_weakening_release_identity() {
    let fixture = Fixture::new();
    for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
        let text =
            "# Notes\n\n## [0.13.0]\n\n## [0.12.3] - 2026-10-04\n\nHistory without a final newline";
        fs::write(fixture.repo.root.join(path), text).unwrap();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .source
            .insert(path.to_owned(), text.to_owned());
    }
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    fixture.commit().unwrap();
    fixture.repo.publish(true).unwrap();
    for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
            "# Notes\n\n## [0.13.0] - 2026-10-05\n\n## [0.12.3] - 2026-10-04\n\nHistory without a final newline"
        );
    }
}

#[test]
fn preparation_stage_commit_qualification_and_publish_preserve_exact_evidence() {
    let fixture = Fixture::new();
    fixture.release();
    let prepared: PackageEvidence = serde_json::from_slice(
        &fs::read(fixture.repo.receipt_path("0.13.0", true).unwrap()).unwrap(),
    )
    .unwrap();
    let final_evidence = fixture
        .repo
        .verify_evidence(&fixture.selection, "release", false, true)
        .unwrap();
    assert_ne!(prepared.package_sha256, final_evidence.package_sha256);
    assert!(
        fixture
            .repo
            .retained_package(&prepared.package_sha256)
            .unwrap()
            .is_file()
    );
    assert!(
        fixture
            .repo
            .retained_package(&final_evidence.package_sha256)
            .unwrap()
            .is_file()
    );
    assert_eq!(final_evidence.package_head, "release");
    fixture
        .repo
        .tagged_check(&fixture.selection, "release")
        .unwrap();
    assert!(
        fixture
            .repo
            .root
            .join("configured output/release-validation/0.13.0-prepared.json")
            .is_file()
    );
    fixture
        .repo
        .push_check(&fixture.selection, "release")
        .unwrap();
    fixture.repo.publish(true).unwrap();
    let state = fixture.repo.exec.state.borrow();
    assert!(
        !state.calls.iter().any(|call| call[0] == "git"
            && matches!(call[1].as_str(), "add" | "commit" | "tag" | "push"))
    );
    assert!(state.calls.iter().any(|call| call
        == &[
            "cargo",
            "+1.99.0",
            "publish",
            "--locked",
            "--registry",
            "crates-io",
            "--dry-run"
        ]));
    assert!(!state.calls.iter().any(|call| {
        call.iter()
            .any(|arg| arg == "generate-lockfile" || arg == "clean")
    }));
}

#[test]
fn version_preparation_failures_restore_files_and_preserve_evidence_and_artifacts() {
    for failure in ["validate", "formatting", "package", "dependency-update"] {
        let fixture = Fixture::new();
        let receipt = fixture.repo.receipt_path("0.13.0", false).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&receipt, "previous evidence").unwrap();
        let sentinel = fixture.repo.target().unwrap().join("consumer-evidence");
        fs::write(&sentinel, "keep").unwrap();
        fixture.fail(failure);
        assert!(fixture.prepare().is_err());
        assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
        if failure == "dependency-update" {
            assert!(
                fs::read_to_string(fixture.repo.root.join("Cargo.lock"))
                    .unwrap()
                    .contains("name = \"other\"\nversion = \"0.12.4\"")
            );
        } else {
            fixture.assert_original();
        }
        for (path, text) in &fixture.repo.exec.state.borrow().source {
            if failure == "dependency-update" && path == "Cargo.lock" {
                continue;
            }
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                *text
            );
        }
        assert_eq!(fs::read_to_string(receipt).unwrap(), "previous evidence");
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
        if failure == "formatting" {
            assert!(
                !fixture
                    .repo
                    .exec
                    .state
                    .borrow()
                    .calls
                    .iter()
                    .any(|call| { call.get(2).is_some_and(|arg| arg == "package") })
            );
        }
    }
}

#[test]
fn preparation_retains_primary_and_rollback_errors_and_restores_other_owned_files() {
    struct RollbackFailures<'a>(&'a Substitute);

    impl Execute for RollbackFailures<'_> {
        fn run(&self, root: &Path, program: &str, args: &[&str], capture: bool) -> Result<String> {
            let result = self.0.run(root, program, args, capture)?;
            if program == "cargo" && args.get(1) == Some(&"package") {
                // A missing surface models failed inspection during restoration.
                fs::rename(root.join("CHANGELOG.md"), root.join("retained-changelog"))?;
                // A foreign replacement must never be overwritten by rollback.
                fs::write(root.join(PACKAGE_README), b"concurrent edit")?;
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
            }
            Ok(result)
        }

        fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
            if path.ends_with("Cargo.lock") && bytes == LOCK.as_bytes() {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
            }
            self.0.write_bytes(path, bytes)
        }
    }

    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let saved = fs::read(&validation).unwrap();
    let repo = Repository {
        root: fixture.repo.root.clone(),
        exec: RollbackFailures(&fixture.repo.exec),
    };
    let error = repo.prepare_selected(&fixture.selection).unwrap_err();
    let failure = error.downcast_ref::<PreparationError>().unwrap();
    assert_eq!(
        failure
            .source
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::UnexpectedEof
    );
    let failures: BTreeMap<_, _> = failure
        .rollback_errors
        .iter()
        .map(|(path, error)| {
            (
                path.as_str(),
                error.downcast_ref::<std::io::Error>().unwrap().kind(),
            )
        })
        .collect();
    assert_eq!(
        failures,
        BTreeMap::from([
            ("CHANGELOG.md", std::io::ErrorKind::NotFound),
            ("Cargo.lock", std::io::ErrorKind::PermissionDenied),
        ])
    );
    assert_eq!(repo.version(None).unwrap(), "0.12.3");
    assert_eq!(
        fs::read(repo.root.join(PACKAGE_README)).unwrap(),
        b"concurrent edit"
    );
    assert_eq!(
        fs::read_to_string(repo.root.join("docs/changelog/0.13.md")).unwrap(),
        fixture.repo.exec.state.borrow().source["docs/changelog/0.13.md"]
    );
    assert_eq!(fs::read(validation).unwrap(), saved);
    assert!(repo.package_path("0.13.0").unwrap().exists());
    assert!(!repo.receipt_path("0.13.0", true).unwrap().exists());
    assert!(failure.source().unwrap().is::<std::io::Error>());
    let diagnostic = failure.to_string();
    for path in failures.keys() {
        assert!(diagnostic.contains(path));
    }
}

#[test]
fn failed_first_publication_preserves_untouched_metadata_identity_and_permissions() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let saved = fs::read(&validation).unwrap();
    let metadata: BTreeMap<_, _> = fixture
        .repo
        .surfaces(&fixture.selection)
        .unwrap()
        .into_keys()
        .map(|path| {
            let file = fixture.repo.root.join(&path);
            fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
            let metadata = fs::metadata(file).unwrap();
            (path, (metadata.ino(), metadata.mode()))
        })
        .collect();
    fixture.fail("write-before:CHANGELOG.md");
    let error = fixture
        .repo
        .prepare_selected(&fixture.selection)
        .unwrap_err();
    assert!(error.is::<std::io::Error>());
    fixture.assert_original();
    for (path, identity) in metadata {
        let actual = fs::metadata(fixture.repo.root.join(path)).unwrap();
        assert_eq!((actual.ino(), actual.mode()), identity);
    }
    assert_eq!(fs::read(validation).unwrap(), saved);
    assert!(!fixture.repo.receipt_path("0.13.0", true).unwrap().exists());
}

#[test]
fn metadata_publication_failures_reconcile_visible_bytes_and_retry_saved_intent() {
    for phase in ["write-before", "write-after"] {
        for path in [
            "Cargo.lock",
            "CHANGELOG.md",
            PACKAGE_README,
            "docs/changelog/0.13.md",
            "Cargo.toml",
        ] {
            let fixture = Fixture::new();
            fixture.repo.verify(&fixture.selection).unwrap();
            let receipt = fixture.repo.validation_path("0.13.0").unwrap();
            let saved = fs::read(&receipt).unwrap();
            let sentinel = fixture.repo.target().unwrap().join("consumer-evidence");
            fs::write(&sentinel, "keep").unwrap();
            fixture.fail(&format!("{phase}:{path}"));
            assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
            for (path, text) in &fixture.repo.exec.state.borrow().source {
                assert_eq!(
                    fs::read(fixture.repo.root.join(path)).unwrap(),
                    text.as_bytes()
                );
            }
            assert_eq!(fs::read(&receipt).unwrap(), saved);
            assert_eq!(fs::read_to_string(&sentinel).unwrap(), "keep");
            let published_errors = fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| call[0] == "published-write-error")
                .count();
            assert_eq!(published_errors, usize::from(phase == "write-after"));
            fixture.repo.prepare_selected(&fixture.selection).unwrap();
            fixture.repo.prepared(&fixture.selection).unwrap();
            assert_eq!(fixture.repo.version(None).unwrap(), "0.13.0");
            assert_eq!(fs::read(receipt).unwrap(), saved);
            assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
            assert_eq!(
                fixture
                    .repo
                    .exec
                    .state
                    .borrow()
                    .calls
                    .iter()
                    .filter(|call| { call.get(2).is_some_and(|argument| argument == "validate") })
                    .count(),
                1
            );
        }
    }
}

#[test]
fn validation_refuses_make_modes_before_gates_and_preserves_evidence() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let receipt = fixture.repo.validation_path("0.13.0").unwrap();
    let previous = fs::read(&receipt).unwrap();
    fixture.fail("make-execution");
    fixture.repo.exec.state.borrow_mut().calls.clear();
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&receipt).unwrap(), previous);
    assert_eq!(
        fixture.repo.exec.state.borrow().calls,
        vec![vec!["bash", "scripts/ci/check-make-execution.sh"]]
    );
    assert!(!receipt.parent().unwrap().join("attempts").exists());
    fixture.assert_original();
}

#[test]
fn validation_binds_saved_intent_and_retains_previous_attempts() {
    let fixture = Fixture::new();
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    fixture.assert_original();
    fixture.repo.verify(&fixture.selection).unwrap();
    let receipt = fixture.repo.validation_path("0.13.0").unwrap();
    let first = fs::read(&receipt).unwrap();
    for field in [
        "kind", "version", "previous", "source", "date", "remote", "branch",
    ] {
        let mut selection = serde_json::to_value(&fixture.selection).unwrap();
        selection[field] = serde_json::json!("changed");
        let selection = serde_json::from_value(selection).unwrap();
        assert!(
            fixture.repo.prepare_selected(&selection).is_err(),
            "{field}"
        );
        fixture.assert_original();
        assert_eq!(fs::read(&receipt).unwrap(), first);
    }
    fixture.fail("validate");
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&receipt).unwrap(), first);
    fixture.repo.exec.state.borrow_mut().fail = None;
    fixture.repo.verify(&fixture.selection).unwrap();
    let attempts = receipt.parent().unwrap().join("attempts");
    let entries: Vec<_> = fs::read_dir(attempts).unwrap().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        fs::read(entries[0].as_ref().unwrap().path()).unwrap(),
        first
    );
    fixture.assert_original();
}

#[test]
fn validation_lock_cannot_be_rebound_to_an_uncommitted_graph() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let path = fixture.repo.validation_path("0.13.0").unwrap();
    let mut evidence: ValidationEvidence =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let changed = LOCK.replace(
        "name = \"other\"\nversion = \"0.12.3\"",
        "name = \"other\"\nversion = \"0.12.4\"",
    );
    evidence.lock = changed.as_bytes().to_vec();
    fs::write(&path, serde_json::to_vec(&evidence).unwrap()).unwrap();
    fs::write(fixture.repo.root.join("Cargo.lock"), &changed).unwrap();
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
        changed
    );
    assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
        call.get(2)
            .is_some_and(|arg| matches!(arg.as_str(), "update" | "package"))
    }));
}

#[test]
fn interrupted_preparation_finishes_saved_candidate_without_revalidating() {
    for interruption in ["partial", "partial-lock", "manifest", "lock", "package"] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let evidence = fs::read(fixture.repo.validation_path("0.13.0").unwrap()).unwrap();
        let expected = fixture.repo.surfaces(&fixture.selection).unwrap();
        for (path, text) in &expected {
            if match interruption {
                "partial" => path == PACKAGE_README,
                "partial-lock" => path.as_str() <= "Cargo.lock",
                _ => true,
            } {
                fs::write(fixture.repo.root.join(path), text).unwrap();
            }
        }
        if matches!(interruption, "lock" | "package") {
            fixture
                .repo
                .run_command(&fixture.repo.preparation_commands().unwrap()[0])
                .unwrap();
        }
        if interruption == "package" {
            fixture.fail("package");
            assert!(fixture.repo.prepared(&fixture.selection).is_err());
            assert!(fixture.repo.package_path("0.13.0").unwrap().exists());
            fixture.repo.exec.state.borrow_mut().fail = None;
        }
        if matches!(interruption, "partial" | "partial-lock") {
            fixture.repo.preflight(&fixture.selection).unwrap();
            if interruption == "partial-lock" {
                assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
                    call[0] == "cargo" && call.get(2).is_some_and(|arg| arg == "fetch")
                }));
            }
            fixture.repo.prepare_selected(&fixture.selection).unwrap();
        } else {
            fixture.repo.prepared(&fixture.selection).unwrap();
        }
        for (path, text) in expected {
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                text
            );
        }
        assert_eq!(
            fs::read(fixture.repo.validation_path("0.13.0").unwrap()).unwrap(),
            evidence
        );
        let calls = fixture.repo.exec.state.borrow().calls.clone();
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.get(2).is_some_and(|arg| arg == "validate"))
                .count(),
            1
        );
        let start = calls.len();
        fixture.repo.prepared(&fixture.selection).unwrap();
        assert!(
            !fixture.repo.exec.state.borrow().calls[start..]
                .iter()
                .any(|call| call.get(2).is_some_and(|arg| matches!(
                    arg.as_str(),
                    "update" | "package" | "fmt-check" | "validate"
                )))
        );
        assert!(fixture.repo.exec.state.borrow().release.is_none());
    }
}

#[test]
fn partial_lock_preflight_requires_saved_validation() {
    for invalid in ["missing", "selection", "configuration"] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let path = fixture.repo.validation_path("0.13.0").unwrap();
        if invalid == "missing" {
            fs::remove_file(&path).unwrap();
        } else {
            let mut evidence: ValidationEvidence =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            if invalid == "selection" {
                evidence.selection.date = "2026-10-04".to_owned();
            } else {
                evidence
                    .configuration
                    .insert("RUSTFLAGS".to_owned(), "--cfg foreign".to_owned());
            }
            fs::write(&path, serde_json::to_vec(&evidence).unwrap()).unwrap();
        }
        let expected = fixture.repo.surfaces(&fixture.selection).unwrap();
        fs::write(
            fixture.repo.root.join("Cargo.lock"),
            &expected["Cargo.lock"],
        )
        .unwrap();
        assert!(fixture.repo.preflight(&fixture.selection).is_err());
        assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
            expected["Cargo.lock"]
        );
        assert!(
            !fixture.repo.exec.state.borrow().calls.iter().any(|call| {
                call[0] == "cargo" && call.get(2).is_some_and(|arg| arg == "fetch")
            })
        );
    }
}

#[test]
fn preparation_recovery_preserves_conflicting_inputs_and_corrupted_evidence() {
    for conflict in ["metadata", "lock", "receipt", "archive"] {
        let fixture = Fixture::new();
        fixture.prepare().unwrap();
        let receipt = fixture.repo.receipt_path("0.13.0", true).unwrap();
        let evidence: PackageEvidence =
            serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
        let path = match conflict {
            "metadata" => fixture.repo.root.join(PACKAGE_README),
            "lock" => fixture.repo.root.join("Cargo.lock"),
            "receipt" => receipt,
            "archive" => fixture
                .repo
                .retained_package(&evidence.package_sha256)
                .unwrap(),
            _ => unreachable!(),
        };
        fs::write(&path, b"conflicting retained input").unwrap();
        let start = fixture.repo.exec.state.borrow().calls.len();
        assert!(
            fixture.repo.prepared(&fixture.selection).is_err(),
            "{conflict}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"conflicting retained input");
        assert!(
            !fixture.repo.exec.state.borrow().calls[start..]
                .iter()
                .any(|call| call
                    .get(2)
                    .is_some_and(|arg| matches!(arg.as_str(), "update" | "package")))
        );
    }
}

#[test]
fn all_release_kinds_finalize_root_and_detail_notes_at_saved_date() {
    for (kind, version, detail) in [
        ("patch", "0.12.4", "docs/changelog/0.12.md"),
        ("minor", "0.13.0", "docs/changelog/0.13.md"),
        ("major", "1.0.0", "docs/changelog/1.0.md"),
    ] {
        let mut fixture = Fixture::new();
        fixture.selection.kind = kind.to_owned();
        fixture.selection.version = version.to_owned();
        for path in ["CHANGELOG.md", detail] {
            let text = format!("# Notes\n\n## [{version}]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n");
            fs::write(fixture.repo.root.join(path), &text).unwrap();
            fixture
                .repo
                .exec
                .state
                .borrow_mut()
                .source
                .insert(path.to_owned(), text);
        }
        fixture.prepare().unwrap();
        assert_eq!(fixture.repo.version(None).unwrap(), version);
        let files = fixture.repo.surfaces(&fixture.selection).unwrap();
        assert_eq!(
            files.keys().map(String::as_str).collect::<Vec<_>>(),
            [
                "CHANGELOG.md",
                "Cargo.lock",
                "Cargo.toml",
                PACKAGE_README,
                detail
            ],
        );
        for path in ["CHANGELOG.md", detail] {
            let text = fs::read_to_string(fixture.repo.root.join(path)).unwrap();
            assert!(text.contains(&format!("## [{version}] - 2026-10-05")));
            assert!(text.ends_with("## 0.12.3\n\nHistory.\n"));
        }
        assert!(
            fs::read_to_string(fixture.repo.root.join("Cargo.lock"))
                .unwrap()
                .contains("name = \"other\"\nversion = \"0.12.3\"")
        );
    }
}

#[test]
fn archive_publication_errors_preserve_validation_and_reconcile_visible_bytes() {
    for phase in ["archive-before", "archive-after"] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let validation = fixture.repo.validation_path("0.13.0").unwrap();
        let validation_bytes = fs::read(&validation).unwrap();
        let digest = Sha256Digest::compute(b"prepared-archive").to_string();
        let retained = fixture.repo.retained_package(&digest).unwrap();
        fs::create_dir_all(retained.parent().unwrap()).unwrap();
        let foreign = retained.with_extension("crate.tmp");
        fs::write(&foreign, b"unrelated evidence").unwrap();
        fixture.fail(phase);
        assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
        fixture.assert_original();
        assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
        assert!(!fixture.repo.receipt_path("0.13.0", true).unwrap().exists());
        assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
        if phase == "archive-after" {
            assert_eq!(fs::read(&retained).unwrap(), b"prepared-archive");
        } else {
            assert!(!retained.exists());
        }
        let transfers = || {
            fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| {
                    call.first()
                        .is_some_and(|command| command == "stream-write")
                })
                .count()
        };
        let before_retry = transfers();
        fixture.repo.exec.state.borrow_mut().fail = None;
        fixture.repo.prepare_selected(&fixture.selection).unwrap();
        let evidence = fixture.repo.prepared(&fixture.selection).unwrap();
        assert_eq!(evidence.package_sha256, digest);
        assert_eq!(fs::read(&retained).unwrap(), b"prepared-archive");
        assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
        assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
        assert_eq!(
            transfers() - before_retry,
            usize::from(phase == "archive-before")
        );
        fs::write(&retained, b"corrupted retained archive").unwrap();
        assert!(fixture.repo.record_package("0.13.0").is_err());
        assert_eq!(fs::read(&retained).unwrap(), b"corrupted retained archive");
    }
}

#[test]
fn archive_copy_io_failure_preserves_native_error_and_unpublished_state() {
    let fixture = Fixture::new();
    let package = fixture.repo.package_path("0.13.0").unwrap();
    fs::create_dir_all(package.parent().unwrap()).unwrap();
    fs::write(&package, b"archive").unwrap();
    let digest = Sha256Digest::compute(b"archive").to_string();
    let retained = fixture.repo.retained_package(&digest).unwrap();
    fs::create_dir_all(retained.parent().unwrap()).unwrap();
    let foreign = retained.with_extension("crate.tmp");
    fs::write(&foreign, b"unrelated evidence").unwrap();
    fixture.repo.exec.state.borrow_mut().archive_change = Some(package.clone());
    fixture.fail("archive-read");

    let error = fixture.repo.record_package("0.13.0").unwrap_err();
    let ic_host_fs::durable::NamedWriteError::Producer {
        source: actual,
        cleanup_error: None,
    } = error
        .downcast_ref::<ic_host_fs::durable::NamedWriteError<std::io::Error>>()
        .unwrap()
    else {
        panic!("archive read failure must retain its producer error before publication");
    };
    let expected = fs::read(&package).unwrap_err();
    assert_eq!(actual.kind(), expected.kind());
    assert_eq!(actual.raw_os_error(), expected.raw_os_error());
    assert!(!retained.exists());
    assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
    assert_eq!(fs::read_dir(retained.parent().unwrap()).unwrap().count(), 1);
    assert!(!fixture.repo.receipt_path("0.13.0", true).unwrap().exists());
}

#[test]
fn changed_archive_source_stops_before_publication_or_receipt() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let validation_bytes = fs::read(&validation).unwrap();
    let package = fixture.repo.package_path("0.13.0").unwrap();
    fixture.repo.exec.state.borrow_mut().archive_change = Some(package.clone());
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    fixture.assert_original();
    let digest = Sha256Digest::compute(b"prepared-archive").to_string();
    assert!(!fixture.repo.retained_package(&digest).unwrap().exists());
    assert!(!fixture.repo.receipt_path("0.13.0", true).unwrap().exists());
    assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
    assert_eq!(
        fs::read(&package).unwrap(),
        b"changed after digest admission"
    );
    fixture.repo.prepare_selected(&fixture.selection).unwrap();
    assert_eq!(
        fixture
            .repo
            .prepared(&fixture.selection)
            .unwrap()
            .package_sha256,
        digest
    );
}

#[test]
fn source_admission_reports_actual_status_and_preserves_git_failures() {
    let fixture = Fixture::new();
    let checkout = fixture.repo.root.join("status-checkout");
    Processes
        .run(
            &fixture.repo.root,
            "git",
            &[
                "clone",
                "--quiet",
                "--shared",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .to_str()
                    .unwrap(),
                checkout.to_str().unwrap(),
            ],
            true,
        )
        .unwrap();
    let repo = Repository {
        root: checkout,
        exec: Processes,
    };
    // Discard stat-cache entries without changing the tree, so a plain status
    // observation would refresh the index even though the checkout is clean.
    repo.git(&["read-tree", "HEAD"]).unwrap();
    repo.git(&["checkout-index", "--all", "--force"]).unwrap();
    let clean_index = fs::read(repo.root.join(".git/index")).unwrap();
    repo.clean().unwrap();
    assert_eq!(fs::read(repo.root.join(".git/index")).unwrap(), clean_index);
    let lock = format!(
        "{}\n# staged fixture\n",
        repo.text("Cargo.lock", None).unwrap()
    );
    fs::write(repo.root.join("Cargo.lock"), &lock).unwrap();
    repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
    let manifest = format!(
        "{}\n# working fixture\n",
        repo.text("Cargo.toml", None).unwrap()
    );
    fs::write(repo.root.join("Cargo.toml"), &manifest).unwrap();
    let untracked = "untracked\nfile";
    fs::write(repo.root.join(untracked), b"untracked fixture").unwrap();
    let tree = repo.git(&["write-tree"]).unwrap();
    let status = repo
        .output(
            "git",
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )
        .unwrap();
    assert!(status.contains("M  Cargo.lock"));
    assert!(status.contains(" M Cargo.toml"));
    assert!(status.contains("?? \"untracked\\nfile\""));
    let dirty_index = fs::read(repo.root.join(".git/index")).unwrap();
    let error = repo.clean().unwrap_err().to_string();
    assert!(error.ends_with(status.trim_end()));
    assert_eq!(fs::read(repo.root.join(".git/index")).unwrap(), dirty_index);
    assert_eq!(repo.git(&["write-tree"]).unwrap(), tree);
    assert_eq!(repo.text("Cargo.lock", None).unwrap(), lock);
    assert_eq!(repo.text("Cargo.toml", None).unwrap(), manifest);
    assert_eq!(
        fs::read(repo.root.join(untracked)).unwrap(),
        b"untracked fixture"
    );

    fixture.fail("status");
    assert_eq!(
        fixture.repo.clean().unwrap_err().to_string(),
        "fixture Git status observation failure"
    );
    assert_eq!(
        fixture.repo.exec.state.borrow().calls,
        [vec![
            "git",
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--untracked-files=all"
        ]]
    );
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
}

#[test]
fn release_admission_rejects_whitespace_only_paths() {
    let fixture = Fixture::new();
    let repo = Repository {
        root: fixture.repo.root.clone(),
        exec: Processes,
    };
    fs::create_dir_all(repo.root.join("scripts/ci")).unwrap();
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/ci/next-release-version.sh"),
        repo.root.join("scripts/ci/next-release-version.sh"),
    )
    .unwrap();
    fs::write(repo.root.join(" "), b"original tracked bytes\n").unwrap();
    repo.git(&["init", "--quiet"]).unwrap();
    repo.git(&["add", "--", "."]).unwrap();
    let selection = ReleaseSelection {
        source: repo.git(&["write-tree"]).unwrap(),
        ..fixture.selection.clone()
    };
    let surfaces = repo.surfaces(&selection).unwrap();
    repo.check_surfaces(&selection, None, true).unwrap();
    repo.check_index(&selection, &surfaces).unwrap();

    fs::write(repo.root.join("  "), b"retained untracked bytes\n").unwrap();
    assert_eq!(
        repo.check_surfaces(&selection, None, true)
            .unwrap_err()
            .to_string(),
        "release candidate has untracked files"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), selection.source);
    assert_eq!(
        fs::read(repo.root.join("  ")).unwrap(),
        b"retained untracked bytes\n"
    );
    fs::remove_file(repo.root.join("  ")).unwrap();

    fs::write(repo.root.join(" "), b"changed tracked bytes\n").unwrap();
    assert_eq!(
        repo.check_surfaces(&selection, None, true)
            .unwrap_err()
            .to_string(),
        "release candidate contains changes outside its version surfaces"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), selection.source);
    assert_eq!(
        fs::read(repo.root.join(" ")).unwrap(),
        b"changed tracked bytes\n"
    );

    repo.git(&["add", "--", " "]).unwrap();
    let staged = repo.git(&["write-tree"]).unwrap();
    fs::write(repo.root.join(" "), b"original tracked bytes\n").unwrap();
    repo.check_surfaces(&selection, None, true).unwrap();
    assert_eq!(
        repo.check_index(&selection, &surfaces)
            .unwrap_err()
            .to_string(),
        "index contains unrelated release changes"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), staged);
    assert_eq!(
        fs::read(repo.root.join(" ")).unwrap(),
        b"original tracked bytes\n"
    );
}

#[test]
fn dirty_sources_and_changed_validation_inputs_prevent_version_edits() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.root.join(PACKAGE_README), "unfinished").unwrap();
    assert!(fixture.prepare().is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
    let fixture = Fixture::new();
    fixture.fail("validation-lock");
    assert!(fixture.prepare().is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
}

#[test]
fn lockfile_changes_during_post_validation_remote_inspection_stop_before_mutation() {
    let fixture = Fixture::new();
    fixture.fail("remote-lock");
    assert!(fixture.prepare().is_err());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for (path, text) in &fixture.repo.exec.state.borrow().source {
        if path == "Cargo.lock" {
            continue;
        }
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
            *text
        );
    }
    // Preserve the separately changed lock selection; it is not our version edit.
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
        LOCK.replace(
            "name = \"other\"\nversion = \"0.12.3\"",
            "name = \"other\"\nversion = \"0.12.4\""
        )
    );
    assert!(!fixture.repo.receipt_path("0.13.0", true).unwrap().exists());
    assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
        call.get(2)
            .is_some_and(|arg| matches!(arg.as_str(), "update" | "package"))
    }));
}

#[test]
fn changed_missing_and_wrong_source_receipts_never_dispatch_publication() {
    for changed in [
        "lock",
        "package",
        "receipt",
        "compiler",
        "source",
        "commands",
        "tag",
        "configuration",
        "retained",
    ] {
        let fixture = Fixture::new();
        fixture.release();
        let receipt = fixture.repo.receipt_path("0.13.0", false).unwrap();
        match changed {
            "lock" => {
                fs::remove_file(fixture.repo.root.join("Cargo.lock")).unwrap();
            }
            "package" => {
                fs::write(
                    fixture.repo.package_path("0.13.0").unwrap(),
                    "changed artifact",
                )
                .unwrap();
            }
            "receipt" => {
                fs::remove_file(receipt).unwrap();
            }
            "retained" => {
                let evidence: PackageEvidence =
                    serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
                fs::remove_file(
                    fixture
                        .repo
                        .retained_package(&evidence.package_sha256)
                        .unwrap(),
                )
                .unwrap();
            }
            "tag" => {
                fixture.repo.exec.state.borrow_mut().tag = false;
            }
            field => {
                let mut evidence: serde_json::Value =
                    serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
                if field == "configuration" {
                    evidence["validation"][field] =
                        serde_json::json!({"env:RUSTFLAGS": "--cfg unqualified"});
                } else if field == "commands" {
                    evidence[field] = serde_json::json!([]);
                } else if field == "compiler" {
                    evidence["validation"]["identities"][1] = serde_json::json!("changed");
                } else {
                    evidence["validation"]["selection"][field] = serde_json::json!("changed");
                }
                fs::write(receipt, serde_json::to_vec(&evidence).unwrap()).unwrap();
            }
        }
        if changed == "tag" {
            assert!(
                fixture
                    .repo
                    .tagged_check(&fixture.selection, "release")
                    .is_err()
            );
        }
        assert!(fixture.repo.publish(true).is_err());
        assert!(
            !fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .any(|call| call.get(2).is_some_and(|arg| arg == "publish"))
        );
    }
}

#[test]
fn publication_rechecks_tag_identity_without_replaying_effects_or_replacing_evidence() {
    for change in [
        "publish-before-tag",
        "publish-tag-missing",
        "publish-tag-commit",
        "publish-tag-object",
    ] {
        let fixture = Fixture::new();
        fixture.release();
        let receipt = fixture.repo.receipt_path("0.13.0", false).unwrap();
        let before = fs::read(&receipt).unwrap();
        fixture.fail(change);
        assert!(fixture.repo.publish(true).is_err(), "accepted {change}");
        assert_eq!(fs::read(&receipt).unwrap(), before);
        assert_eq!(
            fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| { call.get(2).is_some_and(|arg| arg == "publish") })
                .count(),
            usize::from(change != "publish-before-tag")
        );
    }
}

#[test]
fn invalid_prepared_artifacts_stop_final_qualification_without_replacing_evidence() {
    for changed in ["head", "missing", "corrupted"] {
        let fixture = Fixture::new();
        fixture.release();
        let prepared_path = fixture.repo.receipt_path("0.13.0", true).unwrap();
        let mut prepared: PackageEvidence =
            serde_json::from_slice(&fs::read(&prepared_path).unwrap()).unwrap();
        let retained = fixture
            .repo
            .retained_package(&prepared.package_sha256)
            .unwrap();
        match changed {
            "head" => {
                prepared.package_head = "unvalidated-head".to_owned();
                fs::write(&prepared_path, serde_json::to_vec(&prepared).unwrap()).unwrap();
            }
            "missing" => fs::remove_file(&retained).unwrap(),
            "corrupted" => fs::write(&retained, "corrupted prepared archive").unwrap(),
            _ => unreachable!(),
        }
        let final_path = fixture.repo.receipt_path("0.13.0", false).unwrap();
        let final_bytes = fs::read(&final_path).unwrap();
        let final_evidence: PackageEvidence = serde_json::from_slice(&final_bytes).unwrap();
        let final_archive = fixture
            .repo
            .retained_package(&final_evidence.package_sha256)
            .unwrap();
        let archive_bytes = fs::read(&final_archive).unwrap();
        let working_archive = fixture.repo.package_path("0.13.0").unwrap();
        let working_bytes = fs::read(&working_archive).unwrap();
        let prepared_bytes = fs::read(&prepared_path).unwrap();
        let call_start = fixture.repo.exec.state.borrow().calls.len();

        assert!(fixture.repo.qualify_release().is_err(), "{changed}");
        assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
        assert_eq!(fs::read(&final_archive).unwrap(), archive_bytes);
        assert_eq!(fs::read(&working_archive).unwrap(), working_bytes);
        assert_eq!(fs::read(&prepared_path).unwrap(), prepared_bytes);
        assert!(
            !fixture.repo.exec.state.borrow().calls[call_start..]
                .iter()
                .any(|call| call.get(2).is_some_and(|arg| arg == "package"))
        );
    }
}

#[test]
fn final_package_failure_is_retryable_without_a_second_commit() {
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    fixture.fail("package");
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().release.is_some());
    assert!(!fixture.repo.exec.state.borrow().tag);
    assert!(!fixture.repo.receipt_path("0.13.0", false).unwrap().exists());
    fixture.repo.exec.state.borrow_mut().fail = None;
    fixture.commit().unwrap();
    assert_eq!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .filter(|call| call.get(1).is_some_and(|arg| arg == "commit"))
            .count(),
        1
    );
}

#[test]
fn head_change_during_final_packaging_preserves_prepared_evidence() {
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    let path = fixture.repo.receipt_path("0.13.0", true).unwrap();
    let prepared_bytes = fs::read(&path).unwrap();
    let prepared: PackageEvidence = serde_json::from_slice(&prepared_bytes).unwrap();
    let archive = fixture
        .repo
        .retained_package(&prepared.package_sha256)
        .unwrap();
    let archive_bytes = fs::read(&archive).unwrap();
    fixture.fail("package-head");
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().descendant.is_some());
    assert!(!fixture.repo.exec.state.borrow().tag);
    assert!(!fixture.repo.receipt_path("0.13.0", false).unwrap().exists());
    assert_eq!(fs::read(&path).unwrap(), prepared_bytes);
    assert_eq!(fs::read(&archive).unwrap(), archive_bytes);
}

#[test]
fn selected_older_release_reuses_exact_receipts_and_archives_after_a_fix() {
    let fixture = Fixture::new();
    fixture.release();
    let prepared_path = fixture.repo.receipt_path("0.13.0", true).unwrap();
    let final_path = fixture.repo.receipt_path("0.13.0", false).unwrap();
    let prepared_bytes = fs::read(&prepared_path).unwrap();
    let final_bytes = fs::read(&final_path).unwrap();
    let evidence: PackageEvidence = serde_json::from_slice(&final_bytes).unwrap();
    let archive = fixture
        .repo
        .retained_package(&evidence.package_sha256)
        .unwrap();
    let archive_bytes = fs::read(&archive).unwrap();
    let mut descendant = fixture.repo.exec.state.borrow().release.clone().unwrap();
    descendant.insert(
        "source.rs".to_owned(),
        "// committed callback fix\n".to_owned(),
    );
    for (path, text) in &descendant {
        fs::write(fixture.repo.root.join(path), text).unwrap();
    }
    fixture.repo.exec.state.borrow_mut().descendant = Some(descendant);
    fs::write(
        fixture.repo.package_path("0.13.0").unwrap(),
        "newer working archive",
    )
    .unwrap();
    let calls = fixture.repo.exec.state.borrow().calls.len();
    fixture
        .repo
        .selected_committed_check(&fixture.selection, "release")
        .unwrap();
    fixture
        .repo
        .tagged_check(&fixture.selection, "release")
        .unwrap();
    fixture
        .repo
        .push_check(&fixture.selection, "release")
        .unwrap();
    assert!(fixture.repo.release_commit().is_err());
    assert!(
        fixture
            .repo
            .selected_committed_check(&fixture.selection, "fix")
            .is_err()
    );
    assert_eq!(fs::read(&prepared_path).unwrap(), prepared_bytes);
    assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
    assert_eq!(fs::read(&archive).unwrap(), archive_bytes);
    assert!(
        !fixture.repo.exec.state.borrow().calls[calls..]
            .iter()
            .any(|call| {
                call[0] == "make"
                    || call[0] == "cargo"
                        && call.get(2).is_some_and(|arg| {
                            matches!(arg.as_str(), "package" | "publish" | "update")
                        })
            })
    );

    for conflict in ["history", "parent", "tag-commit"] {
        fixture.fail(conflict);
        assert!(
            fixture
                .repo
                .push_check(&fixture.selection, "release")
                .is_err(),
            "{conflict}"
        );
        assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
    }
    fixture.repo.exec.state.borrow_mut().fail = None;
    fs::write(&archive, "corrupted retained archive").unwrap();
    assert!(
        fixture
            .repo
            .push_check(&fixture.selection, "release")
            .is_err()
    );
    fs::write(&archive, archive_bytes).unwrap();
    fs::remove_file(&final_path).unwrap();
    let calls = fixture.repo.exec.state.borrow().calls.len();
    assert!(
        fixture
            .repo
            .selected_committed_check(&fixture.selection, "release")
            .is_err()
    );
    assert!(!final_path.exists());
    assert!(
        !fixture.repo.exec.state.borrow().calls[calls..]
            .iter()
            .any(|call| call.get(2).is_some_and(|arg| arg == "package"))
    );
}

#[test]
fn real_index_rejects_hidden_staging_and_checks_exact_prepared_bytes() {
    let fixture = Fixture::new();
    let checkout = fixture.repo.root.join("index-checkout");
    Processes
        .run(
            &fixture.repo.root,
            "git",
            &[
                "clone",
                "--quiet",
                "--shared",
                "--no-checkout",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .to_str()
                    .unwrap(),
                checkout.to_str().unwrap(),
            ],
            true,
        )
        .unwrap();
    let repo = Repository {
        root: checkout,
        exec: Processes,
    };
    let mut selection = fixture.selection.clone();
    selection.source = repo.git(&["rev-parse", "HEAD"]).unwrap();
    repo.git(&["read-tree", "HEAD"]).unwrap();
    // Give this real index fixture a tracked lock in an uncommitted tree object.
    // Existing repository history is reused; no commit or tag is created.
    fs::write(repo.root.join("Cargo.lock"), LOCK).unwrap();
    repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
    selection.source = repo.git(&["write-tree"]).unwrap();
    repo.git(&["read-tree", &selection.source]).unwrap();
    let original = repo.text("README.md", Some("HEAD")).unwrap();
    let expected = format!("{original}\nPrepared release fixture.\n");
    let locked = replace_lock_version(LOCK, "0.12.3", "0.13.0").unwrap();
    let surfaces = BTreeMap::from([
        ("README.md".to_owned(), expected.clone()),
        ("Cargo.lock".to_owned(), locked.clone()),
    ]);
    repo.check_index(&selection, &surfaces).unwrap();

    fs::write(repo.root.join("README.md"), &original).unwrap();
    repo.git(&["update-index", "--chmod=+x", "README.md"])
        .unwrap();
    assert!(repo.check_index(&selection, &surfaces).is_err());
    repo.git(&["read-tree", &selection.source]).unwrap();

    fs::write(repo.root.join("unrelated.rs"), "unrelated index change").unwrap();
    repo.git(&["add", "--", "unrelated.rs"]).unwrap();
    fs::remove_file(repo.root.join("unrelated.rs")).unwrap();
    assert!(repo.check_index(&selection, &surfaces).is_err());
    repo.git(&["read-tree", &selection.source]).unwrap();

    for (staged, accepted) in [
        ("arbitrary staged metadata".to_owned(), false),
        (expected, true),
        (format!("{original} "), false),
    ] {
        fs::write(repo.root.join("README.md"), staged).unwrap();
        repo.git(&["add", "--", "README.md"]).unwrap();
        fs::write(repo.root.join("README.md"), &original).unwrap();
        assert_eq!(repo.check_index(&selection, &surfaces).is_ok(), accepted);
        repo.git(&["read-tree", &selection.source]).unwrap();
    }
    for (staged, accepted) in [
        (locked.clone(), true),
        (
            locked.replace(
                "name = \"other\"\nversion = \"0.12.3\"",
                "name = \"other\"\nversion = \"0.12.4\"",
            ),
            false,
        ),
        (format!("{locked}# arbitrary staged text\n"), false),
    ] {
        fs::write(repo.root.join("Cargo.lock"), staged).unwrap();
        repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
        fs::write(repo.root.join("Cargo.lock"), LOCK).unwrap();
        assert_eq!(repo.check_index(&selection, &surfaces).is_ok(), accepted);
        repo.git(&["read-tree", &selection.source]).unwrap();
    }
}

#[test]
fn wasm_metadata_and_budgets_use_only_the_selected_artifact_directory() {
    let fixture = Fixture::new();
    let target = fixture.repo.target().unwrap();
    let artifacts = target.join("wasm32-unknown-unknown/wasm-size/examples");
    fs::create_dir_all(&artifacts).unwrap();
    for (probe, budget) in PROBES {
        let path = artifacts.join(format!("wasm_{probe}_size_probe.wasm"));
        let file = fs::File::create(path).unwrap();
        file.set_len(budget).unwrap();
    }
    check_wasm_artifacts(&target).unwrap();
    let path = artifacts.join("wasm_core_size_probe.wasm");
    fs::File::create(&path).unwrap().set_len(260_001).unwrap();
    assert!(check_wasm_artifacts(&target).is_err());
    fs::remove_file(path).unwrap();
    assert!(check_wasm_artifacts(&target).is_err());
    assert!(target_directory("{}").is_err());
    fixture.fail("metadata");
    assert!(fixture.repo.target().is_err());
}

#[test]
fn patch_preparation_keeps_head_and_rejects_unvalidated_source_or_staging() {
    let mut fixture = Fixture::new();
    let path = "CHANGELOG.md";
    let notes = fixture.repo.exec.state.borrow().source[path].replace("[0.13.0]", "[0.12.4]");
    fs::write(fixture.repo.root.join(path), &notes).unwrap();
    fixture
        .repo
        .exec
        .state
        .borrow_mut()
        .source
        .insert(path.to_owned(), notes);
    fixture.selection.kind = "patch".to_owned();
    fixture.selection.version = "0.12.4".to_owned();
    let detail = "docs/changelog/0.12.md";
    let notes = "# Detail\n\n## [0.12.4]\n\nFixture changes.\n";
    fs::write(fixture.repo.root.join(detail), notes).unwrap();
    fixture
        .repo
        .exec
        .state
        .borrow_mut()
        .source
        .insert(detail.to_owned(), notes.to_owned());
    fixture.prepare().unwrap();
    assert!(fixture.repo.exec.state.borrow().release.is_none());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.4");
    assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
        call.get(1)
            .is_some_and(|arg| matches!(arg.as_str(), "commit" | "tag" | "push"))
    }));
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    assert!(fixture.commit().is_err()); // Expected edits are not staged.
    fixture.stage().unwrap();
    fs::write(
        fixture.repo.root.join("source.rs"),
        "// unvalidated source\n",
    )
    .unwrap();
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().release.is_none());
    let fixture = Fixture::new();
    fixture.fail("remote-ahead");
    assert!(fixture.prepare().is_err());
    fixture.assert_original();
}
