# Documentation

The user documentation is maintained as rustdoc under `core/src/docs`. Its entry point is
[`docs.rs`](../core/src/docs.rs), with four paths based on [Diátaxis](https://diataxis.fr/):

| Reader's need | Where to write |
| --- | --- |
| Learn through a guided, working example | [`tutorial.rs`](../core/src/docs/tutorial.rs) |
| Complete a specific task | [`how_to.rs`](../core/src/docs/how_to.rs) |
| Look up an API, option, or contract | [`reference.rs`](../core/src/docs/reference.rs) and the relevant public item |
| Understand a concept or tradeoff | [`explanation.rs`](../core/src/docs/explanation.rs), [`guide.rs`](../core/src/docs/guide.rs), [`details.rs`](../core/src/docs/details.rs) |

Keep the tutorial on one path with a visible result. Recipes should state prerequisites and
give steps for a named task. Put signatures, exhaustive options, and edge-case contracts next
to the API. Explain design choices separately, linking from the procedures when useful.
Keep breaking-change instructions in the crate changelogs.

Use runnable Rust examples. State any required Cargo features, and gate feature-dependent
doctests so default-feature documentation still builds. Avoid links to optional dependencies
that disappear when their features are disabled; use their public documentation URLs instead.

Validate documentation with:

```sh
cargo test -p measured --no-default-features --doc
RUSTFLAGS='--cfg tokio_unstable' cargo test --workspace --all-features --doc
RUSTDOCFLAGS='-D warnings' cargo doc -p measured --no-default-features --no-deps
RUSTFLAGS='--cfg tokio_unstable' RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps
```

With `cargo-hack` installed, check every doctest feature combination with:

```sh
RUSTFLAGS='--cfg tokio_unstable' RUSTDOCFLAGS='-D warnings' cargo hack test --workspace --exclude axum-example --feature-powerset --doc
```

The Axum example is a binary without a library doctest target, so this command excludes it.

The diagrams in this directory illustrate storage layouts; their `.d2` files are the editable sources.

For the CI jobs, required-check setup, and scheduled coverage runs, see [Continuous integration](ci.md).
