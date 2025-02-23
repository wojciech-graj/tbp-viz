use std::{collections::HashMap, result};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{GameId, Meta};

pub fn serialize<S>(value: &HashMap<GameId, Meta>, serializer: S) -> result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let vec: Vec<_> = value.values().collect();
    vec.serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> result::Result<HashMap<GameId, Meta>, D::Error>
where
    D: Deserializer<'de>,
{
    let metas = <Vec<Meta>>::deserialize(deserializer)?;
    Ok(metas
        .into_iter()
        .map(|meta| (meta.id.clone(), meta))
        .collect())
}
