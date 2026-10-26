# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/conradludgate/measured/compare/measured-derive-v0.0.25...measured-derive-v0.1.0) - 2026-10-26

### Fixed

- fix derive code generation and validate label definitions

### Other

- make each crate documentation self-contained and clarify API contracts
- add metric units through non-exhaustive FamilyMeta
- document the 0.1 release with tutorials, guides and migration notes
- use NumBuffer and zmij and require Rust 1.98
- add non-inserting metric lookup, iteration and projection
- redesign encoding around metric families and sample kinds
- establish nightly rustfmt configuration and format the workspace

### Breaking

- The minimum supported Rust version is now 1.98.
- `LabelGroup` set constructors take arguments in field declaration order. Previously `dynamic_with` fields came first.
- `LabelGroup` rejects duplicate label names, including renames.
- `FixedCardinalityLabel` rejects duplicate label values and enums with no variants.
- `#[metric(init)]`, `#[metric(metadata)]` and `#[metric(label_set)]` require `#[metric(new(...))]` on the struct, instead of being silently ignored.

### Added

- `#[derive(LabelGroup)]` no longer requires `#[label(set = ...)]`.
- Generated `LabelGroupSet` impls implement `find`, for the new `LabelGroupSet::find` method.
- `#[metric(unit = "...")]` declares metric-family units and checks the metric-name suffix.

### Fixed

- Generated code uses fully qualified paths, so it compiles when `Result` or `Option` are shadowed.
- `#[doc(hidden)]` and non-literal doc attributes are accepted on `MetricGroup` fields.
- Metric and label renames that differ only in case no longer collide, and invalid renames no longer panic the macro.
- Raw identifiers, eg `r#type`, are accepted, and named without `r#`.

See the `measured` changelog for the rest of this release.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-derive-v0.0.24...measured-derive-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
