# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Breaking Changes

- [#9][9]`Value::from_serialize` now returns a `Result`. Previously, if a Serialize
  implementation returned an error, the function would panic. Thanks to
  @wackbyte for finding this.

### Changed

- `pot::Result<T>` is now `pot::Result<T,E = pot::Error>`. This avoids issues
  with other code when `pot::Result` is imported.

[9]: https://github.com/khonsulabs/pot/issues/9

## v2.0.0 (2023-02-28)

### Breaking Changes

- The `format` module has been refactored to pass `Write` by value rather than
  by mutable reference. Most code should not be affected because `Write` is
  implemented for `&mut Write`.

### Changed

- The unit type `()` and `Option::None` are more fuzzy when deserializing. If
  users deserialize a value that was serialized as `None` or `()`, the default
  value will be returned rather than an error, when possible. For example:

  ```rust
  let unit = pot::to_vec(&())?;
  assert_eq!(pot::from_slice(&unit).unwrap(), 0_u32)
  let none = pot::to_vec(&Option::<bool>::None)?;
  assert_eq!(pot::from_slice(&unit).unwrap(), 0_u32)
  ```

  This is not practically useful for most users, but when designing traits that
  have associated serializable types, sometimes it's useful to use `()` when no
  data needs to be stored. However, it can be painful to update existing data
  when switching between `()` and other types, as Serde offers no built-in
  transmutation. Pot now offers this internally.

### Added

- `Value::from_serialize` and `Value::deserialize_as` have been added, allowing
  `Value` to be transmuted directly from types that implement `Serialize` and
  `Deserialize`.
- `OwnedValue` is a new-type wrapper around `Value<'static>` that can be used in
  situations where `DeserializeOwned` is a requirement. This type is needed
  because `Value<'a>` can borrow from the source of the deserialization, and
  this flexibility causes lifetime errors when trying to deserialize a
  `Value<'static>` as `DeserializeOwned`.

## v1.0.2 (2022-04-09)

### Fixed

- [#5][5]: Removed `release_max_level_off` feature flag.

[5]: https://github.com/khonsulabs/pot/issues/5

## v1.0.1 (2022-02-10)

### Added

- `serde(flatten)` is now supported.

## v1.0.0 (2022-02-04)

- There were no changes in this release.

## v1.0.0-rc.4 (2022-01-25)

### Changed

- Fixed compilation error caused by a new dependency upgrade.

## v1.0.0-rc.3 (2021-12-31)

### Changed

- `from_reader` and `Config::deserialize_from` to `DeserializeOwned`. This
  prevents errors at compile time that would arise at runtime when deserializing
  instead.

## v1.0.0-rc.2 (2021-12-27)

### Changed

- `Config` now implements `Clone` and `Debug`.

## v1.0.0-rc.1 (2021-12-23)

### Added

- Added the `Value` type, allowing deserializing arbitrary Pot payloads without its
  original data structure.

### Fixed

- Small fixes when packing floats and integers. No breaking changes, as the
  incorrect code would just use more space than needed for certain values.

## v0.1.0-alpha.3 (2021-12-10)

### Fixed

- Fixed deserializing unit enum variants.

## v0.1.0-alpha.2 (2021-10-28)

### Breaking Changes

- A minor format change was necessary to add full `deserialize_any()` support.

## v0.1.0-alpha.1 (2021-10-27)

- First release.
