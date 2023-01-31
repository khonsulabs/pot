// begin rustme snippet: example
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct User {
    id: u64,
    name: String,
}

fn main() -> Result<(), pot::Error> {
    let user = User {
        id: 42,
        name: String::from("ecton"),
    };
    let serialized = pot::to_vec(&user)?;
    println!("User serialized: {serialized:02x?}");
    let deserialized: User = pot::from_slice(&serialized)?;
    assert_eq!(deserialized, user);

    // Pot also provides a "Value" type for serializing Pot encoded payloads
    // without needing the original structure.
    let user: pot::Value<'_> = pot::from_slice(&serialized)?;
    println!("User decoded as value: {user}");

    Ok(())
}
// end rustme snippet

#[test]
fn runs() {
    main().unwrap();
}
