use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

/// Convenience functions around a string->string map
// BTreeMap for deterministic serialization
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tags(pub BTreeMap<String, String>);

#[allow(dead_code)]
impl Tags {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.0.get(k)
    }

    pub fn has(&self, k: &str) -> bool {
        self.0.contains_key(k)
    }

    pub fn has_any(&self, keys: Vec<&str>) -> bool {
        keys.into_iter().any(|k| self.0.contains_key(k))
    }

    pub fn is(&self, k: &str, v: &str) -> bool {
        self.0.get(k) == Some(&v.to_string())
    }

    pub fn is_any(&self, k: &str, values: Vec<&str>) -> bool {
        if let Some(v) = self.0.get(k) {
            values.contains(&v.as_ref())
        } else {
            false
        }
    }

    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, k: K, v: V) {
        self.0.insert(k.into(), v.into());
    }

    pub fn remove(&mut self, k: &str) -> Option<String> {
        self.0.remove(k)
    }
}

impl From<HashMap<String, String>> for Tags {
    fn from(map: HashMap<String, String>) -> Self {
        let mut tags = Self::empty();
        for (k, v) in map {
            tags.insert(k, v);
        }
        tags
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn various() {
        let mut tags = Tags::empty();

        tags.insert("key", "value");
        assert!(tags.is("key", "value"));
        assert!(tags.is_any("key", vec!["val1", "val2", "value"]));
    }
}
