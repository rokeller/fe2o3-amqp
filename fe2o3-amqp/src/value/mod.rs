use ordered_float::OrderedFloat;
use serde_bytes::ByteBuf;
use std::collections::BTreeMap;

use crate::{format_code::EncodingCodes, types::{Array, Dec128, Dec32, Dec64, Described, Symbol, Timestamp, Uuid}};

pub mod de;
pub mod ser;

pub const U32_MAX_AS_USIZE: usize = u32::MAX as usize;
pub const VALUE: &str = "VALUE";

/// Primitive type definitions
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// Described type
    Described(Described<Value>),

    /// Indicates an empty value
    ///
    /// encoding code = 0x40,
    /// category = fixed, width = 0,
    /// label = "the null value"
    Null,

    /// Represents a true or false value
    ///
    /// encoding code = 0x56
    /// category = fixed, width = 1
    /// label = "boolean with the octet 0x00 being false and octet 0x01 being true"
    ///
    /// encoding name = "true", encoding code = 0x41
    /// category = fixed, width = 0
    /// label = "the boolean value true"
    ///
    /// encoding name = "false", encoding code = 0x42
    /// category = fixed, width = 0
    /// label = "the boolean value false"
    Bool(bool),

    /// Integer in the range 0 to 2^8-1 inclusive
    ///
    /// encoding code = 0x50,
    /// category = fixed, width = 1
    /// label = "8-bit unsigned integer"
    Ubyte(u8),

    /// Integer in the range 0 to 2^16-1 inclusive
    ///
    /// encoding code = 0x60,
    /// category = fixed, width = 2
    /// label = "16-bit unsigned integer in network byte order"
    /// (AKA. Big-Endian, rust uses BigEndian by default)
    Ushort(u16),

    /// Integer in the range 0 to 2^32-1 inclusive
    ///
    /// encoding code = 0x70,
    /// category = fixed, width = 4
    /// label = "32-bit unsigned integer in network byte order"
    /// (AKA. Big-Endian, rust uses BigEndian by default)
    ///
    /// encoding name = "smalluint", encoding code = 0x52
    /// category = fixed, width = 1
    /// label = "unsigned integer value in the range 0 to 255 inclusive"
    ///
    /// encoding name = "uint0", encoding code = 0x43
    /// category = fixed, width = 0
    /// label = "the uint value 0"
    Uint(u32),

    /// Integer in the range 0 to 2^64-1 inclusive
    ///
    /// encoding code = 0x80,
    /// category = fixed, width = 8
    /// label = "64-bit unsigned integer in network byte order"
    /// (AKA. Big-Endian, rust uses BigEndian by default)
    ///
    /// encoding name = "smallulong", encoding code = 0x53
    /// category = fixed, width = 1
    /// label = "unsigned long value in the range 0 to 255 inclusive"
    ///
    /// encoding name = "ulong0", encoding code = 0x44
    /// category = fixed, width = 0
    /// label = "the ulong value 0"
    Ulong(u64),

    /// Integer in the range -(2^7) to 2^7-1 inclusive
    ///
    /// encoding code = 0x51,
    /// category = fixed, width = 1
    /// label = "8-bit two's-complement integer"
    Byte(i8),

    /// Integer in the range -(2^15) to 2^15-1 inclusive
    ///
    /// encoding code = 0x61,
    /// category = fixed, width = 2
    /// label = "16-bit two’s-complement integer in network byte order"
    Short(i16),

    /// Integer in the range -(2^31) to 2^31-1 inclusive
    ///
    /// encoding code = 0x71,
    /// category = fixed, width = 4
    /// label = "32-bit two’s-complement integer in network byte order"
    ///
    /// encoding name = "smallint", encoding code = 0x54
    /// category = fixed, width = 1
    /// label = "8-bit two’s-complement integer"
    Int(i32),

    /// Integer in the range -(2^63) to 2^63-1 inclusive
    ///
    /// encoding code = 0x81,
    /// category = fixed, width = 8
    /// label = "64-bit two’s-complement integer in network byte order"
    ///
    /// encoding name = "smalllong", encoding code = 0x55
    /// category = fixed, width = 1
    /// label = "8-bit two’s-complement integer"
    Long(i64),

    /// 32-bit floating point number (IEEE 754-2008 binary32)
    ///
    /// encoding name = "ieee-754", encoding code = 0x72
    /// category = fixed, width = 4
    /// label = "IEEE 754-2008 binary32"
    Float(OrderedFloat<f32>),

    /// 64-bit floating point number (IEEE 754-2008 binary64).
    ///
    /// encoding name = "ieee-754", encoding code = 0x82
    /// category = fixed, width = 8
    /// label = "IEEE 754-2008 binary64"
    Double(OrderedFloat<f64>),

    /// 32-bit decimal number (IEEE 754-2008 decimal32).
    ///
    /// encoding name = "ieee-754", encoding code = 0x74
    /// category = fixed, width = 4
    /// label = "IEEE 754-2008 decimal32 using the Binary Integer Decimal encoding"
    Decimal32(Dec32),

    /// 64-bit decimal number (IEEE 754-2008 decimal64).
    ///
    /// encoding name = "ieee-754", encoding code = 0x84
    /// category = fixed, width = 8
    /// label = "IEEE 754-2008 decimal64 using the Binary Integer Decimal encoding"
    Decimal64(Dec64),

    /// 128-bit decimal number (IEEE 754-2008 decimal128).
    ///
    /// encoding name = "ieee-754", encoding code = 0x94
    /// category = fixed, width = 16
    /// label = "IEEE 754-2008 decimal128 using the Binary Integer Decimal encoding"
    Decimal128(Dec128),

    /// A single Unicode character
    ///
    /// encoding name = "utf32", encoding code = 0x73
    /// category = fixed, width = 4,
    /// label = "a UTF-32BE encoded Unicode character"
    Char(char),

    /// An absolute point in time
    ///
    /// encoding name = "ms64", code = 0x83,
    /// category = fixed, width = 8
    /// label = "64-bit two’s-complement integer representing milliseconds since the unix epoch"
    Timestamp(Timestamp),

    /// A universally unique identifier as defined by RFC-4122 in section 4.1.2
    ///
    /// encoding code = 0x98,
    /// category = fixed, width = 16,
    /// label="UUID as defined in section 4.1.2 of RFC-4122"
    Uuid(Uuid),

    /// A sequence of octets.
    ///
    /// encoding name = "vbin8", encoding code = 0xa0
    /// category = variable, width = 1
    /// label = "up to 2^8 - 1 octets of binary data"
    ///
    /// encoding name = "vbin32", encoding code = 0xb0,
    /// category = variable, width = 4,
    /// label="up to 2^32 - 1 octets of binary data"
    Binary(ByteBuf),

    /// A sequence of Unicode characters.
    ///
    /// encoding name = "str8-utf8", encoding code = 0xa1,
    /// category = variable, width = 1
    /// label = "up to 2^8 - 1 octets worth of UTF-8 Unicode (with no byte order mark)"
    ///
    /// encoding name = "str32-utf8", encoding code = 0xb1
    /// category = variable, width = 4
    /// label="up to 2^32 - 1 octets worth of UTF-8 Unicode (with no byte order mark)"
    String(String),

    /// Symbolic values from a constrained domain.
    ///
    /// encoding name = "sym8", encoding code = 0xa3,
    /// category = variable, width = 1
    /// label="up to 2^8 - 1 seven bit ASCII characters representing a symbolic value"
    ///
    /// encoding name = "sym32", encoding code = 0xb3
    /// category = variable, width = 4
    /// label="up to 2^32 - 1 seven bit ASCII characters representing a symbolic value"
    ///
    /// Symbols are values from a constrained domain.
    /// Although the set of possible domains is open-ended,
    /// typically the both number and size of symbols in use for any
    /// given application will be small, e.g. small enough that it is reasonable
    /// to cache all the distinct values. Symbols are encoded as ASCII characters [ASCII].
    Symbol(Symbol),

    /// A sequence of polymorphic values.
    ///
    /// encoding name = "list0", encoding code = 0x45
    /// category = fixed, width = 0,
    /// label="the empty list (i.e. the list with no elements)"
    ///
    /// encoding name = "list8", encoding code = 0xc0
    /// category = compound, width = 1
    /// label="up to 2^8 - 1 list elements with total size less than 2^8 octets
    ///
    /// encoding name = "list32", encoding code = 0xd0
    /// category = compound, width = 4
    /// label="up to 2^32 - 1 list elements with total size less than 2^32 octets"
    List(Vec<Value>),

    /// A polymorphic mapping from distinct keys to values.
    ///
    /// encoding name = "map8", encoding code = 0xc1,
    /// category = compound, width = 1
    /// label="up to 2^8 - 1 octets of encoded map data"
    ///
    /// encoding name = "map32", encoding code = 0xd1,
    /// category = compound, width = 4
    /// label="up to 2^32 - 1 octets of encoded map data
    ///
    /// Map encodings MUST contain an even number of items (i.e. an equal number of keys and values).
    /// A map in which there exist two identical key values is invalid. Unless known to be otherwise,
    /// maps MUST be considered to be ordered, that is, the order of the key-value pairs is semantically
    /// important and two maps which are different only in the order in which their key-value pairs are
    /// encoded are not equal.
    ///
    /// Note: Can only use BTreeMap as it must be considered to be ordered
    Map(BTreeMap<Value, Value>),

    /// A sequence of values of a single type.
    ///
    /// encoding name = "array8", encoding code = 0xe0
    /// category = array, width = 1,
    /// label="up to 2^8 - 1 array elements with total size less than 2^8 octets"
    ///
    /// encoding name = "array32", encoding code = 0xf0,
    /// category = array, width = 4
    /// label="up to 2^32 - 1 array elements with total size less than 2^32 octets"
    Array(Array<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl Value {
    pub fn format_code(&self) -> u8 {
        let code = match *self {
            Value::Described(_) => EncodingCodes::DescribedType,
            Value::Null => EncodingCodes::Null,
            Value::Bool(_) => EncodingCodes::Boolean,
            Value::Ubyte(_) => EncodingCodes::Ubyte,
            Value::Ushort(_) => EncodingCodes::Ushort,
            Value::Uint(_) => EncodingCodes::Uint,
            Value::Ulong(_) => EncodingCodes::Ulong,
            Value::Byte(_) => EncodingCodes::Byte,
            Value::Short(_) => EncodingCodes::Short,
            Value::Int(_) => EncodingCodes::Int,
            Value::Long(_) => EncodingCodes::Long,
            Value::Float(_) => EncodingCodes::Float,
            Value::Double(_) => EncodingCodes::Double,
            Value::Decimal32(_) => EncodingCodes::Decimal32,
            Value::Decimal64(_) => EncodingCodes::Decimal64,
            Value::Decimal128(_) => EncodingCodes::Decimal128,
            Value::Char(_) => EncodingCodes::Char,
            Value::Timestamp(_) => EncodingCodes::Timestamp,
            Value::Uuid(_) => EncodingCodes::Uuid,
            Value::Binary(_) => EncodingCodes::VBin32,
            Value::String(_) => EncodingCodes::Str32,
            Value::Symbol(_) => EncodingCodes::Sym32,
            Value::List(_) => EncodingCodes::List32,
            Value::Map(_) => EncodingCodes::Map32,
            Value::Array(_) => EncodingCodes::Array32,
        };
        code as u8
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;
    use serde::de::DeserializeOwned;

    use crate::de::from_reader;
    use crate::ser::to_vec;
    use crate::types::{Described, Descriptor, Symbol};

    use super::Value;

    fn assert_eq_from_reader_vs_expected<T>(buf: Vec<u8>, expected: T)
    where
        T: DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let deserialized: T = from_reader(buf.as_slice()).unwrap();
        assert_eq!(deserialized, expected)
    }

    #[test]
    fn mem_size_of_value() {
        let size = std::mem::size_of::<String>();
        println!("{:?}", size);
    }

    #[test]
    fn test_value_null() {
        let expected = Value::Null;
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_bool() {
        let expected = Value::Bool(true);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        let expected = Value::Bool(false);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_ubyte() {
        let expected = Value::Ubyte(13);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_ushort() {
        let expected = Value::Ushort(1313);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_uint() {
        // uint0
        let expected = Value::Uint(0);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // smalluint
        let expected = Value::Uint(255);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // uint
        let expected = Value::Uint(u32::MAX);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_ulong() {
        // ulong0
        let expected = Value::Ulong(0);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // smallulong
        let expected = Value::Ulong(255);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // ulong
        let expected = Value::Ulong(u64::MAX);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_byte() {
        let expected = Value::Byte(13);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_short() {
        let expected = Value::Short(1313);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_int() {
        // smallint
        let expected = Value::Int(0);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        let expected = Value::Int(255);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // int
        let expected = Value::Int(i32::MAX);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_long() {
        // smalllong
        let expected = Value::Long(0);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        let expected = Value::Long(255);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);

        // ulong
        let expected = Value::Long(i64::MAX);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_float() {
        let expected = Value::Float(OrderedFloat::from(1.313));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_double() {
        let expected = Value::Double(OrderedFloat::from(13.13));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_decimal32() {
        use crate::types::Dec32;
        let expected = Value::Decimal32(Dec32::from([1, 2, 3, 4]));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_decimal64() {
        use crate::types::Dec64;
        let expected = Value::Decimal64(Dec64::from([1, 2, 3, 4, 5, 6, 7, 8]));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_decimal128() {
        use crate::types::Dec128;
        let expected = Value::Decimal128(Dec128::from([
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        ]));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_char() {
        let expected = Value::Char('a');
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_timestamp() {
        use crate::types::Timestamp;
        let expected = Value::Timestamp(Timestamp::from(13));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_uuid() {
        use crate::types::Uuid;
        let expected = Value::Uuid(Uuid::from([
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        ]));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_binary() {
        use serde_bytes::ByteBuf;
        let expected = Value::Binary(ByteBuf::from(vec![1, 2, 3, 4]));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_string() {
        let expected = Value::String(String::from("amqp"));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_symbol() {
        use crate::types::Symbol;
        let expected = Value::Symbol(Symbol::from("amqp"));
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_list() {
        let expected = Value::List(
            vec![1u32, 2, 3, 4]
                .iter()
                .map(|v| Value::Uint(*v))
                .collect(),
        );
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_map() {
        use std::collections::BTreeMap;
        let mut map = BTreeMap::new();
        map.insert(Value::Uint(13), Value::Bool(true));
        map.insert(Value::Uint(45), Value::Bool(false));
        let expected = Value::Map(map);
        let buf = to_vec(&expected).unwrap();
        assert_eq_from_reader_vs_expected(buf, expected);
    }

    #[test]
    fn test_value_array() {
        use crate::types::Array;
        let vec: Vec<Value> = vec![1i32, 2, 3, 4]
            .iter()
            .map(|val| Value::Int(*val))
            .collect();
        let arr = Array::from(vec);
        let expected = Value::Array(arr);
        let buf = to_vec(&expected).unwrap();
        println!("{:x?}", &buf);

        assert_eq_from_reader_vs_expected(buf, expected);
    }
}
