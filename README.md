# Pot: The storage and network serialization format for BonsaiDb

PBOR is an encoding format [being proposed](https://community.khonsulabs.com/t/towards-stabilization-serialization-format-s-for-pliantdb/71) for use within `PliantDb`. Its purpose is to provide an encoding format for `serde` that:

* Preserves Identifiers: `bincode` is a great format, but it's so compact because it discards identifiers. This is great if you're willing to accept the limitations on how to maintain version compatibility. However, `JSON` became popular because it preserved enough information that it can be flexibly serialized to and from with cross-version compatibility. Serde offers many features such as renames and aliases that help with this, but bincode can only support so many of these features due to the lack of identifiers in the encoded output.
* Is safe to run in production: As far as I can tell, none of the available CBOR crates offer settings to ensure memory exhustion attacks can be prevented.
* Is Compact: CBOR is compact, but imagine this setup:

  ```rust
  struct ShoppingCart {
      items: Vec<Item>,
  }

  struct Item {
      product_id: u64,
      quantity: u16,
      gift_message: String,
      // ...
  }
  ```

  JSON/CBOR (amongst many other formats that preserve identifiers) include the name of the identifier each time its used. Thus, a shopping cart that had 50 items would have the identifier `product_id` included 50 times. PBOR utilizes an identifier table to ensure that each payload preserves identifiers, but only includes each identifier once.

This blend of features should make it nearly as compact as bincode in many situations, but much more flexible allowing for easier cross-version compatibility: a must for `PliantDb`.

## Status of Project

This experiment [has been a success](https://community.khonsulabs.com/t/towards-stabilization-serialization-format-s-for-pliantdb/71#how-did-the-experiment-go-5), but it's not such a large victory that the benefits of using it over sticking with an open standard aren't clear.

This project has minimal testing at this current stage, and shouldn't be used in
production. It has horrible error reporting for when things go wrong, and it
does not specifically handle maliciously designed payloads yet. If this project is adopted into PliantDb, these aspects will be addressed.
