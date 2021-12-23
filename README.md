# Pot

A concise storage format, written for [`BonsaiDb`](https://bonsaidb.io/).

![`Pot` forbids unsafe code](https://img.shields.io/badge/unsafe-forbid-success)
[![crate version](https://img.shields.io/crates/v/pot.svg)](https://crates.io/crates/pot)
[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/pot/Tests/main)](https://github.com/khonsulabs/pot/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/pot/coverage/badge.svg)](https://khonsulabs.github.io/pot/coverage/)
[![Documentation for `main` branch](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/pot/main/pot/)

`Pot` is an encoding format used within [`BonsaiDb`](https://bonsaidb.io/). Its purpose is to
provide an encoding format for [`serde`](https://serde.rs) that:

* Is self-describing.
* Is safe to run in production.
* Is compact. While still being self-describing, Pot's main space-saving feature
  is not repeating symbols/identifiers more than one time while serializing.
  When serializing arrays of structures, this can make a major difference. The
  [logs.rs](https://github.com/khonsulabs/pot/blob/main/pot/examples/logs.rs)
  example demonstrates this:
  
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

## Benchmarks

Because benchmarks can be subjective and often don't mirror real-world usage,
this library's authors aren't making any specific performance claims. The way
Pot achieves space savings requires some computational overhead. As such, it is
expected that a hypothetically perfect CBOR implementation could outperform a
hypothetically perfect Pot implementation.

The results from the current benchmark suite executed on Github Actions are
[viewable here](https://pot.bonsaidb.io/benchmarks/report/). The current suite
is only aimed at comparing the default performance for each library.

### Serialize into new `Vec<u8>`

[![Serialize Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_serialize/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_serialize/report/index.html)

### Serialize into reused `Vec<u8>`

[![Serialize with Reused Buffer Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_serialize-reuse/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_serialize-reuse/report/index.html)

### Deserialize

[![Deserialize Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_deserialize/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_deserialize/report/index.html)

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), are
open-source. This repository is available under the [MIT License](./LICENSE-MIT)
or the [Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
