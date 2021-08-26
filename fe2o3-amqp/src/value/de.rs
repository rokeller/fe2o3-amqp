use std::{collections::BTreeMap, convert::TryInto};

use ordered_float::OrderedFloat;
use serde::de::{self};

use crate::{error::Error, format_code::EncodingCodes, types::{ARRAY, DECIMAL128, DECIMAL32, DECIMAL64, DESCRIPTOR, SYMBOL, TIMESTAMP, UUID}, 
    util::{
        // AMQP_ERROR, CONNECTION_ERROR, LINK_ERROR, SESSION_ERROR, 
        EnumType, NewType
    }
};

use super::{Value, VALUE};

enum Field {
    Null,
    Bool,
    Ubyte,
    Ushort,
    Uint,
    Ulong,
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Decimal32,
    Decimal64,
    Decimal128,
    Char,
    Timestamp,
    Uuid,
    Binary,
    String,
    Symbol,
    List,
    Map,
    Array,
}

struct FieldVisitor {}

impl<'de> de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("field of enum Value")
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        println!(">>> Debug visit_u8 {:x?}", v);

        let field = match v
            .try_into()
            .map_err(|err: Error| de::Error::custom(err.to_string()))?
        {
            EncodingCodes::Null => Field::Null,
            EncodingCodes::Boolean | EncodingCodes::BooleanFalse | EncodingCodes::BooleanTrue => {
                Field::Bool
            }
            EncodingCodes::Ubyte => Field::Ubyte,
            EncodingCodes::Ushort => Field::Ushort,
            EncodingCodes::Uint | EncodingCodes::Uint0 | EncodingCodes::SmallUint => Field::Uint,
            EncodingCodes::Ulong | EncodingCodes::Ulong0 | EncodingCodes::SmallUlong => {
                Field::Ulong
            }
            EncodingCodes::Byte => Field::Byte,
            EncodingCodes::Short => Field::Short,
            EncodingCodes::Int | EncodingCodes::SmallInt => Field::Int,
            EncodingCodes::Long | EncodingCodes::SmallLong => Field::Long,
            EncodingCodes::Float => Field::Float,
            EncodingCodes::Double => Field::Double,
            EncodingCodes::Decimal32 => Field::Decimal32,
            EncodingCodes::Decimal64 => Field::Decimal64,
            EncodingCodes::Decimal128 => Field::Decimal128,
            EncodingCodes::Char => Field::Char,
            EncodingCodes::Timestamp => Field::Timestamp,
            EncodingCodes::Uuid => Field::Uuid,
            EncodingCodes::VBin32 | EncodingCodes::VBin8 => Field::Binary,
            EncodingCodes::Str32 | EncodingCodes::Str8 => Field::String,
            EncodingCodes::Sym32 | EncodingCodes::Sym8 => Field::Symbol,
            EncodingCodes::List0 | EncodingCodes::List32 | EncodingCodes::List8 => Field::List,
            EncodingCodes::Map32 | EncodingCodes::Map8 => Field::Map,
            EncodingCodes::Array32 | EncodingCodes::Array8 => Field::Array,

            // The `Value` type cannot hold a `Described` type
            EncodingCodes::DescribedType => {
                return Err(de::Error::custom(
                    "Described type in Value enum is not supported yet",
                ))
            } // EncodingCodes::DescribedType => Field::List, // could probably treat it as a list of two items
        };
        Ok(field)
    }
}

impl<'de> de::Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitor {})
    }
}

struct Visitor {}

impl<'de> de::Visitor<'de> for Visitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("enum Value")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        use de::VariantAccess;
        let (val, de) = data.variant()?;

        match val {
            Field::Null => {
                let _: () = de.newtype_variant()?;
                Ok(Value::Null)
            }
            Field::Bool => {
                let val = de.newtype_variant()?;
                Ok(Value::Bool(val))
            }
            Field::Ubyte => {
                let val = de.newtype_variant()?;
                Ok(Value::Ubyte(val))
            }
            Field::Ushort => {
                let val = de.newtype_variant()?;
                Ok(Value::Ushort(val))
            }
            Field::Uint => {
                let val = de.newtype_variant()?;
                Ok(Value::Uint(val))
            }
            Field::Ulong => {
                let val = de.newtype_variant()?;
                Ok(Value::Ulong(val))
            }
            Field::Byte => {
                let val = de.newtype_variant()?;
                Ok(Value::Byte(val))
            }
            Field::Short => {
                let val = de.newtype_variant()?;
                Ok(Value::Short(val))
            }
            Field::Int => {
                let val = de.newtype_variant()?;
                Ok(Value::Int(val))
            }
            Field::Long => {
                let val = de.newtype_variant()?;
                Ok(Value::Long(val))
            }
            Field::Float => {
                let val: f32 = de.newtype_variant()?;
                Ok(Value::Float(OrderedFloat::from(val)))
            }
            Field::Double => {
                let val: f64 = de.newtype_variant()?;
                Ok(Value::Double(OrderedFloat::from(val)))
            }
            Field::Decimal32 => {
                let val = de.newtype_variant()?;
                Ok(Value::Decimal32(val))
            }
            Field::Decimal64 => {
                let val = de.newtype_variant()?;
                Ok(Value::Decimal64(val))
            }
            Field::Decimal128 => {
                let val = de.newtype_variant()?;
                Ok(Value::Decimal128(val))
            }
            Field::Char => {
                let val = de.newtype_variant()?;
                Ok(Value::Char(val))
            }
            Field::Timestamp => {
                let val = de.newtype_variant()?;
                Ok(Value::Timestamp(val))
            }
            Field::Uuid => {
                let val = de.newtype_variant()?;
                Ok(Value::Uuid(val))
            }
            Field::Binary => {
                let val = de.newtype_variant()?;
                Ok(Value::Binary(val))
            }
            Field::String => {
                let val = de.newtype_variant()?;
                Ok(Value::String(val))
            }
            Field::Symbol => {
                let val = de.newtype_variant()?;
                Ok(Value::Symbol(val))
            }
            Field::List => {
                let val = de.newtype_variant()?;
                Ok(Value::List(val))
            }
            Field::Map => {
                let val = de.newtype_variant()?;
                Ok(Value::Map(val))
            }
            Field::Array => {
                let val = de.newtype_variant()?;
                Ok(Value::Array(val))
            }
        }
    }
}

impl<'de> de::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const VARIANTS: &'static [&'static str] = &[
            "Null",
            "Bool",
            "Ubyte",
            "Ushort",
            "Uint",
            "Ulong",
            "Byte",
            "Short",
            "Int",
            "Long",
            "Float",
            "Double",
            "Decimal32",
            "Decimal64",
            "Decimal128",
            "Char",
            "Timestamp",
            "Uuid",
            "Binary",
            "String",
            "Symbol",
            "List",
            "Map",
            "Array",
        ];
        deserializer.deserialize_enum(VALUE, VARIANTS, Visitor {})
    }
}

pub fn from_value<T: de::DeserializeOwned>(value: Value) -> Result<T, Error> {
    let de = Deserializer::new(value);
    T::deserialize(de)
}

pub struct Deserializer {
    new_type: NewType,
    value: Value,
    enum_type: EnumType,
}

impl Deserializer {
    pub fn new(value: Value) -> Self {
        Self {
            new_type: Default::default(),
            enum_type: Default::default(),
            value,
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match &self.value {
            Value::Null => self.deserialize_unit(visitor),
            Value::Bool(_) => self.deserialize_bool(visitor),
            Value::Ubyte(_) => self.deserialize_u8(visitor),
            Value::Ushort(_) => self.deserialize_u16(visitor),
            Value::Uint(_) => self.deserialize_u32(visitor),
            Value::Ulong(_) => self.deserialize_u64(visitor),
            Value::Byte(_) => self.deserialize_i8(visitor),
            Value::Short(_) => self.deserialize_i16(visitor),
            Value::Int(_) => self.deserialize_i32(visitor),
            Value::Long(_) => self.deserialize_i64(visitor),
            Value::Float(_) => self.deserialize_f32(visitor),
            Value::Double(_) => self.deserialize_f64(visitor),
            Value::Decimal32(_) => self.deserialize_newtype_struct(DECIMAL32, visitor),
            Value::Decimal64(_) => self.deserialize_newtype_struct(DECIMAL64, visitor),
            Value::Decimal128(_) => self.deserialize_newtype_struct(DECIMAL128, visitor),
            Value::Char(_) => self.deserialize_char(visitor),
            Value::Timestamp(_) => self.deserialize_newtype_struct(TIMESTAMP, visitor),
            Value::Uuid(_) => self.deserialize_newtype_struct(UUID, visitor),
            Value::Binary(_) => self.deserialize_byte_buf(visitor),
            Value::String(_) => self.deserialize_string(visitor),
            Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
            Value::List(_) => self.deserialize_seq(visitor),
            Value::Map(_) => self.deserialize_map(visitor),
            Value::Array(_) => self.deserialize_newtype_struct(ARRAY, visitor),
        }
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Byte(v) => visitor.visit_i8(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Short(v) => visitor.visit_i16(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Int(v) => visitor.visit_i32(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.new_type {
            NewType::None => match self.value {
                Value::Long(v) => visitor.visit_i64(v),
                _ => Err(Error::InvalidValue),
            },
            NewType::Timestamp => match self.value {
                Value::Timestamp(ref v) => visitor.visit_i64(v.milliseconds()),
                _ => Err(Error::InvalidValue),
            },
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Ubyte(v) => visitor.visit_u8(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Ushort(v) => visitor.visit_u16(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Uint(v) => visitor.visit_u32(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Ulong(v) => visitor.visit_u64(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Float(v) => visitor.visit_f32(v.into_inner()),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Double(v) => visitor.visit_f64(v.into_inner()),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Char(v) => visitor.visit_char(v),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.new_type {
            NewType::None => match self.value {
                Value::String(v) => visitor.visit_string(v),
                _ => Err(Error::InvalidValue),
            },
            NewType::Symbol => match self.value {
                Value::Symbol(v) => visitor.visit_string(v.into_inner()),
                _ => Err(Error::InvalidValue),
            },
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.new_type {
            NewType::None => match self.value {
                Value::Binary(v) => visitor.visit_byte_buf(v.into_vec()),
                _ => Err(Error::InvalidValue),
            },
            NewType::Dec32 => match self.value {
                Value::Decimal32(v) => visitor.visit_byte_buf(v.into_inner().to_vec()),
                _ => Err(Error::InvalidValue),
            },
            NewType::Dec64 => match self.value {
                Value::Decimal64(v) => visitor.visit_byte_buf(v.into_inner().to_vec()),
                _ => Err(Error::InvalidValue),
            },
            NewType::Dec128 => match self.value {
                Value::Decimal128(v) => visitor.visit_byte_buf(v.into_inner().to_vec()),
                _ => Err(Error::InvalidValue),
            },
            NewType::Uuid => match self.value {
                Value::Uuid(v) => visitor.visit_byte_buf(v.into_inner().to_vec()),
                _ => Err(Error::InvalidValue),
            },
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Null => visitor.visit_unit(),
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == SYMBOL {
            self.new_type = NewType::Symbol;
        } else if name == DECIMAL32 {
            self.new_type = NewType::Dec32;
        } else if name == DECIMAL64 {
            self.new_type = NewType::Dec64;
        } else if name == DECIMAL128 {
            self.new_type = NewType::Dec128;
        } else if name == UUID {
            self.new_type = NewType::Uuid;
        } else if name == TIMESTAMP {
            self.new_type = NewType::Timestamp;
        }
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.new_type {
            NewType::None => match self.value {
                Value::List(v) => {
                    let iter = v.into_iter();
                    visitor.visit_seq(SeqAccess { iter })
                }
                _ => Err(Error::InvalidValue),
            },
            NewType::Array => match self.value {
                Value::Array(v) => {
                    let v = v.into_inner();
                    let iter = v.into_iter();
                    visitor.visit_seq(SeqAccess { iter })
                }
                _ => Err(Error::InvalidValue),
            },
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(fields.len(), visitor)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Map(map) => {
                let iter = map.into_iter();
                visitor.visit_map(MapAccess { iter })
            }
            _ => Err(Error::InvalidValue),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        mut self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == VALUE {
            self.enum_type = EnumType::Value;
            self.deserialize_any(visitor)
        } else if name == DESCRIPTOR {
            self.enum_type = EnumType::Descriptor;
            match &self.value {
                Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
                Value::Ulong(_) => self.deserialize_u64(visitor),
                _ => Err(Error::InvalidValue),
            }
        // } else if name == AMQP_ERROR {
        //     self.enum_type = EnumType::AmqpError;
        //     match self.value {
        //         Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
        //         _ => Err(Error::InvalidValue)
        //     }
        // } else if name == CONNECTION_ERROR {
        //     self.enum_type = EnumType::ConnectionError;
        //     match self.value {
        //         Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
        //         _ => Err(Error::InvalidValue)
        //     }
        // } else if name == SESSION_ERROR {
        //     self.enum_type = EnumType::SessionError;
        //     match self.value {
        //         Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
        //         _ => Err(Error::InvalidValue)
        //     }
        // } else if name == LINK_ERROR {
        //     self.enum_type = EnumType::LinkError;
        //     match self.value {
        //         Value::Symbol(_) => self.deserialize_newtype_struct(SYMBOL, visitor),
        //         _ => Err(Error::InvalidValue)
        //     }
        } else {
            match self.value {
                // An Uint should represent a unit_variant
                v @ Value::Uint(_) => visitor.visit_enum(VariantAccess {
                    iter: vec![v].into_iter(),
                }),
                Value::List(v) => visitor.visit_enum(VariantAccess {
                    iter: v.into_iter(),
                }),
                _ => Err(Error::InvalidValue),
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match &self.enum_type {
            EnumType::Value => {
                let code = self.value.format_code();
                visitor.visit_u8(code)
            }
            EnumType::Descriptor => {
                let code = self.value.format_code();
                visitor.visit_u8(code)
            },
            // EnumType::AmqpError => {
            //     self.deserialize_newtype_struct(SYMBOL, visitor)
            // },
            // EnumType::ConnectionError => {
            //     self.deserialize_newtype_struct(SYMBOL, visitor)
            // },
            // EnumType::SessionError => {
            //     self.deserialize_newtype_struct(SYMBOL, visitor)
            // },
            // EnumType::LinkError => {
            //     self.deserialize_newtype_struct(SYMBOL, visitor)
            // },
            EnumType::None => match self.value {
                Value::Uint(v) => visitor.visit_u32(v),
                _ => Err(Error::InvalidValue),
            },
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        // self.deserialize_any(visitor)
        visitor.visit_unit()
    }
}

pub struct SeqAccess {
    iter: <Vec<Value> as IntoIterator>::IntoIter,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(elem) => seed.deserialize(Deserializer::new(elem)).map(Some),
            None => Ok(None),
        }
    }
}

pub struct MapAccess {
    iter: <BTreeMap<Value, Value> as IntoIterator>::IntoIter,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, _seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn next_value_seed<V>(&mut self, _seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                let key = kseed.deserialize(Deserializer::new(k))?;
                let value = vseed.deserialize(Deserializer::new(v))?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }
}

pub struct VariantAccess {
    iter: <Vec<Value> as IntoIterator>::IntoIter,
}

impl<'de> de::EnumAccess<'de> for VariantAccess {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => {
                let val = seed.deserialize(Deserializer::new(value))?;
                Ok((val, self))
            }
            None => Err(Error::Message("Expecting a Value".to_string())),
        }
    }
}

impl<'de> de::VariantAccess<'de> for VariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(Deserializer::new(value)),
            None => Err(Error::Message("Expecting a value".to_string())),
        }
    }

    fn tuple_variant<V>(mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.iter.next() {
            Some(value) => match &value {
                Value::List(_) => {
                    de::Deserializer::deserialize_tuple(Deserializer::new(value), len, visitor)
                }
                _ => Err(Error::InvalidValue),
            },
            None => Err(Error::Message("Expecting Value::List".to_string())),
        }
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.tuple_variant(fields.len(), visitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::de;

    use crate::value::Value;

    use super::from_value;

    fn assert_eq_from_value_vs_expected<T>(value: Value, expected: T)
    where
        T: de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        let deserialized: T = from_value(value).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[test]
    fn test_bool_from_value() {
        let value = Value::Bool(true);
        let expected = true;
        assert_eq_from_value_vs_expected(value, expected);

        let value = Value::Bool(false);
        let expected = false;
        assert_eq_from_value_vs_expected(value, expected);
    }

    #[test]
    fn test_serialize_value_unit_variant() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum Foo {
            A,
            B,
            C,
        }

        let expected = Foo::B;
        let val = Value::Uint(1);
        assert_eq_from_value_vs_expected(val, expected);
    }

    #[test]
    fn test_serialize_value_newtype_variant() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum Foo {
            A(String),
            B(u64),
        }

        let expected = Foo::B(13);
        let val = Value::List(vec![Value::Uint(1), Value::Ulong(13)]);
        assert_eq_from_value_vs_expected(val, expected);
    }

    #[test]
    fn test_serialize_value_tuple_variant() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum Foo {
            A(u32, bool),
            B(i32, String),
        }
        let expected = Foo::B(13, "amqp".to_string());
        let val = Value::List(vec![
            Value::Uint(1),
            Value::List(vec![Value::Int(13), Value::String(String::from("amqp"))]),
        ]);
        assert_eq_from_value_vs_expected(val, expected);
    }

    #[test]
    fn test_serialize_value_struct_variant() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        enum Foo {
            A { num: u32, is_a: bool },
            B { signed_num: i32, amqp: String },
        }

        let expected = Foo::A {
            num: 13,
            is_a: true,
        };
        let val = Value::List(vec![
            Value::Uint(0),
            Value::List(vec![Value::Uint(13), Value::Bool(true)]),
        ]);
        assert_eq_from_value_vs_expected(val, expected);
    }
}
