A concise storage format, written for [`BonsaiDb`](https://bonsaidb.io/).

![Pot forbids unsafe code](https://img.shields.io/badge/unsafe-forbid-success)
[![crate version](https://img.shields.io/crates/v/pot.svg)](https://crates.io/crates/pot)
[![Live Build Status](https://img.shields.io/github/actions/workflow/status/khonsulabs/pot/rust.yml?branch=main)](https://github.com/khonsulabs/pot/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/pot/coverage/badge.svg)](https://khonsulabs.github.io/pot/coverage/)
[![Documentation for `main` branch](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/pot/main/pot/)

Pot is an encoding format used within [`BonsaiDb`](https://bonsaidb.io/). Its purpose is to
provide an encoding format for [`serde`](https://serde.rs) that:

* Is self-describing.
* Is safe to run in production.
* Is compact. While still being self-describing, Pot's main space-saving feature
  is not repeating symbols/identifiers more than one time while serializing.
  When serializing arrays of structures, this can make a major difference. The
  [logs.rs](https://github.com/khonsulabs/pot/blob/main/benchmarks/examples/logs.rs)
  example demonstrates this:

  ```sh
  $$ cargo test --example logs -- average_sizes --nocapture
  Generating 1000 LogArchives with 100 entries.
  +-----------------+-----------+-----------------+
  | Format          | Bytes     | Self-Describing |
  +-----------------+-----------+-----------------+
  | pot             | 2,627,586 | yes             |
  +-----------------+-----------+-----------------+
  | cbor            | 3,072,369 | yes             |
  +-----------------+-----------+-----------------+
  | msgpack(named)  | 3,059,915 | yes             |
  +-----------------+-----------+-----------------+
  | msgpack         | 2,559,907 | no              |
  +-----------------+-----------+-----------------+
  | bincode(varint) | 2,506,844 | no              |
  +-----------------+-----------+-----------------+
  | bincode         | 2,755,137 | no              |
  +-----------------+-----------+-----------------+
  ```

## Example

```rust
$../pot/examples/simple.rs:example$
```

Outputs:

```text
User serialized: [50, 6f, 74, 00, a2, c4, 69, 64, 40, 2a, c8, 6e, 61, 6d, 65, e5, 65, 63, 74, 6f, 6e]
User decoded as value: {id: 42, name: ecton}
```

## Benchmarks

Because benchmarks can be subjective and often don't mirror real-world usage,
this library's authors aren't making any specific performance claims. The way
Pot achieves space savings requires some computational overhead. As such, it is
expected that a hypothetically perfect CBOR implementation could outperform a
hypothetically perfect Pot implementation.

The results from the current benchmark suite executed on GitHub Actions are
[viewable here](https://pot.bonsaidb.io/benchmarks/report/). The current suite
is only aimed at comparing the default performance for each library.

### Serialize into new `Vec<u8>`

[![Serialize Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_serialize/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_serialize/report/index.html)

### Serialize into reused `Vec<u8>`

[![Serialize with Reused Buffer Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_serialize-reuse/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_serialize-reuse/report/index.html)

### Deserialize

[![Deserialize Benchmark Violin Chart](https://pot.bonsaidb.io/benchmarks/logs_deserialize/report/violin.svg)](https://pot.bonsaidb.io/benchmarks/logs_deserialize/report/index.html)
