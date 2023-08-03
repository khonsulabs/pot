// begin rustme snippet: example
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Default)]
pub struct User {
    id: u64,
    name: String,
}

fn main() {
    // Pot's main space saving feature is being able to reuse previously encoded
    // fields. Pot also supports persisting "symbol maps" in many powerful ways.
    // This example shows how to compute a symbol map that can be pre-shared to
    // keep payloads smaller.
    let mut preshared_map = pot::ser::SymbolMap::new();
    // Load the symbols from an instance of `User`.
    preshared_map.populate_from(&User::default()).unwrap();
    println!("Preshared symbols: {preshared_map:?}");

    let original_user = User {
        id: 42,
        name: String::from("ecton"),
    };
    let encoded_without_map = pot::to_vec(&original_user).unwrap();
    let mut encoded_with_map = Vec::new();
    original_user
        .serialize(&mut preshared_map.serializer_for(&mut encoded_with_map).unwrap())
        .unwrap();
    println!(
        "Default User encoded without map: {} bytes",
        encoded_without_map.len()
    );
    println!(
        "Default User encoded with map:    {} bytes",
        encoded_with_map.len()
    );
    assert!(encoded_with_map.len() < encoded_without_map.len());

    // Serialize the map and "send" it to the receiver.
    let preshared_map_bytes = pot::to_vec(&preshared_map).unwrap();

    // Deserialize the symbol map.
    let mut deserializer_map: pot::de::SymbolMap = pot::from_slice(&preshared_map_bytes).unwrap();
    // Deserialize the payload using the map.
    let user = User::deserialize(
        &mut deserializer_map
            .deserializer_for_slice(&encoded_with_map)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(user, original_user);
}

#[test]
fn runs() {
    main().unwrap();
}
