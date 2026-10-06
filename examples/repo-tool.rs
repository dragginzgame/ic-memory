//! Repository-owned release and Wasm qualification tooling.
//!
//! This binary is built as a development example and is never part of canister
//! runtime code. Git commits, tags and pushes are maintainer-only commands.

#[cfg(not(target_arch = "wasm32"))]
mod repo_tool;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(error) = repo_tool::main() {
        eprintln!("repo-tool: {error}");
        std::process::exit(1);
    }
}

#[cfg(target_arch = "wasm32")]
// This host-only example is included by the workspace's Wasm test check.
fn main() {}
