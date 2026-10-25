# Continuous integration

[CI](../.github/workflows/ci.yml) runs on pull requests, pushes to `main`, and manual dispatch.
Its final **CI** job succeeds only when every required job succeeds:

| Job | Checks |
| --- | --- |
| Formatting | A pinned nightly rustfmt with the repository configuration |
| Clippy | All targets and features with warnings denied |
| Tests | All targets, unit tests, integration tests, and doctests with default and all features |
| Minimum version | All targets with Rust 1.98.0 and all features |
| Feature powerset | Every feature combination with dev dependencies removed |
| Documentation | Default and all-feature rustdoc with warnings denied, plus the doctest powerset |

The all-feature runs enable `--cfg tokio_unstable` because Tokio's schedule-latency feature requires it.
The all-feature test job also runs the regression tests in release mode to exercise float counter validation
without debug assertions. Jobs that build protobuf test dependencies install `protoc`.

Once the workflow has run on GitHub, require the **CI** status check in the `main` branch ruleset.
The workflow defines the check; required-check enforcement is a repository setting. The existing release
workflow continues to run on pushes to `main`.

## Lint and formatting policy

Clippy runs on stable Rust with its default lint groups and `-D warnings`, which also rejects compiler
warnings. Pedantic, nursery, and restriction groups are not enabled globally. Keep lint exceptions
local to the code that needs them; the core crate already allows several numeric-cast lints.

Formatting uses `nightly-2026-10-24`, pinned separately from the build and test toolchains. Nightly
enables the custom options in [`rustfmt.toml`](../rustfmt.toml): merge imports by module, group standard
library/external/local imports, format Rust examples in doc comments, and leave generated files alone.
See the [rustfmt configuration reference](https://rust-lang.github.io/rustfmt/) for these options.

```sh
rustup toolchain install nightly-2026-10-24 --profile minimal --component rustfmt
cargo +nightly-2026-10-24 fmt --all
cargo +nightly-2026-10-24 fmt --all -- --check
```

Use that same pinned formatter locally and in CI. Update the pin in the workflow, configuration comment,
and this guide together when adopting a new formatter version.

## Scheduled quality checks

[Scheduled quality checks](../.github/workflows/quality.yml) runs on Mondays at 04:17 UTC and can
also be dispatched manually. It produces workspace LLVM line coverage with all features as LCOV
and browsable HTML, with artifacts retained for 14 days.

## Run checks locally

The workflows contain the exact commands. Install Rust 1.98.0 and stable, `protoc`, and these tool versions
when reproducing their jobs:

```sh
cargo install cargo-hack --version 0.6.45 --locked
cargo install cargo-llvm-cov --version 0.9.1 --locked
rustup component add llvm-tools-preview
```
