//! AMF0 value types.

use std::borrow::Cow;
use std::collections::HashMap;

use scuffle_bytes_util::StringCow;
use serde::ser::{SerializeMap, SerializeSeq};

/// Represents any AMF0 object.
pub type Amf0Object<'a> = HashMap<StringCow<'a>, Amf0Value<'a>>;
/// Represents any AMF0 array.
pub type Amf0Array<'a> = Cow<'a, [Amf0Value<'a>]>;

/// Represents any AMF0 value.
#[derive(Debug, PartialEq, Clone)]
pub enum Amf0Value<'a> {
    /// AMF0 Number.
    Number(f64),
    /// AMF0 Boolean.
    Boolean(bool),
    /// AMF0 String.
    String(StringCow<'a>),
    /// AMF0 Object.
    Object(Amf0Object<'a>),
    /// AMF0 Null.
    Null,
    /// AMF0 Array.
    Array(Amf0Array<'a>),
}

impl<'a> Amf0Value<'a> {
    /// Converts this AMF0 value into an owned version (static lifetime).
    pub fn into_owned(self) -> Amf0Value<'static> {
        match self {
            Amf0Value::Number(v) => Amf0Value::Number(v),
            Amf0Value::Boolean(v) => Amf0Value::Boolean(v),
            Amf0Value::String(v) => Amf0Value::String(v.into_owned()),
            Amf0Value::Object(v) => {
                Amf0Value::Object(v.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect())
            }
            Amf0Value::Null => Amf0Value::Null,
            Amf0Value::Array(v) => Amf0Value::Array(v.into_owned().into_iter().map(|v| v.into_owned()).collect()),
        }
    }
}

impl From<f64> for Amf0Value<'_> {
    fn from(value: f64) -> Self {
        Amf0Value::Number(value)
    }
}

impl From<bool> for Amf0Value<'_> {
    fn from(value: bool) -> Self {
        Amf0Value::Boolean(value)
    }
}

impl<'a> From<StringCow<'a>> for Amf0Value<'a> {
    fn from(value: StringCow<'a>) -> Self {
        Amf0Value::String(value)
    }
}

// object
impl<'a> From<Amf0Object<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Object<'a>) -> Self {
        Amf0Value::Object(value)
    }
}

// owned array
impl<'a> From<Vec<Amf0Value<'a>>> for Amf0Value<'a> {
    fn from(value: Vec<Amf0Value<'a>>) -> Self {
        Amf0Value::Array(Cow::Owned(value))
    }
}

// borrowed array
impl<'a> From<&'a [Amf0Value<'a>]> for Amf0Value<'a> {
    fn from(value: &'a [Amf0Value<'a>]) -> Self {
        Amf0Value::Array(Cow::Borrowed(value))
    }
}

// cow array
impl<'a> From<Amf0Array<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Array<'a>) -> Self {
        Amf0Value::Array(value)
    }
}

impl<'a> FromIterator<Amf0Value<'a>> for Amf0Value<'a> {
    fn from_iter<T: IntoIterator<Item = Amf0Value<'a>>>(iter: T) -> Self {
        Amf0Value::Array(Cow::Owned(iter.into_iter().collect()))
    }
}

impl<'de> serde::de::Deserialize<'de> for Amf0Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Amf0ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for Amf0ValueVisitor {
            type Value = Amf0Value<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an AMF0 value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Number(v))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_owned())
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }

                Ok(vec.into())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut object = HashMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    object.insert(key, value);
                }

                Ok(object.into())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let array = v.iter().map(|b| Amf0Value::Number(*b as f64)).collect();
                Ok(Amf0Value::Array(array))
            }
        }

        deserializer.deserialize_any(Amf0ValueVisitor)
    }
}

impl<'a> serde::ser::Serialize for Amf0Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Amf0Value::Number(v) => serializer.serialize_f64(*v),
            Amf0Value::Boolean(v) => serializer.serialize_bool(*v),
            Amf0Value::String(v) => v.serialize(serializer),
            Amf0Value::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;

                for (key, value) in v.iter() {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }
            Amf0Value::Null => serializer.serialize_none(),
            Amf0Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;

                for value in v.iter() {
                    seq.serialize_element(value)?;
                }

                seq.end()
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::borrow::Cow;
    use std::fmt::{Debug, Display};

    use bytes::Bytes;
    use scuffle_bytes_util::StringCow;
    use serde::Deserialize;
    use serde::de::{IntoDeserializer, MapAccess, SeqAccess};

    use super::Amf0Value;
    use crate::{Amf0Array, Amf0Error, Amf0Marker, Amf0Object, from_bytes, to_bytes};

    #[test]
    fn from() {
        let value: Amf0Value = 1.0.into();
        assert_eq!(value, Amf0Value::Number(1.0));

        let value: Amf0Value = true.into();
        assert_eq!(value, Amf0Value::Boolean(true));

        let value: Amf0Value = StringCow::from("abc").into();
        assert_eq!(value, Amf0Value::String("abc".into()));

        let object: Amf0Object = [("a".into(), Amf0Value::Boolean(true))].into_iter().collect();
        let value: Amf0Value = object.clone().into();
        assert_eq!(value, Amf0Value::Object(object));

        let array: Vec<Amf0Value> = vec![Amf0Value::Boolean(true)];
        let value: Amf0Value = array.clone().into();
        assert_eq!(value, Amf0Value::Array(Cow::Owned(array)));

        let array: &[Amf0Value] = &[Amf0Value::Boolean(true)];
        let value: Amf0Value = array.into();
        assert_eq!(value, Amf0Value::Array(Cow::Borrowed(array)));

        let array: Amf0Array = Cow::Borrowed(&[Amf0Value::Boolean(true)]);
        let value: Amf0Value = array.clone().into();
        assert_eq!(value, Amf0Value::Array(array));

        let iter = std::iter::once(Amf0Value::Boolean(true));
        let value: Amf0Value = iter.collect();
        assert_eq!(value, Amf0Value::Array(Cow::Owned(vec![Amf0Value::Boolean(true)])));
    }

    #[test]
    fn unsupported_marker() {
        let bytes = [Amf0Marker::MovieClipMarker as u8];

        let err = from_bytes::<Amf0Value>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(err, Amf0Error::UnsupportedMarker(Amf0Marker::MovieClipMarker)));
    }

    #[test]
    fn string() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 3, // length
            b'a', b'b', b'c',
        ];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::String("abc".into()));
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 0];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::Boolean(false));
    }

    #[test]
    fn object() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1,
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect())
        );
    }

    #[test]
    fn array() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 1,
            Amf0Marker::Boolean as u8,
            1,
        ];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::Array(Cow::Borrowed(&[Amf0Value::Boolean(true)])));

        let serialized = to_bytes(&value).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn null() {
        let bytes = [Amf0Marker::Null as u8];

        let value: Amf0Value = from_bytes(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Amf0Value::Null);

        let serialized = to_bytes(&value).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn into_owned() {
        let value = Amf0Value::Number(1.0);
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Boolean(true);
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::String("abc".into());
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect());
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value,);

        let value = Amf0Value::Null;
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Array(Cow::Borrowed(&[Amf0Value::Boolean(true)]));
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);
    }

    #[test]
    fn deserialize() {
        #[derive(Debug)]
        struct TestError;

        impl Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Test error")
            }
        }

        impl std::error::Error for TestError {}

        impl serde::de::Error for TestError {
            fn custom<T: std::fmt::Display>(msg: T) -> Self {
                assert_eq!(msg.to_string(), "invalid type: Option value, expected a byte slice");
                Self
            }
        }

        enum Mode {
            Bool,
            I64,
            U64,
            F64,
            Str,
            String,
            BorrowedStr,
            Unit,
            None,
            Some,
            Seq,
            Map,
            Bytes,
            End,
        }

        struct TestDeserializer {
            mode: Mode,
        }

        impl<'de> SeqAccess<'de> for TestDeserializer {
            type Error = TestError;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Seq => {
                        self.mode = Mode::End;
                        Ok(Some(seed.deserialize(TestDeserializer { mode: Mode::I64 })?))
                    }
                    Mode::End => Ok(None),
                    _ => Err(TestError),
                }
            }
        }

        impl<'de> MapAccess<'de> for TestDeserializer {
            type Error = TestError;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Map => Ok(Some(seed.deserialize(TestDeserializer { mode: Mode::Str })?)),
                    Mode::End => Ok(None),
                    _ => Err(TestError),
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Map => {
                        self.mode = Mode::End;
                        Ok(seed.deserialize(TestDeserializer { mode: Mode::I64 })?)
                    }
                    _ => Err(TestError),
                }
            }
        }

        impl<'de> serde::Deserializer<'de> for TestDeserializer {
            type Error = TestError;

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
                option unit unit_struct newtype_struct seq tuple tuple_struct
                map struct enum identifier ignored_any
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self.mode {
                    Mode::Bool => visitor.visit_bool(true),
                    Mode::I64 => visitor.visit_i64(1),
                    Mode::U64 => visitor.visit_u64(1),
                    Mode::F64 => visitor.visit_f64(1.0),
                    Mode::Str => visitor.visit_str("hello"),
                    Mode::String => visitor.visit_string("hello".to_owned()),
                    Mode::BorrowedStr => visitor.visit_borrowed_str("hello"),
                    Mode::Unit => visitor.visit_unit(),
                    Mode::None => visitor.visit_none(),
                    Mode::Some => visitor.visit_some(1.into_deserializer()),
                    Mode::Seq => visitor.visit_seq(self),
                    Mode::Map => visitor.visit_map(self),
                    Mode::Bytes => visitor.visit_bytes(b"hello"),
                    Mode::End => unreachable!(),
                }
            }
        }

        fn test_de(mode: Mode, expected: Amf0Value) {
            let deserializer = TestDeserializer { mode };
            let deserialized_value: Amf0Value = Amf0Value::deserialize(deserializer).unwrap();
            assert_eq!(deserialized_value, expected);
        }

        test_de(Mode::Bool, Amf0Value::Boolean(true));
        test_de(Mode::I64, Amf0Value::Number(1.0));
        test_de(Mode::U64, Amf0Value::Number(1.0));
        test_de(Mode::F64, Amf0Value::Number(1.0));
        test_de(Mode::Str, Amf0Value::String("hello".into()));
        test_de(Mode::String, Amf0Value::String("hello".into()));
        test_de(Mode::BorrowedStr, Amf0Value::String("hello".into()));
        test_de(Mode::Unit, Amf0Value::Null);
        test_de(Mode::None, Amf0Value::Null);
        test_de(Mode::Some, Amf0Value::Number(1.0));
        test_de(Mode::Seq, Amf0Value::Array(Cow::Owned(vec![Amf0Value::Number(1.0)])));
        test_de(
            Mode::Map,
            Amf0Value::Object([("hello".into(), Amf0Value::Number(1.0))].into_iter().collect()),
        );
        test_de(
            Mode::Bytes,
            Amf0Value::Array(Cow::Owned(vec![
                Amf0Value::Number(104.0),
                Amf0Value::Number(101.0),
                Amf0Value::Number(108.0),
                Amf0Value::Number(108.0),
                Amf0Value::Number(111.0),
            ])),
        );
    }
}
