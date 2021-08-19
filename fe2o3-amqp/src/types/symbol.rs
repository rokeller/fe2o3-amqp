use serde::{
    de::{self, Visitor},
    Serialize,
};

pub const SYMBOL: &str = "SYMBOL";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(val: String) -> Self {
        Self(val)
    }
}

impl From<String> for Symbol {
    fn from(val: String) -> Self {
        Self(val)
    }
}

impl From<&str> for Symbol {
    fn from(val: &str) -> Self {
        Self(val.into())
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(SYMBOL, &self.0)
    }
}

struct SymbolVisitor {}

impl<'de> Visitor<'de> for SymbolVisitor {
    type Value = Symbol;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct Symbol")
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val: String = de::Deserialize::deserialize(deserializer)?;
        Ok(Symbol::new(val))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Symbol::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Symbol::from(v.to_string()))
    }
}

impl<'de> de::Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(SYMBOL, SymbolVisitor {})
    }
}

// #[cfg(test)]
// mod tests{
//     use super::*;

//     #[test]
//     fn test_serialize_symbol() {
//         let symbol = Symbol("amqp".into());
//         let expected = [0xa3 as u8, 0x04, 0x61, 0x6d, 0x71, 0x70];
//         let
//     }
// }
