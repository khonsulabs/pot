var searchIndex = JSON.parse('{\
"pot":{"doc":"A concise storage format, written for <code>BonsaiDb</code>.","t":[13,13,3,13,4,13,13,13,13,13,13,13,13,13,13,13,13,6,13,13,13,13,13,13,13,13,4,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,0,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,11,5,11,11,11,11,11,11,11,0,0,11,11,11,11,11,11,11,11,5,5,11,11,11,11,11,11,11,11,11,11,12,12,12,12,12,12,12,12,12,12,12,12,12,12,13,3,13,13,4,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,3,13,13,13,13,13,13,13,13,3,13,13,13,3,13,4,13,13,13,13,4,13,4,13,13,13,13,13,13,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,5,5,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,12,12,12,12,3,8,3,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Bool","Bytes","Config","Eof","Error","Float","ImpreciseCastWouldLoseData","IncompatibleVersion","Integer","InvalidAtomHeader","InvalidKind","InvalidUtf8","Io","Mappings","Message","None","NotAPot","Result","Sequence","SequenceSizeMustBeKnown","String","TooManyBytesRead","TrailingBytes","UnexpectedKind","Unit","UnknownSymbol","Value","allocation_budget","as_bool","as_bytes","as_float","as_integer","as_str","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","custom","custom","de","default","deserialize","deserialize","deserialize_from","eq","fmt","fmt","fmt","fmt","fmt","format","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_mappings","from_reader","from_sequence","from_slice","into","into","into","into_static","is_empty","mappings","provide","reader","ser","serialize","serialize","serialize_into","source","to_owned","to_owned","to_string","to_string","to_vec","to_writer","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id","values","0","0","0","0","0","0","1","0","0","0","0","0","0","0","Borrowed","Deserializer","Owned","Persistent","SymbolMap","borrow","borrow","borrow_mut","borrow_mut","deserialize_any","deserialize_bool","deserialize_byte_buf","deserialize_bytes","deserialize_char","deserialize_enum","deserialize_f32","deserialize_f64","deserialize_i128","deserialize_i16","deserialize_i32","deserialize_i64","deserialize_i8","deserialize_identifier","deserialize_ignored_any","deserialize_map","deserialize_newtype_struct","deserialize_option","deserialize_seq","deserialize_str","deserialize_string","deserialize_struct","deserialize_tuple","deserialize_tuple_struct","deserialize_u128","deserialize_u16","deserialize_u32","deserialize_u64","deserialize_u8","deserialize_unit","deserialize_unit_struct","deserializer_for_slice","end_of_input","fmt","fmt","from","from","into","into","is_human_readable","new","newtype_variant_seed","struct_variant","try_from","try_from","try_into","try_into","tuple_variant","type_id","type_id","unit_variant","variant_seed","0","0","0","Atom","Boolean","Bytes","Bytes","DynamicEnd","DynamicEnd","DynamicMap","DynamicMap","False","Float","Float","Float","Int","Integer","Integer","Kind","Map","Named","Named","None","Nucleus","Sequence","Special","Special","Symbol","True","UInt","Unit","Unit","arg","as_f32","as_f32","as_f64","as_f64","as_float","as_i128","as_i16","as_i32","as_i64","as_i8","as_integer","as_u128","as_u16","as_u32","as_u64","as_u8","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clone","clone","clone","clone_into","clone_into","clone_into","deserialize","deserialize","eq","eq","eq","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from","from_u8","into","into","into","into","into","into","is_zero","is_zero","kind","nucleus","read_atom","read_atom_header","read_from","read_from","read_header","serialize","serialize","to_owned","to_owned","to_owned","to_string","to_string","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","write_atom_header","write_bool","write_bytes","write_f32","write_f64","write_header","write_i128","write_i16","write_i24","write_i32","write_i48","write_i64","write_i8","write_named","write_none","write_special","write_str","write_u128","write_u16","write_u24","write_u32","write_u48","write_u64","write_u8","write_unit","0","0","0","0","IoReader","Reader","SliceReader","borrow","borrow","borrow_mut","borrow_mut","buffered_read_bytes","buffered_read_bytes","buffered_read_bytes","fmt","from","from","from","into","into","read","read","read_exact","read_exact","read_to_end","read_to_string","read_vectored","try_from","try_from","try_into","try_into","type_id","type_id","MapSerializer","Serializer","SymbolMap","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","default","end","end","end","end","end","end","end","fmt","fmt","from","from","from","into","into","into","is_human_readable","new","serialize_bool","serialize_bytes","serialize_char","serialize_element","serialize_element","serialize_f32","serialize_f64","serialize_field","serialize_field","serialize_field","serialize_field","serialize_i128","serialize_i16","serialize_i32","serialize_i64","serialize_i8","serialize_key","serialize_map","serialize_newtype_struct","serialize_newtype_variant","serialize_none","serialize_seq","serialize_some","serialize_str","serialize_struct","serialize_struct_variant","serialize_tuple","serialize_tuple_struct","serialize_tuple_variant","serialize_u128","serialize_u16","serialize_u32","serialize_u64","serialize_u8","serialize_unit","serialize_unit_struct","serialize_unit_variant","serialize_value","serializer_for","try_from","try_from","try_from","try_into","try_into","try_into","type_id","type_id","type_id"],"q":["pot","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::Error","","","","","","","pot::Value","","","","","","","pot::de","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::de::SymbolMap","","","pot::format","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::format::Nucleus","","","","pot::reader","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::ser","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"d":["A boolean value","A value containing arbitrary bytes.","Serialization and deserialization configuration.","Expected more data but encountered the end of the input.","All errors that Pot may return.","A floating point value.","A numerical value could not be handled without losing …","Data was written with an incompatible version.","An integer value.","An atom header was incorrectly formatted.","An unknown kind was encountered. Generally a sign that …","String data contained invalid utf-8 characters.","An error occurred from io.","A sequence of key-value mappings.","A generic error occurred.","A value representing None.","Payload is not a Pot payload.","A result alias that returns <code>Error</code>.","A sequence of values.","A sequence of unknown size cannot be serialized.","A string value.","The amount of data read exceeds the configured maximum …","Extra data appeared at the end of the input.","Encountered an unexpected atom kind.","A value representing a Unit (<code>()</code>).","A requested symbol id was not found.","A Pot encoded value. This type can be used to deserialize …","Sets the maximum number of bytes able to be allocated. …","Returns the value represented as a value.","Returns the value’s bytes, or None if the value is not …","Returns the value as an <code>Float</code>. Returns None if the value …","Returns the value as an <code>Integer</code>. Returns None if the value …","Returns the value as a string, or None if the value is not …","","","","","","","","","","","","","Types for deserializing pots.","","","Deserializes a value from a slice using the configured …","Deserializes a value from a <code>Read</code> implementor using the …","","","","","","","Low-level interface for reading and writing the pot format.","","Returns the argument unchanged.","","","","","","","","","","","Returns the argument unchanged.","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns a new value from an interator of 2-element tuples …","Restore a previously Pot-serialized value from a <code>Read</code> …","Returns a new value from an interator of items that can be …","Restore a previously Pot-serialized value from a slice.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Converts <code>self</code> to a static lifetime by cloning any borrowed …","Returns true if the value contained is considered empty.","Returns an interator that iterates over all mappings …","","Types for reading data.","Types for serializing pots.","","Serializes a value to a <code>Vec</code> using the configured options.","Serializes a value to a writer using the configured …","","","","","","Serialize <code>value</code> using Pot into a <code>Vec&lt;u8&gt;</code>.","Serialize <code>value</code> using Pot into <code>writer</code>.","","","","","","","","","","Returns an interator that iterates over all values …","","","","","","","","","","","","","","","A list of borrowed symbols.","Deserializer for the Pot format.","An owned list of symbols.","A mutable reference to an owned list of symbols.","A collection of deserialized symbols.","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns a deserializer for <code>slice</code>.","Returns true if the input has been consumed completely.","","","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","Returns a new symbol map that will persist symbols between …","","","","","","","","","","","","","","","An encoded <code>Kind</code>, argument, and optional contained value.","A boolean value.","A series of bytes. The argument is the length. The bytes …","A buffer of bytes.","A terminal value for a <code>Self::DynamicMap</code>.","A marker denoting the end of a map with unknown length.","A sequence of key-value pairs with an unknown length.","A marker denoting a map with unknown length is next in the …","The <code>false</code> boolean literal.","A floating point number that can safely convert between …","A floating point value. Argument is the byte length, minus …","A floating point value.","A signed integer. Argument is the byte length, minus one. …","An integer type that can safely convert between other …","An integer value.","The type of an atom.","A list of key-value pairs. Argument is the count of …","A named value. A symbol followed by another value.","A named value.","A None value.","A value contained within an <code>Atom</code>.","A list of atoms. Argument is the count of atoms in the …","A special value type.","A value with a special meaning.","A symbol. If the least-significant bit of the arg is 0, …","The <code>true</code> boolean literal.","An unsigned integer. Argument is the byte length, minus …","A Unit value.","A unit.","The argument contained in the atom header.","Converts this integer to an f32, but only if it can be …","Returns this number as an f32, if it can be done without …","Converts this integer to an f64, but only if it can be …","Returns this number as an f64.","Converts this integer to an f64, but only if it can be …","Returns the contained value as an i64, or an error if the …","Returns the contained value as an i16, or an error if the …","Returns the contained value as an i32, or an error if the …","Returns the contained value as an i64, or an error if the …","Returns the contained value as an i8, or an error if the …","Returns this number as an <code>Integer</code>, if the stored value has …","Returns the contained value as an u64, or an error if the …","Returns the contained value as an u16, or an error if the …","Returns the contained value as an u32, or an error if the …","Returns the contained value as an u64, or an error if the …","Returns the contained value as an u8, or an error if the …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","","","","","","","","","Returns the argument unchanged.","","","Returns the argument unchanged.","Returns the argument unchanged.","","","Returns the argument unchanged.","Converts from a u8. Returns an error if <code>kind</code> is an invalid …","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Returns true if the value contained is zero.","Returns true if the value contained is zero.","The type of atom.","The contained value, if any.","Reads an atom.","Reads an atom header (kind and argument).","Reads an integer based on the atom header (<code>kind</code> and …","Reads a floating point number given the atom <code>kind</code> and …","Reads a Pot header. See <code>write_header</code> for more information. …","","","","","","","","","","","","","","","","","","","","","","","","","","","Writes an atom header into <code>writer</code>.","Writes a <code>Kind::Special</code> atom with either <code>Special::True</code> or …","Writes an <code>Kind::Bytes</code> atom with the given value.","Writes an <code>Kind::Float</code> atom with the given value.","Writes an <code>Kind::Float</code> atom with the given value.","Writes the Pot header. A u32 written in big endian. The …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes a <code>Kind::Special</code> atom with <code>Special::Named</code>.","Writes a <code>Kind::Special</code> atom with <code>Special::None</code>.","Writes a <code>Kind::Special</code> atom.","Writes an <code>Kind::Bytes</code> atom with the bytes of the string.","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::UInt</code> atom with the given value.","Writes a <code>Kind::Special</code> atom with <code>Special::Unit</code>.","","","","","A reader over <code>ReadBytesExt</code>.","A reader that can temporarily buffer bytes read.","Reads data from a slice.","","","","","Read exactly <code>length</code> bytes and return a reference to the …","","","","Returns the argument unchanged.","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","","","","","","","","","Serializes map-like values.","A Pot serializer.","A list of previously serialized symbols.","","","","","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","Returns a new serializer outputting written bytes into …","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Returns a serializer that writes into <code>output</code> that persists …","","","","","","","","",""],"i":[3,3,0,10,0,3,10,10,3,10,10,10,10,3,10,3,10,0,3,10,3,10,10,10,3,10,0,1,3,3,3,3,3,10,3,1,10,3,1,3,1,3,1,10,10,0,1,3,1,1,3,10,10,3,3,1,0,10,10,10,10,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,1,3,0,3,0,10,3,1,3,3,3,10,0,0,3,1,1,10,3,1,10,3,0,0,10,3,1,10,3,1,10,3,1,3,55,56,57,58,59,60,59,61,62,63,64,65,66,67,39,0,39,39,0,38,39,38,39,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,38,39,38,38,39,38,39,38,39,38,39,38,38,38,39,38,39,38,38,39,38,38,68,69,70,0,44,42,44,45,44,45,44,45,0,42,44,42,0,44,0,42,45,44,45,0,42,0,42,42,45,42,45,44,43,7,6,7,6,7,7,7,7,7,7,6,7,7,7,7,7,45,42,7,43,6,44,45,42,7,43,6,44,42,7,6,42,7,6,7,6,42,7,6,42,7,7,43,6,6,44,45,42,7,7,7,7,7,7,7,7,7,7,7,43,6,6,6,44,42,45,42,7,43,6,44,7,6,43,43,0,0,7,6,0,7,6,42,7,6,7,6,45,45,42,7,43,6,44,45,42,7,43,6,44,45,42,7,43,6,44,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,71,72,73,74,0,0,0,40,49,40,49,41,40,49,40,40,40,49,40,49,40,49,40,49,49,49,49,40,49,40,49,40,49,0,0,0,52,53,50,52,53,50,50,52,52,52,53,53,53,53,53,50,52,53,50,52,53,50,53,53,53,53,53,53,53,53,53,52,52,53,53,53,53,53,53,53,52,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,53,52,50,52,53,50,52,53,50,52,53,50],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[1,2],1],[3,4],[3,5],[3,[[5,[6]]]],[3,[[5,[7]]]],[3,[[5,[8]]]],[[]],[[]],[[]],[[]],[[]],[[]],[3,3],[1,1],[[]],[[]],[9,10],[9,10],0,[[],1],[[],[[11,[3]]]],[1,12],[[1,13],12],[[3,3],4],[[10,14],15],[[10,14],15],[[3,14],15],[[3,14],15],[[1,14],15],0,[16,10],[[]],[17,10],[18,10],[19,3],[4,3],[20,3],[21,3],[[[5,[3]]],3],[22,3],[23,3],[8,3],[[]],[24,3],[25,3],[26,3],[27,3],[28,3],[[],3],[29,3],[[[23,[3]]],3],[30,3],[[[23,[30]]],3],[[],3],[[],3],[31,3],[32,3],[[]],[33,3],[[],12],[33,3],[[],12],[[]],[[]],[[]],[3,3],[3,4],[3,34],[35],0,0,[3,11],[1,[[12,[[23,[30]]]]]],[1,12],[10,[[5,[36]]]],[[]],[[]],[[],19],[[],19],[[],[[12,[[23,[30]]]]]],[[],12],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],37],[[],37],[[],37],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[38,12],[38,12],[38,12],[38,12],[38,12],[[38,8],12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[[38,8],12],[38,12],[38,12],[38,12],[38,12],[[38,8],12],[[38,2],12],[[38,8,2],12],[38,12],[38,12],[38,12],[38,12],[38,12],[38,12],[[38,8],12],[39,[[12,[[38,[40]]]]]],[[[38,[40]]],4],[[[38,[41]],14],15],[[39,14],15],[[]],[[]],[[]],[[]],[38,4],[[],39],[38,12],[38,12],[[],11],[[],11],[[],11],[[],11],[[38,2],12],[[],37],[[],37],[38,12],[38,12],0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[7,[[11,[31,10]]]],[6,[[11,[31,10]]]],[7,[[11,[32,10]]]],[6,32],[7,[[11,[6,10]]]],[7,[[11,[27,10]]]],[7,[[11,[22,10]]]],[7,[[11,[25,10]]]],[7,[[11,[26,10]]]],[7,[[11,[20,10]]]],[6,[[11,[7,10]]]],[7,[[11,[21,10]]]],[7,[[11,[29,10]]]],[7,[[11,[28,10]]]],[7,[[11,[24,10]]]],[7,[[11,[30,10]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[42,42],[7,7],[6,6],[[]],[[]],[[]],[[],[[11,[7]]]],[[],[[11,[6]]]],[[42,42],4],[[7,7],4],[[6,6],4],[[42,14],15],[[7,14],15],[[7,14],15],[[43,14],15],[[6,14],15],[[6,14],15],[[44,14],15],[[]],[[]],[20,7],[26,7],[24,7],[21,7],[29,7],[28,7],[22,7],[25,7],[[]],[30,7],[27,7],[[]],[[]],[32,6],[31,6],[[]],[30,[[11,[42,10]]]],[[]],[[]],[[]],[[]],[[]],[[]],[7,4],[6,4],0,0,[2,[[11,[43,10]]]],[[],[[11,[10]]]],[[42,2],[[11,[7,10]]]],[[42,2],[[11,[6,10]]]],[[],[[11,[30,10]]]],[7,11],[6,11],[[]],[[]],[[]],[[],19],[[],19],[[],11],[24,[[11,[45]]]],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],37],[[],37],[[],37],[[],37],[[],37],[[],37],[[42,[5,[24]]],[[46,[2]]]],[4,[[46,[2]]]],[[],[[46,[2]]]],[31,[[46,[2]]]],[32,[[46,[2]]]],[30,[[46,[2]]]],[27,[[46,[2]]]],[22,[[46,[2]]]],[25,[[46,[2]]]],[25,[[46,[2]]]],[26,[[46,[2]]]],[26,[[46,[2]]]],[20,[[46,[2]]]],[[],[[46,[2]]]],[[],[[46,[2]]]],[45,[[46,[2]]]],[8,[[46,[2]]]],[21,[[46,[2]]]],[29,[[46,[2]]]],[28,[[46,[2]]]],[28,[[46,[2]]]],[24,[[46,[2]]]],[24,[[46,[2]]]],[30,[[46,[2]]]],[[],[[46,[2]]]],0,0,0,0,0,0,0,[[]],[[]],[[]],[[]],[2,[[11,[47,10]]]],[[40,2],[[11,[47,10]]]],[[[49,[48]],2],[[11,[47,10]]]],[[40,14],15],[[]],[[],40],[[]],[[]],[[]],[40,[[46,[2]]]],[[[49,[48]]],[[46,[2]]]],[40,46],[[[49,[48]]],46],[[[49,[48]],23],[[46,[2]]]],[[[49,[48]],19],[[46,[2]]]],[[[49,[48]]],[[46,[2]]]],[[],11],[[],11],[[],11],[[],11],[[],37],[[],37],0,0,0,[[]],[[]],[[]],[[]],[[]],[[]],[[],50],[[[52,[51]]],12],[[[52,[51]]],12],[[[52,[51]]],12],[53,12],[53,12],[53,12],[53,12],[[[53,[51]],14],15],[[50,14],15],[[]],[[]],[[]],[[]],[[]],[[]],[53,4],[51,[[12,[[53,[51]]]]]],[[53,4],12],[53,12],[[53,54],12],[53,12],[53,12],[[53,31],12],[[53,32],12],[[[52,[51]],8],12],[[[52,[51]],8],12],[53,12],[53,12],[[53,27],12],[[53,22],12],[[53,25],12],[[53,26],12],[[53,20],12],[[[52,[51]]],12],[[53,[5,[2]]],12],[[53,8],12],[[53,8,28,8],12],[53,12],[[53,[5,[2]]],12],[53,12],[[53,8],12],[[53,8,2],12],[[53,8,28,8,2],12],[[53,2],12],[[53,8,2],12],[[53,8,28,8,2],12],[[53,21],12],[[53,29],12],[[53,28],12],[[53,24],12],[[53,30],12],[53,12],[[53,8],12],[[53,8,28,8],12],[[[52,[51]]],12],[[50,51],[[12,[[53,[51]]]]]],[[],11],[[],11],[[],11],[[],11],[[],11],[[],11],[[],37],[[],37],[[],37]],"p":[[3,"Config"],[15,"usize"],[4,"Value"],[15,"bool"],[4,"Option"],[3,"Float"],[3,"Integer"],[15,"str"],[8,"Display"],[4,"Error"],[4,"Result"],[6,"Result"],[8,"Read"],[3,"Formatter"],[6,"Result"],[3,"FromUtf8Error"],[3,"Error"],[3,"Utf8Error"],[3,"String"],[15,"i8"],[15,"u128"],[15,"i16"],[3,"Vec"],[15,"u64"],[15,"i32"],[15,"i64"],[15,"i128"],[15,"u32"],[15,"u16"],[15,"u8"],[15,"f32"],[15,"f64"],[8,"IntoIterator"],[3,"Iter"],[3,"Demand"],[8,"Error"],[3,"TypeId"],[3,"Deserializer"],[4,"SymbolMap"],[3,"SliceReader"],[8,"Reader"],[4,"Kind"],[3,"Atom"],[4,"Nucleus"],[4,"Special"],[6,"Result"],[4,"Cow"],[8,"ReadBytesExt"],[3,"IoReader"],[3,"SymbolMap"],[8,"WriteBytesExt"],[3,"MapSerializer"],[3,"Serializer"],[15,"char"],[13,"Message"],[13,"Io"],[13,"InvalidUtf8"],[13,"InvalidKind"],[13,"UnexpectedKind"],[13,"UnknownSymbol"],[13,"Bool"],[13,"Integer"],[13,"Float"],[13,"Bytes"],[13,"String"],[13,"Sequence"],[13,"Mappings"],[13,"Owned"],[13,"Persistent"],[13,"Borrowed"],[13,"Boolean"],[13,"Integer"],[13,"Float"],[13,"Bytes"]]},\
"xtask":{"doc":"","t":[4,11,11,11,11,5,11,11,11,11],"n":["Config","borrow","borrow_mut","from","into","main","paths","try_from","try_into","type_id"],"q":["xtask","","","","","","","","",""],"d":["","","","Returns the argument unchanged.","Calls <code>U::from(self)</code>.","","","","",""],"i":[0,6,6,6,6,0,6,6,6,6],"f":[0,[[]],[[]],[[]],[[]],[[],1],[[],[[3,[2]]]],[[],4],[[],4],[[],5]],"p":[[6,"Result"],[3,"String"],[3,"Vec"],[4,"Result"],[3,"TypeId"],[4,"Config"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};
