# Pot: The storage and network serialization format for BonsaiDb

![`Pot` is considered experimental and unsupported](https://img.shields.io/badge/status-experimental-blueviolet)
[![crate version](https://img.shields.io/crates/v/pot.svg)](https://crates.io/crates/pot)
[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/pot/Tests/main)](https://github.com/khonsulabs/pot/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/pot/coverage/badge.svg)](https://khonsulabs.github.io/pot/coverage/)
[![Documentation for `main` branch](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/pot/main/pot/)

`Pot` is an encoding format used within `BonsaiDb`. Its purpose is to provide an encoding format for `serde` that:

* Is self-describing.
* Is safe to run in production.
* Is compact.
  
  ```sh
  $ cargo test --example logs -- average_sizes --nocapture
  Generating 1000 LogArchies with 100 entries.
  +-----------------+------------+
  | Format          | Avg. Bytes |
  +-----------------+------------+
  | pot             | 26,642.383 |
  +-----------------+------------+
  | bincode(varint) | 25,361.761 |
  +-----------------+------------+
  | bincode         | 27,855.579 |
  +-----------------+------------+
  | cbor            | 31,025.765 |
  +-----------------+------------+
  ```

Because benchmarks can be subjective and often don't mirror real-world usage, this library's authors aren't making any specific performance claims. Thorough benchmarks will be written eventually, the way pot achieves space savings requires some computational overhead. Thus, it is expected that a theoretically perfect CBOR implementation could outperform a perfect Pot implementation.

## Status of Project

This project is still experimental, but the authors of `BonsaiDb` have elected to adopt it as the default storage format.
