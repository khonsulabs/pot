use bincode::Options;
use chrono::{DateTime, Utc};
use cli_table::{Cell, Table};
use fake::{
    faker::{filesystem::en::FilePath, internet::en::Username, lorem::en::Sentence},
    Fake,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use thousands::Separable;

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

fn main() -> anyhow::Result<()> {
    // Generate a bunch of logs
    let mut logs = LogArchive { entries: vec![] };
    for _ in 0..10_000 {
        logs.entries.push(Log::generate(&mut thread_rng()));
    }

    // Encode without a persistent session
    let pbor_bytes = pot::to_vec(&logs)?;
    let bincode_bytes = bincode::serialize(&logs)?;
    let bincode_varint_bytes = bincode::DefaultOptions::default()
        .with_varint_encoding()
        .serialize(&logs)?;
    let cbor_bytes = serde_cbor::to_vec(&logs)?;

    cli_table::print_stdout(
        vec![
            vec!["pot".cell(), pbor_bytes.len().separate_with_commas().cell()],
            vec![
                "cbor".cell(),
                cbor_bytes.len().separate_with_commas().cell(),
            ],
            vec![
                "bincode(varint)".cell(),
                bincode_varint_bytes.len().separate_with_commas().cell(),
            ],
            vec![
                "bincode".cell(),
                bincode_bytes.len().separate_with_commas().cell(),
            ],
        ]
        .table()
        .title(vec!["Format", "Bytes"]),
    )?;

    // With Pot, you can also use a persistent encoding session to save more
    // bandwidth, as long as you guarantee payloads are serialized and
    // deserialized in a consistent order.
    //
    // In this situation, the payloads across a network are generally smaller,
    // so let's show the benefits by just encoding a single log entry.
    let mut sender_state = pot::ser::SymbolMap::default();
    let mut receiver_state = pot::de::SymbolMap::new();
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
        "Using a persistent encoding session, the first payload was {} bytes long.",
        first_transmission_length
    );
    println!(
        "The same payload sent a second time was {} bytes long.",
        subsequent_transmission_length
    );

    Ok(())
}

#[test]
fn one_log() {
    let log = Log::generate(&mut thread_rng());
    let bytes = pot::to_vec(&log).unwrap();
    let result = pot::from_slice(&bytes).unwrap();
    assert_eq!(log, result);
}
