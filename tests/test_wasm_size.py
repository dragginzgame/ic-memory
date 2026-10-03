"""Exercise the production Wasm budget recipe with disposable Cargo artifacts."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

REPO = Path(__file__).resolve().parent.parent
PROBES = (
    "core",
    "diagnostics",
    "key_only",
    "admission",
    "runtime_integration",
)


class WasmSizeTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="ic-memory-wasm-size-test-")
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)
        shutil.copyfile(REPO / "Makefile", self.work / "Makefile")
        fakebin = self.work / "bin"
        fakebin.mkdir()
        cargo = fakebin / "cargo"
        cargo.write_text('''#!/usr/bin/env python3
import json, os
from pathlib import Path
import sys, tomllib
root = Path.cwd()
config_path = root / '.cargo/config.toml'
config = tomllib.loads(config_path.read_text()) if config_path.exists() else {}
target = Path(os.environ.get('CARGO_TARGET_DIR', config.get('build', {}).get('target-dir', 'target')))
if not target.is_absolute():
    target = root / target
if 'metadata' in sys.argv:
    if os.environ.get('FAIL_METADATA') == '1':
        sys.exit(1)
    print(json.dumps({'target_directory': str(target)}))
elif 'build' in sys.argv:
    artifacts = target / 'wasm32-unknown-unknown/wasm-size/examples'
    artifacts.mkdir(parents=True, exist_ok=True)
    for name in json.loads(os.environ['WASM_TEST_PROBES']):
        if name != os.environ.get('MISSING_PROBE'):
            (artifacts / f'wasm_{name}_size_probe.wasm').write_bytes(bytes(int(os.environ.get('PROBE_BYTES', '1000'))))
else:
    sys.exit(2)
''')
        cargo.chmod(0o755)
        self.env = os.environ.copy()
        for variable in ("CARGO_TARGET_DIR", "MAKEFLAGS", "MFLAGS"):
            self.env.pop(variable, None)
        self.env["PATH"] = str(fakebin) + os.pathsep + self.env["PATH"]
        self.env["WASM_TEST_PROBES"] = json.dumps(PROBES)

    def run_budget(self, success=True, **variables):
        result = subprocess.run(
            ["make", "--no-print-directory", "wasm-size", "VALIDATION_TOOLCHAIN=test"],
            cwd=self.work,
            env={**self.env, **variables},
            text=True,
            capture_output=True,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertEqual(result.stdout.count("1000 bytes (budget:"), len(PROBES))
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        return result

    def stale_artifacts(self):
        artifacts = self.work / "target/wasm32-unknown-unknown/wasm-size/examples"
        artifacts.mkdir(parents=True)
        for name in PROBES:
            (artifacts / f"wasm_{name}_size_probe.wasm").write_bytes(b"old")

    def test_default_target_directory(self):
        self.run_budget()

    def test_environment_target_directories(self):
        for target in ("relative output", str(self.work / "absolute output")):
            with self.subTest(target=target):
                self.run_budget(CARGO_TARGET_DIR=target)

    def test_cargo_config_target_directory(self):
        (self.work / ".cargo").mkdir()
        (self.work / ".cargo/config.toml").write_text('[build]\ntarget-dir = "configured output"\n')
        self.run_budget()

    def test_stale_artifacts_cannot_hide_oversized_build(self):
        self.stale_artifacts()
        result = self.run_budget(
            success=False, CARGO_TARGET_DIR="new output", PROBE_BYTES="400000"
        )
        self.assertIn("400000 bytes", result.stdout)

    def test_stale_artifacts_cannot_hide_missing_probe(self):
        self.stale_artifacts()
        self.run_budget(success=False, CARGO_TARGET_DIR="new output", MISSING_PROBE="core")

    def test_metadata_failure_cannot_use_stale_artifacts(self):
        self.stale_artifacts()
        self.run_budget(success=False, CARGO_TARGET_DIR="new output", FAIL_METADATA="1")


if __name__ == "__main__":
    unittest.main()
