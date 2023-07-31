//! Do not take these benchmarks seriously at the moment.
//!
//! Proper benchmarks will be coming.

use std::env;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::faker::filesystem::en::FilePath;
use fake::faker::internet::en::Username;
use fake::faker::lorem::en::Sentence;
use fake::Fake;
use rand::rngs::StdRng;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Log {
    pub level: Level,
    pub user_id: String,
    pub timestamp: DateTime<Utc>,
    pub request: String,
    pub message: Option<String>,
    pub code: u16,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LogArchive {
    entries: Vec<Log>,
}

impl Log {
    fn generate<R: Rng>(rand: &mut R) -> Self {
        Self {
            user_id: Username().fake_with_rng(rand),
            timestamp: Utc::now(),
            code: rand.gen(),
            size: rand.gen(),
            level: Level::generate(rand),
            request: FilePath().fake_with_rng(rand),
            message: if rand.gen() {
                Some(Sentence(3..100).fake_with_rng(rand))
            } else {
                None
            },
        }
    }
}

impl Level {
    fn generate<R: Rng>(rand: &mut R) -> Self {
        match rand.gen_range(0_u8..=4u8) {
            0 => Level::Trace,
            1 => Level::Debug,
            2 => Level::Info,
            3 => Level::Warn,
            4 => Level::Error,
            _ => unreachable!(),
        }
    }
}

enum Backend {
    Pot,
    Cbor,
    Bincode,
    Msgpack,
    MsgpackNamed,
}

impl Backend {
    fn all() -> [Self; 5] {
        [
            Self::Pot,
            Self::Cbor,
            Self::Bincode,
            Self::Msgpack,
            Self::MsgpackNamed,
        ]
    }
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pot => "pot",
            Self::Cbor => "cbor",
            Self::Bincode => "bincode",
            Self::Msgpack => "MessagePack",
            Self::MsgpackNamed => "MessagePack(named)",
        })
    }
}

const LOG_ENTRIES: usize = 10_000;

fn bench_logs(c: &mut Criterion) {
    let mut logs = LogArchive {
        entries: Vec::with_capacity(LOG_ENTRIES),
    };
    let random_seed = env::args().find(|arg| arg.starts_with("-s")).map_or_else(
        || thread_rng().gen(),
        |seed| {
            let (_, seed) = seed.split_at(2);
            let (upper, lower) = if seed.len() > 32 {
                let (upper, lower) = seed.split_at(seed.len() - 32);
                (
                    u128::from_str_radix(upper, 16).expect("invalid hexadecimal seed"),
                    u128::from_str_radix(lower, 16).expect("invalid hexadecimal seed"),
                )
            } else {
                (
                    0,
                    u128::from_str_radix(seed, 16).expect("invalid hexadecimal seed"),
                )
            };
            let mut seed = [0; 32];
            seed[..16].copy_from_slice(&upper.to_be_bytes());
            seed[16..].copy_from_slice(&lower.to_be_bytes());
            seed
        },
    );
    print!("Using random seed -s");
    for b in random_seed {
        print!("{b:02x}");
    }
    println!();
    let mut rng = StdRng::from_seed(random_seed);
    for _ in 0..LOG_ENTRIES {
        logs.entries.push(Log::generate(&mut rng));
    }

    let mut serialize_group = c.benchmark_group("logs/serialize");
    for backend in Backend::all() {
        let serialize = match backend {
            Backend::Pot => |logs| pot::to_vec(logs).unwrap(),
            Backend::Cbor => |logs| {
                let mut cbor_bytes = Vec::new();
                ciborium::ser::into_writer(&logs, &mut cbor_bytes).unwrap();
                cbor_bytes
            },
            Backend::Bincode => |logs| bincode::serialize(logs).unwrap(),
            Backend::Msgpack => |logs| rmp_serde::to_vec(logs).unwrap(),
            Backend::MsgpackNamed => |logs| rmp_serde::to_vec_named(logs).unwrap(),
        };
        serialize_group.bench_function(backend.to_string(), |b| {
            b.iter(|| {
                serialize(black_box(&logs));
            });
        });
    }
    drop(serialize_group);

    let mut buffer = Vec::with_capacity(LOG_ENTRIES * 1024);
    let mut serialize_reuse_group = c.benchmark_group("logs/serialize-reuse");
    for backend in Backend::all() {
        let serialize = match backend {
            Backend::Pot => pot_serialize_into,
            Backend::Cbor => cbor_serialize_into,
            Backend::Bincode => bincode_serialize_into,
            Backend::Msgpack => msgpack_serialize_into,
            Backend::MsgpackNamed => msgpack_serialize_into_named,
        };

        serialize_reuse_group.bench_function(backend.to_string(), |b| {
            b.iter(|| {
                buffer.clear();
                serialize(black_box(&logs), black_box(&mut buffer));
            });
        });
    }
    drop(serialize_reuse_group);

    let mut deserialize_group = c.benchmark_group("logs/deserialize");

    for backend in Backend::all() {
        let deserialize = match backend {
            Backend::Pot => |logs| pot::from_slice::<LogArchive>(logs).unwrap(),
            Backend::Cbor => |logs| ciborium::de::from_reader(logs).unwrap(),
            Backend::Bincode => |logs| bincode::deserialize(logs).unwrap(),
            Backend::Msgpack | Backend::MsgpackNamed => |logs| rmp_serde::from_slice(logs).unwrap(),
        };
        let bytes = match backend {
            Backend::Pot => pot::to_vec(&logs).unwrap(),
            Backend::Cbor => {
                let mut cbor_bytes = Vec::new();
                ciborium::ser::into_writer(&logs, &mut cbor_bytes).unwrap();
                cbor_bytes
            }
            Backend::Bincode => bincode::serialize(&logs).unwrap(),
            Backend::Msgpack => rmp_serde::to_vec(&logs).unwrap(),
            Backend::MsgpackNamed => rmp_serde::to_vec_named(&logs).unwrap(),
        };
        deserialize_group.bench_function(backend.to_string(), |b| {
            b.iter(|| {
                deserialize(black_box(&bytes));
            });
        });
    }
}

fn pot_serialize_into(logs: &LogArchive, buffer: &mut Vec<u8>) {
    pot::to_writer(logs, buffer).unwrap();
}

fn cbor_serialize_into(logs: &LogArchive, buffer: &mut Vec<u8>) {
    ciborium::ser::into_writer(logs, buffer).unwrap();
}

fn bincode_serialize_into(logs: &LogArchive, buffer: &mut Vec<u8>) {
    bincode::serialize_into(buffer, logs).unwrap();
}

fn msgpack_serialize_into(logs: &LogArchive, buffer: &mut Vec<u8>) {
    rmp_serde::encode::write(buffer, logs).unwrap()
}

fn msgpack_serialize_into_named(logs: &LogArchive, buffer: &mut Vec<u8>) {
    rmp_serde::encode::write_named(buffer, logs).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_logs(c)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
