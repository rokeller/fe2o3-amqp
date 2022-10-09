use std::{fmt::Display, marker::PhantomData};

use serde::{
    de::{self, VariantAccess},
    ser, Serialize,
};
use serde_amqp::{primitives::Binary, Value};

use crate::messaging::{
    AmqpSequence, AmqpValue, Data, DeserializableBody, FromDeserializableBody, FromEmptyBody,
    IntoSerializableBody, SerializableBody, __private::BodySection,
};

use serde_amqp::extensions::TransparentVec;

/// The body consists of one of the following three choices: one or more data sections, one or more
/// amqp-sequence sections, or a single amqp-value section.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body<T> {
    /// An amqp-value section contains a single AMQP value
    Value(AmqpValue<T>),

    /// More than one data section
    ///
    /// Added since `"0.6.0"`
    Data(TransparentVec<Data>),

    /// More than one sequence section
    ///
    /// Added since `"0.6.0"`
    Sequence(TransparentVec<AmqpSequence<T>>),

    /// There is no body section at all
    ///
    /// The core specification states that **at least one** body section should be present in
    /// the message. However, this is not the way `proton` is implemented, and according to
    /// [PROTON-2574](https://issues.apache.org/jira/browse/PROTON-2574), the wording in the
    /// core specification was an unintended.
    Empty,
}

impl<T> Body<T> {
    /// Whether the body section is a [`Data`]
    pub fn is_data(&self) -> bool {
        matches!(self, Body::Data(_))
    }

    /// Whether the body section is a [`AmqpSequence`]
    pub fn is_sequence(&self) -> bool {
        matches!(self, Body::Sequence(_))
    }

    /// Whether the body section is a [`AmqpValue`]
    pub fn is_value(&self) -> bool {
        matches!(self, Body::Value(_))
    }

    /// Whether the body section is `Nothing`
    #[deprecated(since = "0.5.2", note = "Please use is_empty() instead")]
    pub fn is_nothing(&self) -> bool {
        matches!(self, Body::Empty)
    }

    /// Whether the body section is `Nothing`
    pub fn is_empty(&self) -> bool {
        matches!(self, Body::Empty)
    }

    /// Consume the delivery into the body if the body is an [`AmqpValue`].
    /// An error will be returned if otherwise
    pub fn try_into_value(self) -> Result<T, Self> {
        match self {
            Body::Value(AmqpValue(value)) => Ok(value),
            _ => Err(self),
        }
    }

    /// Consume the delivery into the body if the body is one or more [`Data`].
    /// An error will be returned if otherwise
    pub fn try_into_data(self) -> Result<impl Iterator<Item = Binary>, Self> {
        match self {
            Body::Data(batch) => Ok(batch.into_iter().map(|data| data.0)),
            _ => Err(self),
        }
    }

    /// Consume the delivery into the body if the body is one or more [`AmqpSequence`].
    /// An error will be returned if otherwise
    pub fn try_into_sequence(self) -> Result<impl Iterator<Item = Vec<T>>, Self> {
        match self {
            Body::Sequence(batch) => Ok(batch.into_iter().map(|seq| seq.0)),
            _ => Err(self),
        }
    }

    /// Get a reference to the delivery body if the body is an [`AmqpValue`].
    /// An error will be returned if the body isnot an [`AmqpValue`]
    pub fn try_as_value(&self) -> Result<&T, &Self> {
        match self {
            Body::Value(AmqpValue(value)) => Ok(value),
            _ => Err(self),
        }
    }

    /// Get a reference to the delivery body if the body is one or more [`Data`].
    /// An error will be returned otherwise
    pub fn try_as_data(&self) -> Result<impl Iterator<Item = &Binary>, &Self> {
        match self {
            Body::Data(batch) => Ok(batch.iter().map(|data| &data.0)),
            _ => Err(self),
        }
    }

    /// Get a reference to the delivery body if the body is one or more [`AmqpSequence`].
    /// An error will be returned otherwise
    pub fn try_as_sequence(&self) -> Result<impl Iterator<Item = &Vec<T>>, &Self> {
        match self {
            Body::Sequence(batch) => Ok(batch.iter().map(|seq| &seq.0)),
            _ => Err(self),
        }
    }
}

impl<T> Display for Body<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Body::Value(val) => write!(f, "{}", val),
            Body::Data(_) => write!(f, "Data"),
            Body::Sequence(_) => write!(f, "Sequence"),
            Body::Empty => write!(f, "Nothing"),
        }
    }
}

impl<T: Serialize> From<T> for Body<T> {
    fn from(value: T) -> Self {
        Self::Value(AmqpValue(value))
    }
}

impl<T: Serialize + Clone, const N: usize> From<[T; N]> for Body<T> {
    fn from(values: [T; N]) -> Self {
        Self::Sequence(TransparentVec::new(vec![AmqpSequence(values.to_vec())]))
    }
}

impl<T> From<AmqpValue<T>> for Body<T> {
    fn from(value: AmqpValue<T>) -> Self {
        Self::Value(value)
    }
}

impl<T> From<AmqpSequence<T>> for Body<T> {
    fn from(val: AmqpSequence<T>) -> Self {
        Self::Sequence(TransparentVec::new(vec![val]))
    }
}

impl From<Data> for Body<Value> {
    fn from(val: Data) -> Self {
        Self::Data(TransparentVec::new(vec![val]))
    }
}

impl<T: Serialize> ser::Serialize for Body<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Body::Data(data) => data.serialize(serializer),
            Body::Sequence(seq) => seq.serialize(serializer),
            Body::Value(val) => val.serialize(serializer),
            Body::Empty => AmqpValue(()).serialize(serializer),
        }
    }
}

struct FieldVisitor {}

#[derive(Debug)]
enum Field {
    Data,
    Sequence,
    Value,
}

impl<'de> de::Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Body variant. One of Vec<Data>, Vec<AmqpSequence>, AmqpValue")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            "amqp:data:binary" => Ok(Field::Data),
            "amqp:amqp-sequence:list" => Ok(Field::Sequence),
            "amqp:amqp-value:*" => Ok(Field::Value),
            _ => Err(de::Error::custom("Invalid descriptor code")),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            0x0000_0000_0000_0075 => Ok(Field::Data),
            0x0000_0000_0000_0076 => Ok(Field::Sequence),
            0x0000_0000_0000_0077 => Ok(Field::Value),
            _ => Err(de::Error::custom("Invalid descriptor code")),
        }
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

struct Visitor<T> {
    marker: PhantomData<T>,
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: de::Deserialize<'de>,
{
    type Value = Body<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("enum Body")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        let (val, variant) = data.variant()?;

        match val {
            Field::Data => {
                let data: TransparentVec<Data> = variant.newtype_variant()?;
                Ok(Body::Data(data))
            }
            Field::Sequence => {
                let sequence: TransparentVec<AmqpSequence<_>> = variant.newtype_variant()?;
                Ok(Body::Sequence(sequence))
            }
            Field::Value => {
                let value = variant.newtype_variant()?;
                Ok(Body::Value(value))
            }
        }
    }
}

impl<'de, T> de::Deserialize<'de> for Body<T>
where
    T: de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_enum(
            serde_amqp::__constants::UNTAGGED_ENUM,
            &["Data", "Sequence", "Value"],
            Visitor {
                marker: PhantomData,
            },
        )
    }
}

impl<T> BodySection for Body<T> {}

impl<T> SerializableBody for Body<T> where T: ser::Serialize {}

impl<'de, T> DeserializableBody<'de> for Body<T> where T: de::Deserialize<'de> {}

impl<T> IntoSerializableBody for Body<T>
where
    T: ser::Serialize,
{
    type SerializableBody = Self;

    fn into_serializable_body(self) -> Self::SerializableBody {
        self
    }
}

impl<'de, T> FromDeserializableBody<'de> for Body<T>
where
    T: de::Deserialize<'de>,
{
    type DeserializableBody = Self;

    fn from_deserializable_body(deserializable: Self::DeserializableBody) -> Self {
        deserializable
    }
}

impl<T> FromEmptyBody for Body<T> {
    type Error = serde_amqp::Error;

    fn from_empty_body() -> Result<Self, Self::Error> {
        Ok(Self::Empty)
    }
}
