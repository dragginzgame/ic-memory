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
        ]);
        for (path, text) in &source {
            fs::write(root.join(path), text).unwrap();
        }
        fs::write(root.join("Cargo.lock"), LOCK).unwrap();
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
                remote_collision: false,
            }),
        };
        Self {
            repo: Repository { root, exec },
        }
    }

    fn release(&self) {
        self.repo.prepare("minor").unwrap();
        self.repo.stage().unwrap();
        self.repo.commit().unwrap();
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
    remote_collision: bool,
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
            "shasum" => {
                // Hashes use the real, portable process boundary rather than an
                // invented checksum. This is the only external fixture command.
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
                    "version = \"0.12.3\"",
                    &format!("version = \"{}\"", version()?),
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
            "cargo" if args.get(1) == Some(&"package") => {
                let directory = target.join("package");
                fs::create_dir_all(&directory)?;
                // Model Cargo's Git metadata changing after the release commit.
                let bytes = if state.release.is_some() {
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
                ["status", ..] => changed(state.release.as_ref().unwrap_or(&state.source)),
                ["rev-parse", "HEAD"] => Ok(if state.release.is_some() {
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
                        "source" => state
                            .source
                            .get(path)
                            .cloned()
                            .ok_or_else(|| "missing fixture source".into()),
                        "HEAD" => state
                            .release
                            .as_ref()
                            .and_then(|files| files.get(path))
                            .cloned()
                            .ok_or_else(|| "missing fixture release".into()),
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
                ["fetch" | "merge-base" | "ls-files" | "push", ..] => Ok(String::new()),
                ["ls-remote", ..] => Ok(if state.remote_collision {
                    "other-tag refs/tags/v0.13.0"
                } else {
                    ""
                }
                .to_owned()),
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
                ["add", ..] => {
                    state.staged = true;
                    Ok(String::new())
                }
                ["commit", ..] => {
                    state.release = Some(
                        state
                            .source
                            .keys()
                            .map(|path| Ok((path.clone(), fs::read_to_string(root.join(path))?)))
                            .collect::<Result<_>>()?,
                    );
                    state.staged = false;
                    Ok(String::new())
                }
                ["log", "-1", "--format=%s"] => Ok(if state.release.is_some() {
                    "Release 0.13.0"
                } else {
                    "Fixture source"
                }
                .to_owned()),
                ["log", "-1", "--format=%B"] => {
                    Ok("Release 0.13.0\n\nValidated-source: source".to_owned())
                }
                ["tag", ..] => {
                    state.tag = true;
                    Ok(String::new())
                }
                ["cat-file", "-t", ..] => Ok("tag".to_owned()),
                ["rev-parse", reference] if reference.ends_with("^{commit}") => {
                    Ok("release".to_owned())
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
        assert!(fixture.repo.prepare("minor").is_err());
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
        assert!(repo.push().is_err());
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
        .verify_evidence("source", "0.13.0", "release", false)
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
                assert!(fixture.repo.prepare("minor").is_err());
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
        .chain(["Cargo.lock", "rust-toolchain.toml"])
    {
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
                remote_collision: false,
            }),
        },
    };
    assert!(repo.prepare("minor").is_err());
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
fn release_versions_and_drafts_preserve_dependency_versions_and_history() {
    assert_eq!(next_version("0.12.3", "patch").unwrap(), "0.12.4");
    assert_eq!(next_version("0.12.3", "minor").unwrap(), "0.13.0");
    for invalid in ["0.01.3", "0.12", "0.12.3-dev", "0.12.+3"] {
        assert!(version_parts(invalid).is_err());
    }
    let updated = replace_package_version(MANIFEST, "0.12.3", "0.13.0").unwrap();
    assert!(updated.contains("other = \"0.12.3\""));
    let current = "# Changelog\n\n## [0.13.0]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n";
    assert_eq!(release_changelog(current, "0.13.0").unwrap(), current);
    assert!(release_changelog(current, "0.12.4").is_err());
    assert!(release_changelog("## [0.13.0]\n\n## 0.12.3\nHistory", "0.13.0").is_err());
    for historical in ["0.13.0", "[0.13.0]", "[0.13.0] - 2026-10-05"] {
        let duplicate = format!("{current}\n## {historical}\n\nAlready released.\n");
        assert!(release_changelog(&duplicate, "0.13.0").is_err());
    }
}

#[test]
fn preparation_stage_commit_qualification_and_publish_preserve_exact_evidence() {
    let fixture = Fixture::new();
    fixture.release();
    let prepared: ReleaseEvidence = serde_json::from_slice(
        &fs::read(fixture.repo.receipt_path("0.13.0", true).unwrap()).unwrap(),
    )
    .unwrap();
    let final_evidence = fixture
        .repo
        .verify_evidence("source", "0.13.0", "release", false)
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
    assert!(
        fixture
            .repo
            .root
            .join("configured output/release-validation/0.13.0-prepared.json")
            .is_file()
    );
    fixture.repo.push().unwrap();
    fixture.repo.publish(true).unwrap();
    let state = fixture.repo.exec.state.borrow();
    assert!(state.calls.iter().any(|call| call
        == &[
            "git",
            "push",
            "--no-follow-tags",
            "--atomic",
            "origin",
            "release:refs/heads/main",
            "refs/tags/v0.13.0:refs/tags/v0.13.0"
        ]));
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
        let receipt = fixture.repo.receipt_path("0.13.0", true).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&receipt, "previous evidence").unwrap();
        let sentinel = fixture.repo.target().unwrap().join("consumer-evidence");
        fs::write(&sentinel, "keep").unwrap();
        fixture.fail(failure);
        assert!(fixture.repo.prepare("minor").is_err());
        fixture.assert_original();
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
fn dirty_sources_remote_collisions_and_changed_validation_inputs_prevent_version_edits() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.root.join("README.md"), "unfinished").unwrap();
    assert!(fixture.repo.prepare("minor").is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
    let fixture = Fixture::new();
    fixture.repo.exec.state.borrow_mut().remote_collision = true;
    assert!(fixture.repo.prepare("minor").is_err());
    fixture.assert_original();
    let fixture = Fixture::new();
    fixture.fail("validation-lock");
    assert!(fixture.repo.prepare("minor").is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
}

#[test]
fn lockfile_changes_during_post_validation_remote_inspection_stop_before_mutation() {
    let fixture = Fixture::new();
    fixture.fail("remote-lock");
    assert!(fixture.repo.prepare("minor").is_err());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for (path, text) in &fixture.repo.exec.state.borrow().source {
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
            *text
        );
    }
    // Preserve the separately changed ignored file; it is not our version edit.
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
                let evidence: ReleaseEvidence =
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
                    evidence[field] = serde_json::json!({"env:RUSTFLAGS": "--cfg unqualified"});
                } else if field == "commands" {
                    evidence[field] = serde_json::json!([]);
                } else {
                    evidence[if field == "compiler" { "rustc" } else { field }] =
                        serde_json::json!("changed");
                }
                fs::write(receipt, serde_json::to_vec(&evidence).unwrap()).unwrap();
            }
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
        let mut prepared: ReleaseEvidence =
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
        let final_evidence: ReleaseEvidence = serde_json::from_slice(&final_bytes).unwrap();
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
    fixture.repo.prepare("minor").unwrap();
    fixture.repo.stage().unwrap();
    fixture.fail("package");
    assert!(fixture.repo.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().release.is_some());
    assert!(!fixture.repo.exec.state.borrow().tag);
    assert!(!fixture.repo.receipt_path("0.13.0", false).unwrap().exists());
    fixture.repo.exec.state.borrow_mut().fail = None;
    fixture.repo.commit().unwrap();
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
    let fixture = Fixture::new();
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
    fixture.repo.prepare("patch").unwrap();
    assert!(fixture.repo.exec.state.borrow().release.is_none());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.4");
    assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
        call.get(1)
            .is_some_and(|arg| matches!(arg.as_str(), "commit" | "tag" | "push"))
    }));
    let fixture = Fixture::new();
    fixture.repo.prepare("minor").unwrap();
    assert!(fixture.repo.commit().is_err()); // Expected edits are not staged.
    fixture.repo.stage().unwrap();
    fs::write(
        fixture.repo.root.join("source.rs"),
        "// unvalidated source\n",
    )
    .unwrap();
    assert!(fixture.repo.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().release.is_none());
    let fixture = Fixture::new();
    fixture.fail("remote-ahead");
    assert!(fixture.repo.prepare("minor").is_err());
    fixture.assert_original();
}
