use serde::de::{value::MapDeserializer, IntoDeserializer, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};
use snafu::ResultExt;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::{error, Error, Result};
use crate::datastore::{deserializer_for_scalar, ScalarDeserializer, KEY_SEPARATOR};

pub fn from_map<'de, S1, S2, T, BH>(map: &'de HashMap<S1, S2, BH>) -> Result<T>
where
    S1: Borrow<str> + Eq + Hash,
    S2: AsRef<str>,
    T: Deserialize<'de>,
    BH: std::hash::BuildHasher,
{
    let de = CompoundDeserializer::new(
        map,
        map.keys().map(|s| s.borrow().to_string()).collect(),
        None,
    );
    trace!("Deserializing keys: {:?}", de.keys);
    T::deserialize(de)
}

/// This is an alternate interface to deserialization that allows deserializing into maps.
///
/// To use this, you need to provide a string prefix, which represents the prefix of the map keys
/// that needs to be stripped away in order to match the map's expected fields.
///
/// For example, if you have `type Services = HashMap<String, Service>` and you have map keys like
/// "services.x.y.z", then you need to strip away the "services" component that represents the
/// map's "name", otherwise we'd think you have a "services" key in the map itself.  (The dot is
/// removed automatically, you don't need to specify it.)
///
/// This isn't necessary for structs because serde knows the struct's name, so we
/// can strip it automatically.
pub fn from_map_with_prefix<'de, S1, S2, T, BH>(
    prefix: Option<String>,
    map: &'de HashMap<S1, S2, BH>,
) -> Result<T>
where
    S1: Borrow<str> + Eq + Hash,
    S2: AsRef<str>,
    T: Deserialize<'de>,
    BH: std::hash::BuildHasher,
{
    let de = CompoundDeserializer::new(
        map,
        map.keys().map(|s| s.borrow().to_string()).collect(),
        prefix,
    );
    trace!(
        "Deserializing keys with prefix {:?}: {:?}",
        de.path,
        de.keys
    );
    T::deserialize(de)
}

/// ValueDeserializer is what interfaces with serde's MapDeserializer, which expects to receive a
/// key name and a deserializer for it on each iteration, i.e. for each field.  Based on whether
/// the key name has a dot, we know if we need to recurse again or just deserialize a final value,
/// which we represent as the two arms of the enum.
enum ValueDeserializer<'de, S1, S2, BH> {
    Scalar(ScalarDeserializer<'de>),
    Compound(CompoundDeserializer<'de, S1, S2, BH>),
}

impl<'de, S1, S2, BH> serde::de::Deserializer<'de> for ValueDeserializer<'de, S1, S2, BH>
where
    S1: Borrow<str> + Eq + Hash,
    S2: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Error = Error;

    /// Here we either pass off a scalar value to actually turn into a Rust data type, or
    /// recursively call our CompoundDeserializer to handle nested structure.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueDeserializer::Scalar(mut scalar_deserializer) => {
                trace!("Handing off to scalar deserializer for deserialize_any");
                scalar_deserializer
                    .deserialize_any(visitor)
                    .context(error::DeserializeScalar)
            }
            ValueDeserializer::Compound(compound_deserializer) => {
                compound_deserializer.deserialize_map(visitor)
            }
        }
    }

    /// Here we deserialize values into Some(value) for any Option fields to represent that
    /// yes, we do indeed have the data.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueDeserializer::Scalar(mut scalar_deserializer) => {
                trace!("Handing off to scalar deserializer for deserialize_option");
                scalar_deserializer
                    .deserialize_option(visitor)
                    .context(error::DeserializeScalar)
            }
            ValueDeserializer::Compound(compound_deserializer) => {
                compound_deserializer.deserialize_option(visitor)
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, S1, S2, BH> IntoDeserializer<'de, Error> for ValueDeserializer<'de, S1, S2, BH>
where
    S1: Borrow<str> + Eq + Hash,
    S2: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

/// CompoundDeserializer is our main structure that drives serde's MapDeserializer and stores the
/// state we need to understand the recursive structure of the output.
struct CompoundDeserializer<'de, S1, S2, BH> {
    /// A reference to the input data we're deserializing.
    map: &'de HashMap<S1, S2, BH>,
    /// The keys that we need to consider in this iteration.  Starts out the same as the keys
    /// of the input map, but on recursive calls it's only the keys that are relevant to the
    /// sub-struct we're handling, with the duplicated prefix (the 'path') removed.
    keys: HashSet<String>,
    /// The path tells us where we are in our recursive structures.
    path: Option<String>,
}

impl<'de, S1, S2, BH> CompoundDeserializer<'de, S1, S2, BH>
where
    BH: std::hash::BuildHasher,
{
    fn new(
        map: &'de HashMap<S1, S2, BH>,
        keys: HashSet<String>,
        path: Option<String>,
    ) -> CompoundDeserializer<'de, S1, S2, BH> {
        CompoundDeserializer { map, keys, path }
    }
}

fn bad_root<T>() -> Result<T> {
    error::BadRoot.fail()
}

impl<'de, S1, S2, BH> serde::de::Deserializer<'de> for CompoundDeserializer<'de, S1, S2, BH>
where
    S1: Borrow<str> + Eq + Hash,
    S2: AsRef<str>,
    BH: std::hash::BuildHasher,
{
    type Error = Error;

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // On the first iteraction for a struct, we won't have a prefix yet, unless the user called
        // from_map_with_prefix and specified it.  We can make the prefix from the struct name.
        // (Recursive calls will have a path but no name, because we always treat nested structures
        // as maps, because we don't need any nested struct names and it lets us use the nice
        // MapDeserializer.)
        if !name.is_empty() {
            trace!("Path before name check: {:?}", self.path);
            self.path
                .get_or_insert_with(|| name.to_lowercase().to_owned());
            trace!("Path after name check: {:?}", self.path);
        }

        if let Some(ref path) = self.path {
            // Remove the known path from the beginning of the keys. serde doesn't care about the
            // name of the top-level struct, just the fields inside, so we have to remove it before
            // handing it to the MapDeserializer.  (Our real customer is the one specifying the
            // dotted keys, and we always use the struct name there for clarity.)
            trace!("Keys before path strip: {:?}", self.keys);
            self.keys = self
                .keys
                .iter()
                .map(|k| k.replacen(&format!("{}.", path.to_lowercase()), "", 1))
                .collect();
            trace!("Keys after path strip: {:?}", self.keys);
        }

        // We have to track which structs we've already handled and skip over them.  This is
        // because we could get keys like "a.b.c" and "a.b.d", so we'll see that "a" prefix
        // twice at the top level, but by the time we see the second one we've already recursed
        // and handled all of "a" from the first one.
        let mut structs_done = HashSet::new();

        // As mentioned above, MapDeserializer does a lot of nice work for us.  We just need to
        // give it an iterator that yields (key, deserializer) pairs.  The nested deserializers
        // have the appropriate 'path' and a subset of 'keys' so they can do their job.
        visitor.visit_map(MapDeserializer::new(self.keys.iter().filter_map(|key| {
            let struct_name = key.split('.').next().unwrap();
            trace!("Visiting key '{}', struct name '{}'", key, struct_name);

            // If we have a path, add a separator, otherwise start with an empty string.
            let old_path = self
                .path
                .as_ref()
                .map(|s| s.clone() + KEY_SEPARATOR)
                .unwrap_or_default();
            trace!("Old path: {}", &old_path);
            let new_path = old_path + struct_name;
            trace!("New path: {}", &new_path);

            if key.contains('.') {
                if structs_done.contains(&struct_name) {
                    // We've handled this structure with a recursive call, so we're done.
                    trace!("Already handled struct '{}', skipping", struct_name);
                    None
                } else {
                    // Otherwise, mark it, and recurse.
                    structs_done.insert(struct_name);

                    // Subset the keys so the recursive call knows what it needs to handle -
                    // only things starting with the new path.
                    let dotted_prefix = struct_name.to_owned() + KEY_SEPARATOR;
                    let keys = self
                        .keys
                        .iter()
                        .filter(|new_key| new_key.starts_with(&dotted_prefix))
                        .map(|new_key| new_key[dotted_prefix.len()..].to_owned())
                        .collect();

                    // And here's what MapDeserializer expects, the key and deserializer for it
                    trace!(
                        "Recursing for struct '{}' with keys: {:?}",
                        struct_name,
                        keys
                    );
                    Some((
                        struct_name.to_owned(),
                        ValueDeserializer::Compound(CompoundDeserializer::new(
                            self.map,
                            keys,
                            Some(new_path),
                        )),
                    ))
                }
            } else {
                // No dot, so we have a scalar; hand the data to a scalar deserializer.
                trace!(
                    "Key '{}' is scalar, getting '{}' from input to deserialize",
                    key,
                    new_path
                );
                let val = self.map.get(&new_path)?;
                Some((
                    key.to_owned(),
                    ValueDeserializer::Scalar(deserializer_for_scalar(val.as_ref())),
                ))
            }
        })))
    }

    /// We use deserialize_map for all maps, including top-level maps, but to allow top-level maps
    /// we require that the user specified a prefix for us using from_map_with_prefix.
    ///
    /// We also use it for structs below the top level, because you don't need a name once you're
    /// recursing - you'd always be pointed to by a struct field or map key whose name we use.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.path {
            Some(_) => self.deserialize_struct("", &[], visitor),
            None => bad_root(),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    /// Scalar types, and compound types we can't use at the root, are forwarded here to be
    /// rejected.  (Compound types need to have a name to serve at the root level.)
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        bad_root()
    }

    // This gives us the rest of the implementations needed to compile, and forwards them to the
    // function above that will reject them.
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct enum identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    use super::{from_map, from_map_with_prefix};
    use crate::datastore::deserialization::Error;

    use maplit::hashmap;
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        id: Option<u64>,
        name: String,
        list: Vec<u8>,
        nested: B,
        map: HashMap<String, String>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct B {
        a: String,
        b: bool,
        c: Option<i64>,
        d: Option<C>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct C {
        boolean: bool,
    }

    #[test]
    fn basic_struct_works() {
        let c: C = from_map(&hashmap! {
            "c.boolean".to_string() => "true".to_string(),
        })
        .unwrap();
        assert_eq!(c, C { boolean: true });
    }

    #[test]
    fn deep_struct_works() {
        let a: A = from_map(&hashmap! {
            "a.id".to_string() => "1".to_string(),
            "a.name".to_string() => "\"it's my name\"".to_string(),
            "a.list".to_string() => "[1,2, 3, 4]".to_string(),
            "a.map.a".to_string() => "\"answer is always map\"".to_string(),
            "a.nested.a".to_string() => "\"quite nested\"".to_string(),
            "a.nested.b".to_string() => "false".to_string(),
            "a.nested.c".to_string() => "null".to_string(),
            "a.nested.d.boolean".to_string() => "true".to_string(),
        })
        .unwrap();
        assert_eq!(
            a,
            A {
                id: Some(1),
                name: "it's my name".to_string(),
                list: vec![1, 2, 3, 4],
                map: hashmap! {
                    "a".to_string() => "answer is always map".to_string(),
                },
                nested: B {
                    a: "quite nested".to_string(),
                    b: false,
                    c: None,
                    d: Some(C { boolean: true })
                }
            }
        );
    }

    #[test]
    fn map_doesnt_work_at_root() {
        let a: Result<HashMap<String, String>, Error> = from_map(&hashmap! {
            "a".to_string() => "\"it's a\"".to_string(),
            "b".to_string() => "\"it's b\"".to_string(),
        });
        a.unwrap_err();
    }

    #[test]
    fn map_works_at_root_with_prefix() {
        let map = &hashmap! {
            "x.boolean".to_string() => "true".to_string()
        };
        let x: HashMap<String, bool> = from_map_with_prefix(Some("x".to_string()), map).unwrap();
        assert_eq!(
            x,
            hashmap! {
                "boolean".to_string() => true,
            }
        );
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Bad {
        id: u64,
    }

    #[test]
    fn disallowed_data_type() {
        let bad: Result<Bad, Error> = from_map(&hashmap! {
            "id".to_string() => "42".to_string(),
        });
        bad.unwrap_err();
    }
}
