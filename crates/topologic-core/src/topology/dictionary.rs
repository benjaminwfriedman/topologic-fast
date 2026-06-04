//! Dictionary for storing arbitrary data on topology elements

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// A dictionary for storing arbitrary key-value data on topology elements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dictionary {
    data: HashMap<String, DictionaryValue>,
}

/// Value types supported by the dictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DictionaryValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<DictionaryValue>),
    Dict(HashMap<String, DictionaryValue>),
    Null,
}

impl Dictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a dictionary from key-value pairs
    pub fn from_pairs(pairs: Vec<(&str, DictionaryValue)>) -> Self {
        let mut dict = Self::new();
        for (key, value) in pairs {
            dict.set(key, value);
        }
        dict
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&DictionaryValue> {
        self.data.get(key)
    }

    /// Set a value by key
    pub fn set(&mut self, key: &str, value: DictionaryValue) {
        self.data.insert(key.to_string(), value);
    }

    /// Remove a value by key
    pub fn remove(&mut self, key: &str) -> Option<DictionaryValue> {
        self.data.remove(key)
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &DictionaryValue> {
        self.data.values()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Merge another dictionary into this one
    pub fn merge(&mut self, other: &Dictionary) {
        for (key, value) in &other.data {
            self.data.insert(key.clone(), value.clone());
        }
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }

    /// Create from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let data: HashMap<String, DictionaryValue> = serde_json::from_str(json)?;
        Ok(Self { data })
    }
}

impl DictionaryValue {
    /// Try to get as integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            DictionaryValue::Int(v) => Some(*v),
            DictionaryValue::Float(v) => Some(*v as i64),
            _ => None,
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            DictionaryValue::Float(v) => Some(*v),
            DictionaryValue::Int(v) => Some(*v as f64),
            _ => None,
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            DictionaryValue::String(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DictionaryValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    /// Try to get as list
    pub fn as_list(&self) -> Option<&Vec<DictionaryValue>> {
        match self {
            DictionaryValue::List(v) => Some(v),
            _ => None,
        }
    }

    /// Try to get as dict
    pub fn as_dict(&self) -> Option<&HashMap<String, DictionaryValue>> {
        match self {
            DictionaryValue::Dict(v) => Some(v),
            _ => None,
        }
    }

    /// Check if null
    pub fn is_null(&self) -> bool {
        matches!(self, DictionaryValue::Null)
    }
}

// Convenient From implementations
impl From<i64> for DictionaryValue {
    fn from(v: i64) -> Self {
        DictionaryValue::Int(v)
    }
}

impl From<i32> for DictionaryValue {
    fn from(v: i32) -> Self {
        DictionaryValue::Int(v as i64)
    }
}

impl From<f64> for DictionaryValue {
    fn from(v: f64) -> Self {
        DictionaryValue::Float(v)
    }
}

impl From<f32> for DictionaryValue {
    fn from(v: f32) -> Self {
        DictionaryValue::Float(v as f64)
    }
}

impl From<String> for DictionaryValue {
    fn from(v: String) -> Self {
        DictionaryValue::String(v)
    }
}

impl From<&str> for DictionaryValue {
    fn from(v: &str) -> Self {
        DictionaryValue::String(v.to_string())
    }
}

impl From<bool> for DictionaryValue {
    fn from(v: bool) -> Self {
        DictionaryValue::Bool(v)
    }
}

impl<T: Into<DictionaryValue>> From<Vec<T>> for DictionaryValue {
    fn from(v: Vec<T>) -> Self {
        DictionaryValue::List(v.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_basic() {
        let mut dict = Dictionary::new();
        dict.set("name", "test".into());
        dict.set("value", 42.into());
        dict.set("ratio", 3.14.into());

        assert_eq!(dict.get("name").unwrap().as_string(), Some("test"));
        assert_eq!(dict.get("value").unwrap().as_int(), Some(42));
        assert!((dict.get("ratio").unwrap().as_float().unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_dictionary_json() {
        let mut dict = Dictionary::new();
        dict.set("key", "value".into());

        let json = dict.to_json().unwrap();
        let parsed = Dictionary::from_json(&json).unwrap();

        assert_eq!(
            parsed.get("key").unwrap().as_string(),
            Some("value")
        );
    }
}
