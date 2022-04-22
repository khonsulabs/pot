//! Do not take these benchmarks seriously at the moment.
//!
//! Proper benchmarks will be coming.
use std::fmt::Display;

use chrono::{DateTime, Utc};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fake::{
    faker::{filesystem::en::FilePath, internet::en::Username, lorem::en::Sentence},
    Fake,
};
use rand::{thread_rng, Rng};
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
    Pbor,
    Cbor,
    Bincode,
    Msgpack,
}

impl Backend {
    fn all() -> [Self; 4] {
        [Self::Pbor, Self::Cbor, Self::Bincode, Self::Msgpack]
    }
}

impl Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Pbor => "pot",
            Self::Cbor => "cbor",
            Self::Bincode => "bincode",
            Self::Msgpack => "msgpack",
        })
    }
}

const LOG_ENTRIES: usize = 10_000;

fn bench_logs(c: &mut Criterion) {
    let mut logs = LogArchive {
        entries: Vec::with_capacity(LOG_ENTRIES),
    };
    for _ in 0..LOG_ENTRIES {
        logs.entries.push(Log::generate(&mut thread_rng()));
    }

    let mut serialize_group = c.benchmark_group("logs/serialize");
    for backend in Backend::all() {
        let serialize = match backend {
            Backend::Pbor => |logs| pot::to_vec(logs).unwrap(),
            Backend::Cbor => |logs| {
                let mut cbor_bytes = Vec::new();
                ciborium::ser::into_writer(&logs, &mut cbor_bytes).unwrap();
                cbor_bytes
            },
            Backend::Bincode => |logs| bincode::serialize(logs).unwrap(),
            Backend::Msgpack => |logs| rmp_serde::to_vec(logs).unwrap(),
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
            Backend::Pbor => pbor_serialize_into,
            Backend::Cbor => cbor_serialize_into,
            Backend::Bincode => bincode_serialize_into,
            Backend::Msgpack => msgpack_serialize_into,
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
            Backend::Pbor => |logs| pot::from_slice::<LogArchive>(logs).unwrap(),
            Backend::Cbor => |logs| ciborium::de::from_reader(logs).unwrap(),
            Backend::Bincode => |logs| bincode::deserialize(logs).unwrap(),
            Backend::Msgpack => |logs| rmp_serde::from_slice(logs).unwrap(),
        };
        let bytes = match backend {
            Backend::Pbor => pot::to_vec(&logs).unwrap(),
            Backend::Cbor => {
                let mut cbor_bytes = Vec::new();
                ciborium::ser::into_writer(&logs, &mut cbor_bytes).unwrap();
                cbor_bytes
            }
            Backend::Bincode => bincode::serialize(&logs).unwrap(),
            Backend::Msgpack => rmp_serde::to_vec(&logs).unwrap(),
        };
        deserialize_group.bench_function(backend.to_string(), |b| {
            b.iter(|| {
                deserialize(black_box(&bytes));
            });
        });
    }
}

fn pbor_serialize_into(logs: &LogArchive, buffer: &mut Vec<u8>) {
    logs.serialize(&mut pot::ser::Serializer::new(buffer).unwrap())
        .unwrap();
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

fn criterion_benchmark(c: &mut Criterion) {
    bench_logs(c)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
