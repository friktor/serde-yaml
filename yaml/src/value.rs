// Copyright 2016 Serde YAML Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::hash::{Hash, Hasher};
use std::mem;

use dtoa;
use linked_hash_map::LinkedHashMap;
use serde::{self, Serialize, Deserialize};
use yaml_rust::Yaml;

use super::{Error, Deserializer, Serializer};

#[derive(Clone, PartialOrd, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Sequence(Sequence),
    Mapping(Mapping),
}

pub type Sequence = Vec<Value>;
pub type Mapping = LinkedHashMap<Value, Value>;

/// Shortcut function to encode a `T` into a YAML `Value`.
///
/// ```rust
/// use serde_yaml::{Value, to_value};
/// let val = to_value("foo");
/// assert_eq!(val, Value::String("foo".to_owned()))
/// ```
pub fn to_value<T: ?Sized>(value: &T) -> Value
    where T: Serialize,
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser).unwrap();
    ser.take().into()
}

/// Shortcut function to decode a YAML `Value` into a `T`.
///
/// ```rust
/// use serde_yaml::{Value, from_value};
/// let val = Value::String("foo".to_owned());
/// assert_eq!("foo", from_value::<String>(val).unwrap());
/// ```
pub fn from_value<T>(value: Value) -> Result<T, Error>
    where T: Deserialize,
{
    let yaml = value.into();
    let mut de = Deserializer::new(&yaml);
    Deserialize::deserialize(&mut de)
}

impl Value {
    pub fn is_null(&self) -> bool {
        if let Value::Null = *self {
            true
        } else {
            false
        }
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::I64(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_f64(&self) -> bool {
        self.as_f64().is_some()
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::F64(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(&s),
            _ => None,
        }
    }

    pub fn is_sequence(&self) -> bool {
        self.as_sequence().is_some()
    }

    pub fn as_sequence(&self) -> Option<&Sequence> {
        match *self {
            Value::Sequence(ref seq) => Some(seq),
            _ => None,
        }
    }

    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence> {
        match *self {
            Value::Sequence(ref mut seq) => Some(seq),
            _ => None,
        }
    }

    pub fn is_mapping(&self) -> bool {
        self.as_mapping().is_some()
    }

    pub fn as_mapping(&self) -> Option<&Mapping> {
        match *self {
            Value::Mapping(ref map) => Some(map),
            _ => None,
        }
    }

    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping> {
        match *self {
            Value::Mapping(ref mut map) => Some(map),
            _ => None,
        }
    }
}

impl From<Yaml> for Value {
    fn from(yaml: Yaml) -> Self {
        match yaml {
            Yaml::Real(f) => Value::F64(f.parse().unwrap()),
            Yaml::Integer(i) => Value::I64(i),
            Yaml::String(s) => Value::String(s),
            Yaml::Boolean(b) => Value::Bool(b),
            Yaml::Array(array) =>  {
                Value::Sequence(array.into_iter()
                                     .map(Into::into)
                                     .collect())
            }
            Yaml::Hash(hash) => {
                Value::Mapping(hash.into_iter()
                                   .map(|(k, v)| (k.into(), v.into()))
                                   .collect())
            }
            Yaml::Alias(_) => panic!("alias unsupported"),
            Yaml::Null => Value::Null,
            Yaml::BadValue => panic!("bad value"),
        }
    }
}

impl From<Value> for Yaml {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Yaml::Null,
            Value::Bool(b) => Yaml::Boolean(b),
            Value::I64(i) => Yaml::Integer(i),
            Value::F64(f) => {
                let mut buf = Vec::new();
                dtoa::write(&mut buf, f).unwrap();
                Yaml::Real(String::from_utf8(buf).unwrap())
            }
            Value::String(s) => Yaml::String(s),
            Value::Sequence(seq) => {
                Yaml::Array(seq.into_iter()
                               .map(Into::into)
                               .collect())
            }
            Value::Mapping(map) => {
                Yaml::Hash(map.into_iter()
                              .map(|(k, v)| (k.into(), v.into()))
                              .collect())
            }
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b),
            Value::I64(i) => serializer.serialize_i64(i),
            Value::F64(f) => serializer.serialize_f64(f),
            Value::String(ref s) => serializer.serialize_str(s),
            Value::Sequence(ref seq) => seq.serialize(serializer),
            Value::Mapping(ref map) => map.serialize(serializer),
        }
    }
}

impl Deserialize for Value {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            fn visit_bool<E>(&mut self, b: bool) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Bool(b))
            }

            fn visit_i64<E>(&mut self, i: i64) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::I64(i))
            }

            fn visit_u64<E>(&mut self, u: u64) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::I64(u as i64))
            }

            fn visit_f64<E>(&mut self, f: f64) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::F64(f))
            }

            fn visit_str<E>(&mut self, s: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::String(s.to_owned()))
            }

            fn visit_string<E>(&mut self, s: String) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::String(s))
            }

            fn visit_unit<E>(&mut self) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Null)
            }

            fn visit_none<E>(&mut self) -> Result<Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Null)
            }

            fn visit_some<D>(
                &mut self,
                deserializer: &mut D
            ) -> Result<Value, D::Error>
                where D: serde::Deserializer,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqVisitor,
            {
                use serde::de::impls::VecVisitor;
                let values = try!(VecVisitor::new().visit_seq(visitor));
                Ok(Value::Sequence(values))
            }

            fn visit_map<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapVisitor,
            {
                use linked_hash_map::serde::LinkedHashMapVisitor;
                let values = try!(LinkedHashMapVisitor::new().visit_map(visitor));
                Ok(Value::Mapping(values))
            }
        }

        deserializer.deserialize(ValueVisitor)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Null, &Value::Null) => true,
            (&Value::Bool(a), &Value::Bool(b)) => a == b,
            (&Value::I64(a), &Value::I64(b)) => a == b,
            (&Value::F64(a), &Value::F64(b)) => {
                if a.is_nan() && b.is_nan() {
                    // compare NaN for bitwise equality
                    let (a, b): (i64, i64) = unsafe {
                        (mem::transmute(a), mem::transmute(b))
                    };
                    a == b
                } else {
                    a == b
                }
            }
            (&Value::String(ref a), &Value::String(ref b)) => a == b,
            (&Value::Sequence(ref a), &Value::Sequence(ref b)) => a == b,
            (&Value::Mapping(ref a), &Value::Mapping(ref b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &Value::Null => 0.hash(state),
            &Value::Bool(b) => (1, b).hash(state),
            &Value::I64(i) => (2, i).hash(state),
            &Value::F64(_) => {
                // you should feel bad for using f64 as a map key
                3.hash(state);
            }
            &Value::String(ref s) => (4, s).hash(state),
            &Value::Sequence(ref seq) => (5, seq).hash(state),
            &Value::Mapping(ref map) => (6, map).hash(state),
        }
    }
}
