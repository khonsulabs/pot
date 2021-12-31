# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v1.0.0-rc.3

### Changed

- `from_reader` and `Config::deserialize_from` to `DeserializeOwned`. This
  prevents errors at compile time that would arise at runtime when deserializing
  instead.

## v1.0.0-rc.2

### Changed

- `Config` now implements `Clone` and `Debug`.

## v1.0.0-rc.1

### Added

- Added the `Value` type, allowing deserializing arbitrary Pot payloads without its
  original data structure.

### Fixed

- Small fixes when packing floats and integers. No breaking changes, as the
  incorrect code would just use more space than needed for certain values.

## v0.1.0-alpha.3

### Fixed

- Fixed deserializing unit enum variants.

## v0.1.0-alpha.2

### Breaking Changes

- A minor format change was necessary to add full `deserialize_any()` support.

## v0.1.0-alpha.1

- First release.
