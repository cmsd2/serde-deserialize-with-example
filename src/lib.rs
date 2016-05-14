#![feature(custom_derive)]
#![feature(custom_attribute)]

extern crate serde;

use std::marker::PhantomData;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Serialize, Deserialize)]
pub struct TypeWithNonDeserializableField {
    #[serde(deserialize_with="DeserializeWith::deserialize_with", serialize_with="SerializeWith::serialize_with")]
    pub value: TypeWithoutDeserialize
}

pub struct TypeWithoutDeserialize {
    pub value: String,
}

impl TypeWithoutDeserialize {
    pub fn new(s: String) -> TypeWithoutDeserialize {
        TypeWithoutDeserialize {
            value: s
        }
    }
}

pub trait SerializeWith: Sized {
    fn serialize_with<S>(&self, ser: &mut S) -> std::result::Result<(), S::Error>
        where S: Serializer;
}

pub trait DeserializeWith: Sized {
    fn deserialize_with<D>(de: &mut D) -> std::result::Result<Self, D::Error>
        where D: Deserializer;
}

impl SerializeWith for TypeWithoutDeserialize {
    fn serialize_with<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error> where S: Serializer {
        self.value.serialize(serializer)
    }
}
 
impl DeserializeWith for TypeWithoutDeserialize {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<TypeWithoutDeserialize, D::Error> where D: Deserializer {
        let s = try!(String::deserialize(deserializer));
        
        Ok(TypeWithoutDeserialize::new(s))
    }
}

/*
// this works fine:

impl DeserializeWith for Option<TypeWithoutDeserialize> {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<Option<TypeWithoutDeserialize>, D::Error> where D: Deserializer {
        let maybe_str: Option<String> = try!(Option::deserialize(deserializer));
        
        Ok(maybe_str.map(|s| TypeWithoutDeserialize::new(s)))
    }
}*/

// this lot doesn't work because serde::de::Visitor::Value has bound serde::de::Deserialize

impl <T> SerializeWith for Option<T> where T: SerializeWith {
    fn serialize_with<S>(&self, serializer: &mut S) -> std::result::Result<(), S::Error> where S: Serializer {
        if let Some(obj) = self.as_ref() {
            obj.serialize_with(serializer)
        } else {
            serializer.serialize_unit()
        }
    }
}

struct OptionDeserializeVisitor<T> where T: DeserializeWith {
    phantom_data: PhantomData<T>
}

impl <T> OptionDeserializeVisitor<T> where T: DeserializeWith {
    pub fn new() -> OptionDeserializeVisitor<T> {
        OptionDeserializeVisitor {
            phantom_data: PhantomData::new(),
        }
    }
}

impl <T> serde::de::Visitor for OptionDeserializeVisitor<T> where T: DeserializeWith {
    type Value = Option<T>;
    
    fn visit_none<E>(&mut self) -> Result<Self::Value, E> where E: serde::de::Error {
        None
    }
    
    fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Self::Value, D::Error> where D: Deserializer {
        Some(try!(T::deserialize_with(deserializer)))
    }
}
 
impl <T> DeserializeWith for Option<T> where T: DeserializeWith {
    fn deserialize_with<D>(deserializer: &mut D) -> std::result::Result<Option<T>, D::Error> where D: Deserializer {
        deserializer.deserialize_option(OptionDeserializeVisitor::new())
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
