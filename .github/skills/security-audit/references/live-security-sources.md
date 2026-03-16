# Live Security Sources

Fetch these sources when the review depends on current external guidance or when repository policy is silent on a point.

## Rust And Dependency Security

- RustSec overview and advisory database: <https://rustsec.org/>
- `cargo-deny` documentation: <https://embarkstudios.github.io/cargo-deny/>
- `cargo-audit` repository and README: <https://github.com/RustSec/rustsec/tree/main/cargo-audit>
- Rust book: <https://doc.rust-lang.org/book/title-page.html>

## What To Pull From Them

- Use RustSec to confirm whether a vulnerability is published, yanked, or unmaintained status exists.
- Use `cargo-deny` docs to confirm how advisories, bans, licenses, and source policy checks should complement each other.
- Use `cargo-audit` docs for current scanner behavior and lockfile-focused advisory workflow.
- Use the Rust book as a language-level baseline when reviewing ownership, error handling, path handling, and command execution patterns.
