//! Repository-owned release and Wasm qualification tooling.
//!
//! This binary is built as a development example and is never part of canister
//! runtime code. Git commits, tags and pushes are maintainer-only commands.

mod repo_tool;

fn main() {
    if let Err(error) = repo_tool::main() {
        eprintln!("repo-tool: {error}");
        std::process::exit(1);
    }
}
