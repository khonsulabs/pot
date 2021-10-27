var searchIndex = JSON.parse('{\
"pot":{"doc":"A concise serialization format written for <code>BonsaiDb</code>.","t":[3,13,4,13,13,13,13,13,13,13,13,6,13,13,13,13,13,11,11,11,11,11,11,11,0,11,11,11,11,0,11,11,11,11,11,5,11,11,0,0,11,11,11,5,11,11,11,11,11,11,12,12,12,12,12,12,12,13,3,13,13,4,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,12,12,3,13,13,13,13,4,13,13,13,13,13,13,13,13,4,13,4,13,13,4,13,13,13,13,13,13,13,13,12,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,12,11,12,5,5,11,11,5,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,12,12,12,12,12,12,12,12,12,12,12,12,12,12,12,3,8,3,11,11,11,11,10,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,3,3,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11,11],"n":["Config","Eof","Error","ImpreciseCastWouldLoseData","IncompatibleVersion","InvalidAtomHeader","InvalidKind","InvalidUtf8","Io","Message","NotAPot","Result","SequenceSizeMustBeKnown","TooManyBytesRead","TrailingBytes","UnexpectedKind","UnknownSymbol","allocation_budget","borrow","borrow","borrow_mut","borrow_mut","custom","custom","de","default","deserialize","fmt","fmt","format","from","from","from","from","from","from_slice","into","into","reader","ser","serialize","source","to_string","to_vec","try_from","try_from","try_into","try_into","type_id","type_id","0","0","0","0","0","0","1","Borrowed","Deserializer","Owned","Persistent","SymbolMap","borrow","borrow","borrow_mut","borrow_mut","deserializer_for_slice","end_of_input","fmt","fmt","from","from","into","into","new","try_from","try_from","try_into","try_into","type_id","type_id","0","0","0","Atom","Bytes","Bytes","F32","F64","Float","Float","Float","I128","I16","I32","I64","I8","Int","Integer","Integer","Kind","Map","None","Nucleus","Sequence","Symbol","U128","U16","U32","U64","U8","UInt","arg","as_f32","as_f32","as_f64","as_f64","as_i128","as_i16","as_i32","as_i64","as_i8","as_integer","as_u128","as_u16","as_u32","as_u64","as_u8","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","clone","clone","clone_into","clone_into","eq","eq","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from_u8","into","into","into","into","into","kind","ne","nucleus","read_atom","read_atom_header","read_from","read_from","read_header","to_owned","to_owned","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","write_atom_header","write_bytes","write_f32","write_f64","write_header","write_i128","write_i16","write_i24","write_i32","write_i48","write_i64","write_i8","write_none","write_str","write_u128","write_u16","write_u24","write_u32","write_u48","write_u64","write_u8","write_unit","0","0","0","0","0","0","0","0","0","0","0","0","0","0","0","IoReader","Reader","SliceReader","borrow","borrow","borrow_mut","borrow_mut","buffered_read_bytes","buffered_read_bytes","fmt","fmt","from","from","from","into","into","read","read_exact","try_from","try_from","try_into","try_into","type_id","type_id","Serializer","SymbolMap","borrow","borrow","borrow_mut","borrow_mut","default","fmt","fmt","from","from","into","into","new","serializer_for","try_from","try_from","try_into","try_into","type_id","type_id"],"q":["pot","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::Error","","","","","","","pot::de","","","","","","","","","","","","","","","","","","","","","","","","pot::de::SymbolMap","","","pot::format","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","pot::format::Float","","pot::format::Integer","","","","","","","","","","pot::format::Nucleus","","","pot::reader","","","","","","","","","","","","","","","","","","","","","","","","pot::ser","","","","","","","","","","","","","","","","","","","",""],"d":["Serialization and deserialization configuration.","Expected more data but encountered the end of the input.","All errors that <code>Pot</code> may return.","A numerical value could not be handled without losing …","Data was written with an incompatible version.","An atom header was incorrectly formatted.","An unknown kind was encountered. Generally a sign that …","String data contained invalid utf-8 characters.","An error occurred from io.","A generic error occurred.","Payload is not a <code>Pot</code> payload.","A result alias that returns <code>Error</code>.","A sequence of unknown size cannot be serialized.","The amount of data read exceeds the configured maximum …","Extra data appeared at the end of the input.","Encountered an unexpected atom kind.","A requested symbol id was not found.","Sets the maximum number of bytes able to be allocated. …","","","","","","","Types for deserializing pots.","","Deserializes a value from a slice using the configured …","","","Low-level interface for reading and writing the pot format.","","","","","","Restore a previously serialized value from a pot.","","","Types for reading data.","Types for serializing pots.","Serializes a value to a <code>Vec</code> using the configured options.","","","Serialize <code>value</code> into a pot.","","","","","","","","","","","","","","A list of borrowed symbols.","Deserializer for the <code>Pot</code> format.","An owned list of symbols.","A mutable reference to an owned list of symbols.","A collection of deserialized symbols.","","","","","Returns a deserializer for <code>slice</code>.","Returns true if the input has been consumed completely.","","","","","","","Returns a new symbol map that will persist symbols between …","","","","","","","","","","An encoded <code>Kind</code>, argument, and optional contained value.","A series of bytes. The argument is the length. The bytes …","A buffer of bytes.","An f32 value.","An f64 value.","A floating point number that can safely convert between …","A floating point value. Argument is the byte length, minus …","A floating point value.","An i128 value.","An i16 value.","An i32 value.","An i64 value.","An i8 value.","A signed integer. Argument is the byte length, minus one. …","An integer type that can safely convert between other …","An integer value.","The type of an atom.","A list of key-value pairs. Argument is the count of …","No value","A value contained within an <code>Atom</code>.","A list of atoms. Argument is the count of atoms in the …","A symbol. If the least-significant bit of the arg is 0, …","An u128 value.","An u16 value.","An u32 value.","An u64 value.","An u8 value.","An unsigned integer. Argument is the byte length, minus …","The argument contained in the atom header.","Converts this integer to an f32, but only if it can be …","Returns this number as an f32, if it can be done without …","Converts this integer to an f64, but only if it can be …","Returns this number as an f64.","Returns the contained value as an i64, or an error if the …","Returns the contained value as an i16, or an error if the …","Returns the contained value as an i32, or an error if the …","Returns the contained value as an i64, or an error if the …","Returns the contained value as an i8, or an error if the …","Returns this number as an <code>Integer</code>, if the stored value has …","Returns the contained value as an u64, or an error if the …","Returns the contained value as an u16, or an error if the …","Returns the contained value as an u32, or an error if the …","Returns the contained value as an u64, or an error if the …","Returns the contained value as an u8, or an error if the …","","","","","","","","","","","","","","","","","","","","","","","","","","","Converts from a u8. Returns an error if <code>kind</code> is an invalid …","","","","","","The type of atom.","","The contained value, if any.","Reads an atom.","Reads an atom header (kind and argument).","Reads an integer based on the atom header (<code>kind</code> and …","Reads a floating point number given the atom <code>kind</code> and …","Reads a Pot header. See <code>write_header</code> for more information. …","","","","","","","","","","","","","","","","","","Writes an atom header into <code>writer</code>.","Writes an <code>Kind::Bytes</code> atom with the given value.","Writes an <code>Kind::Float</code> atom with the given value.","Writes an <code>Kind::Float</code> atom with the given value.","Writes the Pot header. A u32 written in big endian. The …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes a <code>Kind::None</code> atom.","Writes an <code>Kind::Bytes</code> atom with the bytes of the string.","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::Int</code> atom with the given value. Will encode …","Writes an <code>Kind::UInt</code> atom with the given value.","Writes a <code>Kind::None</code> atom. Pot doesn’t distinguish …","","","","","","","","","","","","","","","","A reader over <code>ReadBytesExt</code>.","A reader that can temporarily buffer bytes read.","Reads data from a slice.","","","","","Read exactly <code>length</code> bytes and return a reference to the …","","","","","","","","","","","","","","","","","A <code>Pot</code> serializer.","A list of previously serialized symbols.","","","","","","","","","","","","Returns a new serializer outputting written bytes into …","Returns a serializer that writes into <code>output</code> that persists …","","","","","",""],"i":[0,1,0,1,1,1,1,1,1,1,1,0,1,1,1,1,1,2,2,1,2,1,1,1,0,2,2,1,1,0,2,1,1,1,1,0,2,1,0,0,2,1,1,0,2,1,2,1,2,1,3,4,5,6,7,8,7,9,0,9,9,0,10,9,10,9,9,10,10,9,10,9,10,9,9,10,9,10,9,10,9,11,12,13,0,14,15,16,16,0,14,15,17,17,17,17,17,14,0,15,0,14,14,0,14,14,17,17,17,17,17,14,18,17,16,17,16,17,17,17,17,17,16,17,17,17,17,17,14,17,18,16,15,14,17,18,16,15,14,17,14,17,14,17,14,17,18,16,15,14,17,18,16,15,14,14,17,18,16,15,18,17,18,0,0,17,16,0,14,17,14,17,18,16,15,14,17,18,16,15,14,17,18,16,15,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,19,20,21,22,23,24,25,26,27,28,29,30,31,32,33,0,0,0,34,35,34,35,36,34,34,35,34,34,35,34,35,34,34,34,35,34,35,34,35,0,0,37,38,37,38,38,37,38,37,38,37,38,37,38,37,38,37,38,37,38],"f":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[["usize",15]]],[[]],[[]],[[]],[[]],[[["display",8]]],[[["display",8]]],null,[[]],[[],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],null,[[]],[[["error",3]]],[[["utf8error",3]]],[[["fromutf8error",3]]],[[]],[[],["result",6]],[[]],[[]],null,null,[[],[["result",6,["vec"]],["vec",3,["u8"]]]],[[],[["error",8],["option",4,["error"]]]],[[],["string",3]],[[],[["result",6,["vec"]],["vec",3,["u8"]]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[],[["deserializer",3,["slicereader"]],["result",6,["deserializer"]]]],[[],["bool",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[],[["f32",15],["error",4],["result",4,["f32","error"]]]],[[],[["f32",15],["error",4],["result",4,["f32","error"]]]],[[],[["error",4],["result",4,["f64","error"]],["f64",15]]],[[],["f64",15]],[[],[["i128",15],["error",4],["result",4,["i128","error"]]]],[[],[["result",4,["i16","error"]],["error",4],["i16",15]]],[[],[["i32",15],["error",4],["result",4,["i32","error"]]]],[[],[["error",4],["result",4,["i64","error"]],["i64",15]]],[[],[["i8",15],["error",4],["result",4,["i8","error"]]]],[[],[["result",4,["integer","error"]],["integer",4],["error",4]]],[[],[["error",4],["u128",15],["result",4,["u128","error"]]]],[[],[["u16",15],["result",4,["u16","error"]],["error",4]]],[[],[["result",4,["u32","error"]],["u32",15],["error",4]]],[[],[["u64",15],["result",4,["u64","error"]],["error",4]]],[[],[["u8",15],["error",4],["result",4,["u8","error"]]]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],["kind",4]],[[],["integer",4]],[[]],[[]],[[["kind",4]],["bool",15]],[[["integer",4]],["bool",15]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[["u8",15]],[["result",4,["error"]],["error",4]]],[[]],[[]],[[]],[[]],[[]],null,[[["integer",4]],["bool",15]],null,[[["usize",15]],[["error",4],["atom",3],["result",4,["atom","error"]]]],[[],[["result",4,["error"]],["error",4]]],[[["usize",15],["kind",4]],[["result",4,["error"]],["error",4]]],[[["usize",15],["kind",4]],[["result",4,["error"]],["error",4]]],[[],[["u8",15],["error",4],["result",4,["u8","error"]]]],[[]],[[]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[],["typeid",3]],[[["u64",15],["option",4,["u64"]],["kind",4]],[["result",6,["usize"]],["usize",15]]],[[],[["result",6,["usize"]],["usize",15]]],[[["f32",15]],[["result",6,["usize"]],["usize",15]]],[[["f64",15]],[["result",6,["usize"]],["usize",15]]],[[["u8",15]],[["result",6,["usize"]],["usize",15]]],[[["i128",15]],[["result",6,["usize"]],["usize",15]]],[[["i16",15]],[["result",6,["usize"]],["usize",15]]],[[["i32",15]],[["result",6,["usize"]],["usize",15]]],[[["i32",15]],[["result",6,["usize"]],["usize",15]]],[[["i64",15]],[["result",6,["usize"]],["usize",15]]],[[["i64",15]],[["result",6,["usize"]],["usize",15]]],[[["i8",15]],[["result",6,["usize"]],["usize",15]]],[[],[["result",6,["usize"]],["usize",15]]],[[["str",15]],[["result",6,["usize"]],["usize",15]]],[[["u128",15]],[["result",6,["usize"]],["usize",15]]],[[["u16",15]],[["result",6,["usize"]],["usize",15]]],[[["u32",15]],[["result",6,["usize"]],["usize",15]]],[[["u32",15]],[["result",6,["usize"]],["usize",15]]],[[["u64",15]],[["result",6,["usize"]],["usize",15]]],[[["u64",15]],[["result",6,["usize"]],["usize",15]]],[[["u8",15]],[["result",6,["usize"]],["usize",15]]],[[],[["result",6,["usize"]],["usize",15]]],null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,[[]],[[]],[[]],[[]],[[["usize",15]],[["error",4],["result",4,["error"]]]],[[["usize",15]],[["error",4],["result",4,["error"]]]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[]],[[],[["result",6,["usize"]],["usize",15]]],[[],["result",6]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]],null,null,[[]],[[]],[[]],[[]],[[]],[[["formatter",3]],["result",6]],[[["formatter",3]],["result",6]],[[]],[[]],[[]],[[]],[[],["result",6]],[[["writebytesext",8],["debug",8]],[["serializer",3],["result",6,["serializer"]]]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]],[[],["typeid",3]]],"p":[[4,"Error"],[3,"Config"],[13,"Message"],[13,"Io"],[13,"InvalidUtf8"],[13,"InvalidKind"],[13,"UnexpectedKind"],[13,"UnknownSymbol"],[4,"SymbolMap"],[3,"Deserializer"],[13,"Owned"],[13,"Persistent"],[13,"Borrowed"],[4,"Kind"],[4,"Nucleus"],[4,"Float"],[4,"Integer"],[3,"Atom"],[13,"F64"],[13,"F32"],[13,"I8"],[13,"I16"],[13,"I32"],[13,"I64"],[13,"I128"],[13,"U8"],[13,"U16"],[13,"U32"],[13,"U64"],[13,"U128"],[13,"Integer"],[13,"Float"],[13,"Bytes"],[3,"SliceReader"],[3,"IoReader"],[8,"Reader"],[3,"Serializer"],[3,"SymbolMap"]]},\
"xtask":{"doc":"","t":[3,11,11,11,11,11,5,11,11,11],"n":["CoverageConfig","borrow","borrow_mut","from","ignore_paths","into","main","try_from","try_into","type_id"],"q":["xtask","","","","","","","","",""],"d":["","","","","","","","","",""],"i":[0,1,1,1,1,1,0,1,1,1],"f":[null,[[]],[[]],[[]],[[],[["vec",3,["string"]],["string",3]]],[[]],[[],["result",6]],[[],["result",4]],[[],["result",4]],[[],["typeid",3]]],"p":[[3,"CoverageConfig"]]}\
}');
if (window.initSearch) {window.initSearch(searchIndex)};