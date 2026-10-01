# Repository Instructions

## User-Owned Commits and Pushes

The user exclusively owns committing and pushing. LLM agents must never create,
amend, or rewrite commits, push branches or tags, or invoke scripts that perform
those actions. Requests to prepare a release or push changes do not authorize an
agent to perform these user-owned actions.

- Leave edits unstaged and uncommitted for the user to review.
- Do not create, move, or delete release tags.
- Do not run `make release-commit`, `make release-push`, `make release-minor`, or
  `make release-patch`; these commands commit, tag, or push.
- Prepare source, documentation, and validation within the working tree, then
  report the result and any commands the user needs to run.
- If a workflow requires committed source or a clean working tree, stop before
  that prerequisite and explain the required user action. Never commit or push
  merely to satisfy a workflow's preconditions.

## Pre-1.0 Hard-Cut Policy

Until `1.0.0`, every release uses the current API and durable format only. Do
not preserve backward compatibility with an earlier pre-1.0 release.

- Do not add deprecated forwarders, compatibility shims, renamed aliases,
  legacy modules, or old macro forms.
- Do not add serde field aliases, version-routed legacy decoders, fallback
  readers, compatibility fixtures, or defaults whose purpose is to accept an
  earlier wire shape.
- When an API or format changes, remove the superseded path in the same change
  and update current fixtures, documentation, and downstream callers directly.
- Historical changelogs and archived design documents may describe removed
  behavior, but executable code and current documentation must not retain it.
- Negative compile-fail tests may reference removed forms only to prove that
  they remain rejected.

## Rust Item Documentation Style

For public structs, traits, and enums, prefer a wrapped rustdoc block with the
item name as a short heading, a blank rustdoc line before and after the heading,
and a blank source line before attributes:

```rust
///
/// SchemaMetadata
///
/// Optional diagnostic metadata for an in-place store schema.
///
/// This metadata helps humans and frameworks diagnose which schema version was
/// declared in each generation. It is bounded and validated for durable ledger
/// encoding, but it does not perform application schema migrations or validate
/// stable data semantics.
///

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaMetadata {
    /// Optional in-place schema version.
    pub schema_version: Option<u32>,
}
```

Keep this shape for new or edited item-level docs unless local context clearly
requires a different style.
