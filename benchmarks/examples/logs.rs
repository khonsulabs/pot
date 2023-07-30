use bincode::Options;
use chrono::{DateTime, Utc};
use cli_table::{Cell, Table};
use fake::faker::filesystem::en::FilePath;
use fake::faker::internet::en::Username;
use fake::faker::lorem::en::Sentence;
use fake::Fake;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use thousands::Separable;

fn main() -> anyhow::Result<()> {
    // Generate a bunch of logs
    let logs = LogArchive::generate(&mut thread_rng(), 10_000);

    // Encode without a persistent session
    let pot_bytes = pot::to_vec(&logs)?;
    let bincode_bytes = bincode::serialize(&logs)?;
    let bincode_varint_bytes = bincode::DefaultOptions::default()
        .with_varint_encoding()
        .serialize(&logs)?;
    let mut cbor_bytes = Vec::new();
    ciborium::ser::into_writer(&logs, &mut cbor_bytes)?;
    let msgpack_bytes = rmp_serde::to_vec_named(&logs)?;
    let msgpack_compact_bytes = rmp_serde::to_vec(&logs)?;

    cli_table::print_stdout(
        vec![
            vec![
                "pot".cell(),
                pot_bytes.len().separate_with_commas().cell(),
                "yes".cell(),
            ],
            vec![
                "cbor".cell(),
                cbor_bytes.len().separate_with_commas().cell(),
                "yes".cell(),
            ],
            vec![
                "msgpack(named)".cell(),
                msgpack_bytes.len().separate_with_commas().cell(),
                "yes".cell(),
            ],
            vec![
                "msgpack".cell(),
                msgpack_compact_bytes.len().separate_with_commas().cell(),
                "no".cell(),
            ],
            vec![
                "bincode(varint)".cell(),
                bincode_varint_bytes.len().separate_with_commas().cell(),
                "no".cell(),
            ],
            vec![
                "bincode".cell(),
                bincode_bytes.len().separate_with_commas().cell(),
                "no".cell(),
            ],
        ]
        .table()
        .title(vec!["Format", "Bytes", "Self-Describing"]),
    )?;

    // With Pot, you can also use a persistent encoding session to save more
    // bandwidth, as long as you guarantee payloads are serialized and
    // deserialized in a consistent order.
    //
    // In this situation, the payloads across a network are generally smaller,
    // so let's show the benefits by just encoding a single log entry.
    let mut sender_state = pot::ser::SymbolMap::default();
    let mut receiver_state = pot::de::SymbolMap::default();
    let mut payload_buffer = Vec::new();
    logs.entries[0].serialize(&mut sender_state.serializer_for(&mut payload_buffer)?)?;
    let first_transmission_length = payload_buffer.len();
    {
        assert_eq!(
            &Log::deserialize(&mut receiver_state.deserializer_for_slice(&payload_buffer)?)?,
            &logs.entries[0]
        );
    }
    let mut payload_buffer = Vec::new();
    logs.entries[0].serialize(&mut sender_state.serializer_for(&mut payload_buffer)?)?;
    let subsequent_transmission_length = payload_buffer.len();
    assert_eq!(
        &Log::deserialize(&mut receiver_state.deserializer_for_slice(&payload_buffer)?)?,
        &logs.entries[0]
    );

    println!(
        "Using a persistent encoding session, the first payload was {first_transmission_length} bytes long.",

    );
    println!(
        "The same payload sent a second time was {subsequent_transmission_length} bytes long.",
    );

    Ok(())
}

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

impl LogArchive {
    fn generate<R: Rng>(rand: &mut R, count: usize) -> Self {
        let mut entries = Vec::new();
        for _ in 0..count {
            entries.push(Log::generate(rand));
        }
        Self { entries }
    }
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

#[test]
fn runs() {
    main().unwrap();
}

#[test]
fn one_log() {
    let log = Log::generate(&mut thread_rng());
    let bytes = pot::to_vec(&log).unwrap();
    let result = pot::from_slice(&bytes).unwrap();
    assert_eq!(log, result);
}

#[test]
fn average_sizes() {
    let mut bincode_sizes = Vec::new();
    let mut bincode_varint_sizes = Vec::new();
    let mut cbor_sizes = Vec::new();
    let mut pot_sizes = Vec::new();
    let mut msgpack_sizes = Vec::new();
    let mut msgpack_compact_sizes = Vec::new();

    const ITERATIONS: usize = 1_000;
    println!("Generating {} LogArchives with 100 entries.", ITERATIONS);
    for _ in 0..ITERATIONS {
        let log = LogArchive::generate(&mut thread_rng(), 100);
        bincode_sizes.push(bincode::serialize(&log).unwrap().len());
        bincode_varint_sizes.push(
            bincode::DefaultOptions::default()
                .with_varint_encoding()
                .serialize(&log)
                .unwrap()
                .len(),
        );
        let mut cbor_bytes = Vec::new();
        ciborium::ser::into_writer(&log, &mut cbor_bytes).unwrap();
        cbor_sizes.push(cbor_bytes.len());
        pot_sizes.push(pot::to_vec(&log).unwrap().len());
        msgpack_sizes.push(rmp_serde::to_vec_named(&log).unwrap().len());
        msgpack_compact_sizes.push(rmp_serde::to_vec(&log).unwrap().len());
    }

    let bincode_average = bincode_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;
    let bincode_varint_average =
        bincode_varint_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;
    let cbor_average = cbor_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;
    let pot_average = pot_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;
    let msgpack_average = msgpack_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;
    let msgpack_compact_average =
        msgpack_compact_sizes.iter().copied().sum::<usize>() as f64 / ITERATIONS as f64;

    cli_table::print_stdout(
        vec![
            vec!["pot".cell(), pot_average.separate_with_commas().cell()],
            vec![
                "bincode(varint)".cell(),
                bincode_varint_average.separate_with_commas().cell(),
            ],
            vec![
                "bincode".cell(),
                bincode_average.separate_with_commas().cell(),
            ],
            vec!["cbor".cell(), cbor_average.separate_with_commas().cell()],
            vec![
                "msgpack".cell(),
                msgpack_average.separate_with_commas().cell(),
            ],
            vec![
                "msgpack(compact)".cell(),
                msgpack_compact_average.separate_with_commas().cell(),
            ],
        ]
        .table()
        .title(vec!["Format", "Avg. Bytes"]),
    )
    .unwrap();
}
