// This section ties together serialization and deserialization of scalar values, so it's in the
// parent module of serialization and deserialization.

/// Concrete error type for scalar ser/de.
pub type ScalarError = serde_json::Error;

/// Serialize a given scalar value to the module-standard serialization format.
pub fn serialize_scalar<S, E>(scalar: &S) -> std::result::Result<String, E>
where
    S: Serialize,
    E: From<ScalarError>,
{
    serde_json::to_string(scalar).map_err(Into::into)
}

/// Deserialize a given scalar value from the module-standard serialization format.
pub fn deserialize_scalar<'de, D, E>(scalar: &'de str) -> std::result::Result<D, E>
where
    D: Deserialize<'de>,
    E: From<ScalarError>,
{
    serde_json::from_str(scalar).map_err(Into::into)
}

/// Serde Deserializer type matching the deserialize_scalar implementation.
type ScalarDeserializer<'de> = serde_json::Deserializer<serde_json::de::StrRead<'de>>;

/// Constructor for ScalarDeserializer.
fn deserializer_for_scalar(scalar: &str) -> ScalarDeserializer {
    serde_json::Deserializer::from_str(scalar)
}

/// Serde generic "Value" type representing a tree of deserialized values.  Should be able to hold
/// anything returned by the deserialization bits above.
pub type Value = serde_json::Value;

#[cfg(test)]
mod test {
    use super::memory::MemoryDataStore;
    use super::{Committed, DataStore, Key, KeyType};
    use maplit::hashmap;

    #[test]
    fn set_keys() {
        let mut m = MemoryDataStore::new();

        let k1 = Key::new(KeyType::Data, "memtest1").unwrap();
        let k2 = Key::new(KeyType::Data, "memtest2").unwrap();
        let v1 = "memvalue1".to_string();
        let v2 = "memvalue2".to_string();
        let data = hashmap!(
            &k1 => &v1,
            &k2 => &v2,
        );

        m.set_keys(&data, Committed::Pending).unwrap();

        assert_eq!(m.get_key(&k1, Committed::Pending).unwrap(), Some(v1));
        assert_eq!(m.get_key(&k2, Committed::Pending).unwrap(), Some(v2));
    }

    #[test]
    fn get_metadata_inheritance() {
        let mut m = MemoryDataStore::new();

        let meta = Key::new(KeyType::Meta, "mymeta").unwrap();
        let parent = Key::new(KeyType::Data, "a").unwrap();
        let grandchild = Key::new(KeyType::Data, "a.b.c").unwrap();

        // Set metadata on parent
        m.set_metadata(&meta, &parent, "value").unwrap();
        // Metadata shows up on grandchild...
        assert_eq!(
            m.get_metadata(&meta, &grandchild).unwrap(),
            Some("value".to_string())
        );
        // ...but only through inheritance, not directly.
        assert_eq!(m.get_metadata_raw(&meta, &grandchild).unwrap(), None);
    }

    #[test]
    fn get_prefix() {
        let mut m = MemoryDataStore::new();
        let data = hashmap!(
            Key::new(KeyType::Data, "x.1").unwrap() => "x1".to_string(),
            Key::new(KeyType::Data, "x.2").unwrap() => "x2".to_string(),
            Key::new(KeyType::Data, "y.3").unwrap() => "y3".to_string(),
        );
        m.set_keys(&data, Committed::Pending).unwrap();

        assert_eq!(
            m.get_prefix("x.", Committed::Pending).unwrap(),
            hashmap!(Key::new(KeyType::Data, "x.1").unwrap() => "x1".to_string(),
                     Key::new(KeyType::Data, "x.2").unwrap() => "x2".to_string())
        );
    }

    #[test]
    fn get_metadata_prefix() {
        let mut m = MemoryDataStore::new();

        // Build some data keys to which we can attach metadata; they don't actually have to be
        // set in the data store.
        let k1 = Key::new(KeyType::Data, "x.1").unwrap();
        let k2 = Key::new(KeyType::Data, "x.2").unwrap();
        let k3 = Key::new(KeyType::Data, "y.3").unwrap();

        // Set some metadata to check
        let mk1 = Key::new(KeyType::Meta, "metatest1").unwrap();
        let mk2 = Key::new(KeyType::Meta, "metatest2").unwrap();
        let mk3 = Key::new(KeyType::Meta, "metatest3").unwrap();
        m.set_metadata(&mk1, &k1, "41").unwrap();
        m.set_metadata(&mk2, &k2, "42").unwrap();
        m.set_metadata(&mk3, &k3, "43").unwrap();

        // Check all metadata
        assert_eq!(
            m.get_metadata_prefix("x.", &None as &Option<&str>).unwrap(),
            hashmap!(k1 => hashmap!(mk1 => "41".to_string()),
                     k2.clone() => hashmap!(mk2.clone() => "42".to_string()))
        );

        // Check metadata matching a given name
        assert_eq!(
            m.get_metadata_prefix("x.", &Some("metatest2")).unwrap(),
            hashmap!(k2 => hashmap!(mk2 => "42".to_string()))
        );
    }
}