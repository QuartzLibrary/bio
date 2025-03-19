use std::marker::PhantomData;

use serde::{de::Unexpected, Deserializer};

/// A simple deserializer that parses a string into the correct value
/// requested by the deserialized type.
pub struct StringDeserializer<'de, E>(&'de str, PhantomData<E>);
impl<'de, E> StringDeserializer<'de, E> {
    pub fn new(s: &'de str) -> Self {
        Self(s, PhantomData)
    }
}
impl<'de, E> Deserializer<'de> for StringDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_bool(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u8(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u16(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u32(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_i64(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u8(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u16(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u32(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_u64(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_f32(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_f64(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.0.parse() {
            Ok(v) => visitor.visit_char(v),
            Err(_) => Err(Self::Error::invalid_value(
                Unexpected::Str(self.0),
                &visitor,
            )),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bytes(self.0.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bytes(self.0.as_bytes())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        // The empty string should be managed either the wrapper or the inner type.
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }
}
