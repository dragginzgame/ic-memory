//! Command effects are substituted; no Git mutations or live publication occur.

use super::*;
use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
const MANIFEST: &str = "[package]\nname = \"ic-memory\"\nversion = \"0.12.3\"\nrust-version = \"1.88.0\"\n[dependencies]\nother = \"0.12.3\"\n";
const LOCK: &str = "version = 4\n[[package]]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n[[package]]\nname = \"other\"\nversion = \"0.12.3\"\n";

struct Fixture {
    repo: Repository<Substitute>,
    selection: ReleaseSelection,
}

impl Fixture {
    fn new() -> Self {
        let root = env::temp_dir().join(format!(
            "ic-memory-tooling-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let source = BTreeMap::from([
            ("Cargo.lock".to_owned(), LOCK.to_owned()),
            ("Cargo.toml".to_owned(), MANIFEST.to_owned()),
            (
                "README.md".to_owned(),
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
        fs::remove_dir_all(&self.repo.root).unwrap();
    }
}

#[derive(Deserialize, Serialize)]
struct State {
    source: BTreeMap<String, String>,
    release: Option<BTreeMap<String, String>>,
    staged: bool,
    calls: Vec<Vec<String>>,
    fail: Option<String>,
    tag: bool,
    descendant: Option<BTreeMap<String, String>>,
}

struct Substitute {
    state: RefCell<State>,
}

impl Execute for Substitute {
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
                "package",
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
            "bash" => Processes.run(Path::new(env!("CARGO_MANIFEST_DIR")), program, args, true),
            "shasum" => {
                // Hashes use the real, portable process boundary rather than an
                // invented checksum. The other external command is the read-only version helper.
                Processes.run(root, program, args, true)
            }
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
            "cargo" if args.get(1) == Some(&"publish") => Ok(String::new()),
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
                ["status", ..] => changed(
                    state
                        .descendant
                        .as_ref()
                        .or(state.release.as_ref())
                        .unwrap_or(&state.source),
                ),
                ["rev-parse", "HEAD"] => Ok(if state.descendant.is_some() {
                    "fix"
                } else if state.release.is_some() {
                    "release"
                } else {
                    "source"
                }
                .to_owned()),
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
                ["for-each-ref", ..] => Ok(if state.tag {
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
fn release_versions_and_pending_notes_preserve_dependency_versions_and_history() {
    for invalid in ["0.01.3", "0.12", "0.12.3-dev", "0.12.+3"] {
        assert!(version_parts(invalid).is_err());
    }
    let updated = replace_package_version(MANIFEST, "0.12.3", "0.13.0").unwrap();
    assert!(updated.contains("other = \"0.12.3\""));
    let current = "# Changelog\n\n## [0.13.0]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n";
    assert_eq!(
        release_changelog(current, "0.13.0", "2026-10-05").unwrap(),
        current.replacen("## [0.13.0]", "## [0.13.0] - 2026-10-05", 1)
    );
    assert!(release_changelog(current, "0.12.4", "2026-10-05").is_err());
    assert!(
        release_changelog("## [0.13.0]\n\n## 0.12.3\nHistory", "0.13.0", "2026-10-05").is_err()
    );
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
                "partial" => path == "README.md",
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
            "metadata" => fixture.repo.root.join("README.md"),
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
                "README.md",
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
fn dirty_sources_and_changed_validation_inputs_prevent_version_edits() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.root.join("README.md"), "unfinished").unwrap();
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
                env!("CARGO_MANIFEST_DIR"),
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
