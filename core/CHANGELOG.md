# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking

- Label-group set constructor arguments now follow declaration order. Empty label enums and duplicate names or values are rejected; initialization attributes require `metric(new(...))`.
- All histogram threshold constructors reject NaN, explicit +Inf, overflow into +Inf, and non-increasing bounds.
- The text encoder moved from `measured::text` to `measured-prometheus-text`. `TextEncoder::new` becomes `from_writer`; the writer is private with accessors. `BufferedTextEncoder` is now an alias with `io::Error` instead of `Infallible`. `MetricValue` is removed, `MetricType` is non-exhaustive, and `write_type` takes names by value. The Axum example uses conventional metric suffixes and supplies Content-Type.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-v0.0.24...measured-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
- Merge pull request #13 from ultrabear/expose_lasso
- re-export lasso when enabled so crate users do not have to add it themselves
- enable auto_cfg
- make measured docs build again
