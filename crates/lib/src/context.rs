use std::collections::HashMap;

use serde::{Serialize, Serializer, ser::SerializeStruct};

pub(crate) struct LinkerTemplateContext {
    cfg: HashMap<String, TemplateContextCfg>,
}

impl LinkerTemplateContext {
    pub(crate) fn new() -> Self {
        let cfg = std::env::vars()
            .filter_map(|(name, value)| {
                name.strip_prefix("CARGO_CFG_")
                    .map(|name| (name.to_lowercase(), TemplateContextCfg::from_env_value(value)))
            })
            .collect();

        Self { cfg }
    }
}

impl Serialize for LinkerTemplateContext {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut struct_serializer = serializer.serialize_struct("LinkerTemplateContext", 1)?;
        struct_serializer.serialize_field("cfg", &self.cfg)?;
        struct_serializer.end()
    }
}

pub(crate) enum TemplateContextCfg {
    Value(String),
    List(Vec<String>),
}

impl TemplateContextCfg {
    pub(crate) fn from_env_value(str_value: String) -> Self {
        if str_value.contains(',') {
            let values = str_value.split(',').map(ToOwned::to_owned).collect();
            TemplateContextCfg::List(values)
        } else {
            TemplateContextCfg::Value(str_value)
        }
    }
}

impl Serialize for TemplateContextCfg {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            TemplateContextCfg::Value(value) => value.serialize(serializer),
            TemplateContextCfg::List(list) => list.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for TemplateContextCfg {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(TemplateContextCfgVisitor)
    }
}

struct TemplateContextCfgVisitor;

impl<'de> serde::de::Visitor<'de> for TemplateContextCfgVisitor {
    type Value = TemplateContextCfg;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a singular string or list of strings")
    }

    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut buffer = Vec::with_capacity(seq.size_hint().unwrap_or_default());

        while let Some(element) = seq.next_element()? {
            buffer.push(element);
        }

        Ok(TemplateContextCfg::List(buffer))
    }

    fn visit_str<E: serde::de::Error>(self, str: &str) -> Result<Self::Value, E> {
        Ok(TemplateContextCfg::Value(str.to_owned()))
    }
}
