# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking

- The minimum supported Rust version is now 1.98.
- `LabelGroup` set constructors take arguments in field declaration order. Previously `dynamic_with` fields came first.
- `LabelGroup` rejects duplicate label names, including renames.
- `FixedCardinalityLabel` rejects duplicate label values and enums with no variants.
- `#[metric(init)]`, `#[metric(metadata)]` and `#[metric(label_set)]` require `#[metric(new(...))]` on the struct, instead of being silently ignored.

### Added

- `#[derive(LabelGroup)]` no longer requires `#[label(set = ...)]`.
- Generated `LabelGroupSet` impls implement `find`, for the new `LabelGroupSet::find` method.

### Fixed

- Generated code uses fully qualified paths, so it compiles when `Result` or `Option` are shadowed.
- `#[doc(hidden)]` and non-literal doc attributes are accepted on `MetricGroup` fields.
- Metric and label renames that differ only in case no longer collide, and invalid renames no longer panic the macro.
- Raw identifiers, eg `r#type`, are accepted, and named without `r#`.

See the `measured` changelog for the rest of this release.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-derive-v0.0.24...measured-derive-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
