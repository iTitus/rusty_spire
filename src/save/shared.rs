use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(default, into = "String", try_from = "String")]
pub struct ModelId {
    pub category: String,
    pub entry: String,
}

impl Default for ModelId {
    fn default() -> Self {
        ModelId {
            category: "NONE".into(),
            entry: "NONE".into(),
        }
    }
}

impl Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.category, self.entry)
    }
}

impl From<ModelId> for String {
    fn from(value: ModelId) -> Self {
        value.to_string()
    }
}

impl FromStr for ModelId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (category, empty) = s
            .split('.')
            .collect_tuple()
            .ok_or("model id must contain exactly one dot")?;
        Ok(Self {
            category: category.into(),
            entry: empty.into(),
        })
    }
}

impl TryFrom<String> for ModelId {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocString {
    pub table: String,
    pub key: String,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedProperties {
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub ints: Vec<SavedProperty<i32>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub bools: Vec<SavedProperty<bool>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub strings: Vec<SavedProperty<String>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub int_arrays: Vec<SavedProperty<Vec<i32>>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub model_ids: Vec<SavedProperty<Vec<ModelId>>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub cards: Vec<SavedProperty<Card>>,
    #[serde(skip_serializing_if = "crate::serde::is_default")]
    pub card_arrays: Vec<SavedProperty<Vec<Card>>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SavedProperty<T> {
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub id: ModelId,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub props: SavedProperties,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Enchantment {
    pub id: ModelId,
    pub amount: i32,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub props: SavedProperties,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: ModelId,
    #[serde(
        rename = "current_upgrade_level",
        default,
        skip_serializing_if = "crate::serde::is_default"
    )]
    pub upgrades: i32,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub enchantment: Option<Enchantment>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub props: SavedProperties,
    #[serde(
        rename = "floor_added_to_deck",
        default,
        skip_serializing_if = "crate::serde::is_default"
    )]
    pub floor_added: Option<i32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Potion {
    pub id: ModelId,
    #[serde(rename = "slot_index")]
    pub slot: i32,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Relic {
    pub id: ModelId,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub props: SavedProperties,
    #[serde(
        rename = "floor_added_to_deck",
        default,
        skip_serializing_if = "crate::serde::is_default"
    )]
    pub floor_added: Option<i32>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    pub id: ModelId,
    pub rarity: BadgeRarity,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeRarity {
    None,
    Bronze,
    Silver,
    Gold,
}
