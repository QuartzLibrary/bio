use std::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use serde::{
    Deserializer,
    de::{EnumAccess, SeqAccess, Unexpected, VariantAccess, Visitor},
};
use serde_with::formats::{CommaSeparator, SemicolonSeparator, SpaceSeparator};

pub use serde_with::formats::Separator;

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
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        struct AccessEnum<'v, E> {
            value: &'v str,
            _marker: PhantomData<E>,
        }
        impl<'de, E> EnumAccess<'de> for AccessEnum<'de, E>
        where
            E: serde::de::Error,
        {
            type Error = E;

            type Variant = UnitAccessVariant<E>;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                let value = seed.deserialize(StringDeserializer::new(self.value))?;
                Ok((
                    value,
                    UnitAccessVariant {
                        _marker: PhantomData,
                    },
                ))
            }
        }
        struct UnitAccessVariant<E> {
            _marker: PhantomData<E>,
        }
        impl<'de, E> VariantAccess<'de> for UnitAccessVariant<E>
        where
            E: serde::de::Error,
        {
            type Error = E;

            fn unit_variant(self) -> Result<(), Self::Error> {
                Ok(())
            }

            fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
            where
                T: serde::de::DeserializeSeed<'de>,
            {
                unreachable!()
            }

            fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                unreachable!()
            }

            fn struct_variant<V>(
                self,
                _fields: &'static [&'static str],
                _visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                unreachable!()
            }
        }
        visitor.visit_enum(AccessEnum {
            value: self.0,
            _marker: PhantomData,
        })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_str(self.0)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        unreachable!()
    }
}

pub struct StringSequenceDeserializer<'de, E>(&'de str, char, PhantomData<E>);
impl<'de, E> StringSequenceDeserializer<'de, E> {
    pub fn new(s: &'de str, sep: char) -> Self {
        Self(s, sep, PhantomData)
    }
}
impl<'de, E> Deserializer<'de> for StringSequenceDeserializer<'de, E>
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
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Self::Error::invalid_value(
            Unexpected::Str(self.0),
            &visitor,
        ))
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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_seq(self)
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
impl<'de, E> SeqAccess<'de> for StringSequenceDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.0.split_once(self.1) {
            Some((value, rest)) => {
                self.0 = rest;
                seed.deserialize(StringDeserializer::new(value)).map(Some)
            }
            None if self.0.is_empty() => Ok(None),
            None => {
                let value = self.0;
                self.0 = "";
                seed.deserialize(StringDeserializer::new(value)).map(Some)
            }
        }
    }
}

pub type SpaceSeparated<T> = Separated<SpaceSeparator, T>;
pub type CommaSeparated<T> = Separated<CommaSeparator, T>;
pub type SemicolonSeparated<T> = Separated<SemicolonSeparator, T>;
pub type PipeSeparated<T> = Separated<PipeSeparator, T>;

/// Adapted from [serde_with::StringWithSeparator].
pub struct Separated<Pattern, T> {
    _marker: PhantomData<(Pattern, T)>,
}
impl<Pattern: Separator, T> Separated<Pattern, T> {
    pub fn serialize<S, A>(v: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        for<'a> &'a T: IntoIterator<Item = &'a A>,
        A: Display,
        // This set of bounds is enough to make the function compile but has inference issues
        // making it unusable at the moment.
        // https://github.com/rust-lang/rust/issues/89196#issuecomment-932024770
        // for<'a> &'a T: IntoIterator,
        // for<'a> <&'a T as IntoIterator>::Item: Display,
    {
        struct DisplayWithSeparator<'a, T, Pattern>(&'a T, PhantomData<Pattern>);

        impl<'a, T, Pattern> Display for DisplayWithSeparator<'a, T, Pattern>
        where
            Pattern: Separator,
            &'a T: IntoIterator,
            <&'a T as IntoIterator>::Item: Display,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut iter = self.0.into_iter();

                if let Some(first) = iter.next() {
                    first.fmt(f)?;
                }
                for elem in iter {
                    f.write_str(Pattern::separator())?;
                    elem.fmt(f)?;
                }

                Ok(())
            }
        }

        serializer.collect_str(&DisplayWithSeparator::<T, Pattern>(v, PhantomData))
    }

    pub fn deserialize<'de, D, A>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: FromIterator<A>,
        A: FromStr,
        A::Err: Display,
    {
        struct Helper<Pattern, T, A>(PhantomData<(Pattern, T, A)>);

        impl<Pattern, T, A> Visitor<'_> for Helper<Pattern, T, A>
        where
            Pattern: Separator,
            T: FromIterator<A>,
            A: FromStr,
            A::Err: Display,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if value.is_empty() {
                    Ok(None.into_iter().collect())
                } else {
                    value
                        .split(Pattern::separator())
                        .map(FromStr::from_str)
                        .collect::<Result<_, _>>()
                        .map_err(serde::de::Error::custom)
                }
            }
        }

        deserializer.deserialize_str(Helper::<Pattern, T, _>(PhantomData))
    }
}
pub struct PipeSeparator;
impl Separator for PipeSeparator {
    fn separator() -> &'static str {
        "|"
    }
}

/// Serializes an iterator as a [Vec].
/// Useful for example when serializing a map with non-string keys to json.
pub mod as_vec {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S, I>(iter: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        I: IntoIterator + Clone,
        <I as IntoIterator>::Item: Serialize,
    {
        let vec: Vec<_> = iter.clone().into_iter().collect();
        vec.serialize(serializer)
    }
    pub fn deserialize<'de, D, I>(deserializer: D) -> Result<I, D::Error>
    where
        D: serde::Deserializer<'de>,
        I: IntoIterator + FromIterator<<I as IntoIterator>::Item>,
        <I as IntoIterator>::Item: Deserialize<'de>,
    {
        let vec = Vec::<_>::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

pub mod arc_str {
    use std::{
        borrow::Cow,
        cell::RefCell,
        collections::hash_map::Entry,
        ops::Deref,
        sync::{Arc, LazyLock, Weak},
    };

    thread_local! {
        static STATE: LazyLock<ahash::RandomState> = LazyLock::new(ahash::RandomState::new);
        static POOL: RefCell<nohash_hasher::IntMap<u64, Vec<Weak<str>>>> = RefCell::new(nohash_hasher::IntMap::default());
    }

    pub fn clean_string_pool() {
        POOL.with(|pool| {
            pool.borrow_mut().retain(|_, v| {
                v.retain(|w| w.strong_count() > 0);
                !v.is_empty()
            });
        });
    }

    /// A string that is cheap to clone and avoids needing to enable the [serde]
    /// feature for reference counted pointers.
    ///
    /// This is unaffected by one of the problems [serde] is protecting against (cycles),
    /// and avoids memory blow-ups by keeping a thread local weak reference pool
    /// if `POOL` is true.
    ///
    /// This is not perfect because it has a runtime cost and old [Weak] references are
    /// leaked by default.
    ///
    /// Since this is meant to be used for values which are enumerable, it should be fine.
    /// We'll see in practice if it becomes problematic.
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SerdeArcStr<const POOL: bool = true>(Arc<str>);
    impl<const POOL: bool> Default for SerdeArcStr<POOL> {
        fn default() -> Self {
            Self::new("")
        }
    }
    impl<const POOL: bool> Deref for SerdeArcStr<POOL> {
        type Target = str;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<const POOL: bool> SerdeArcStr<POOL> {
        pub fn new(s: impl AsRef<str> + Into<Arc<str>>) -> Self {
            if POOL { Self(new(s)) } else { Self(s.into()) }
        }
    }

    impl<const POOL: bool> serde::Serialize for SerdeArcStr<POOL> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.0)
        }
    }
    impl<'de, const POOL: bool> serde::Deserialize<'de> for SerdeArcStr<POOL> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            // Avoids a needless String allocation
            let s: Cow<'de, str> = serde::Deserialize::deserialize(deserializer)?;
            Ok(Self::new(s))
        }
    }

    fn hash_str(s: &str) -> u64 {
        STATE.with(|state| state.hash_one(s))
    }
    fn new(s: impl AsRef<str> + Into<Arc<str>>) -> Arc<str> {
        POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            let value = match pool.entry(hash_str(s.as_ref())) {
                Entry::Occupied(mut bucket) => {
                    bucket.get_mut().retain(|w| w.strong_count() > 0);

                    if let Some(found) = bucket
                        .get()
                        .iter()
                        .filter_map(|w| w.upgrade())
                        .find(|a| &**a == s.as_ref())
                    {
                        return found;
                    }

                    bucket.into_mut()
                }
                Entry::Vacant(e) => e.insert(vec![]),
            };
            let arc: Arc<str> = s.into();
            value.push(Arc::downgrade(&arc));
            #[cfg(debug_assertions)]
            if value.len() % 10 == 0 {
                log::warn!("{} collisions in serde arc string pool", value.len());
            }
            #[cfg(debug_assertions)]
            if pool.len() % 1_000 == 0 {
                log::warn!("{} unique hashes in serde arc string pool", pool.len());
            }
            arc
        })
    }
}
