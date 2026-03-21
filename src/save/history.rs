use crate::save::shared::{Card, LocString, ModelId, Potion, Relic, SavedProperties};
use crate::save::version::Migrate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunHistory {
    pub platform_type: PlatformType,
    #[serde(default)]
    pub game_mode: GameMode,
    pub win: bool,
    #[serde(default)]
    pub seed: String,
    pub start_time: i64,
    pub run_time: f32,
    pub ascension: i32,
    #[serde(default = "_default_build_id")]
    pub build_id: String,
    pub was_abandoned: bool,
    #[serde(default)]
    pub killed_by_encounter: ModelId,
    #[serde(default)]
    pub killed_by_event: ModelId,
    #[serde(default)]
    pub players: Vec<RunHistoryPlayer>,
    #[serde(default)]
    pub acts: Vec<ModelId>,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub map_point_history: Vec<Vec<MapPoint>>,
}

fn _default_build_id() -> String {
    "pre-v0.42".into()
}

impl Migrate for RunHistory {
    const CURRENT_SCHEMA_VERSION: i32 = 8;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformType {
    None,
    Steam,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    None,
    #[default]
    Standard,
    Daily,
    Custom,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub id: ModelId,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub props: SavedProperties,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunHistoryPlayer {
    pub id: u64,
    #[serde(default)]
    pub character: ModelId,
    #[serde(default)]
    pub deck: Vec<Card>,
    #[serde(default)]
    pub relics: Vec<Relic>,
    #[serde(default)]
    pub potions: Vec<Potion>,
    #[serde(default = "_max_potion_slot_count")]
    pub max_potion_slot_count: i32,
}

fn _max_potion_slot_count() -> i32 {
    3
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MapPoint {
    pub map_point_type: MapPointType,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub rooms: Vec<Room>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub player_stats: Vec<PlayerStat>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapPointType {
    Unassigned,
    Unknown,
    Shop,
    Treasure,
    RestSite,
    Monster,
    Elite,
    Boss,
    Ancient,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Room {
    pub room_type: RoomType,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub model_id: ModelId,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub monster_ids: Vec<ModelId>,
    pub turns_taken: i32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoomType {
    Unassigned,
    Monster,
    Elite,
    Boss,
    Treasure,
    Shop,
    Event,
    RestSite,
    Map,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlayerStat {
    pub player_id: u64,
    pub gold_gained: i32,
    pub gold_spent: i32,
    pub gold_lost: i32,
    pub gold_stolen: i32,
    pub current_gold: i32,
    pub current_hp: i32,
    pub max_hp: i32,
    pub damage_taken: i32,
    pub hp_healed: i32,
    pub max_hp_lost: i32,
    pub max_hp_gained: i32,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub ancient_choice: Vec<AncientChoice>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub cards_gained: Vec<Card>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub card_choices: Vec<CardChoice>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub relic_choices: Vec<ModelChoice>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub potion_choices: Vec<ModelChoice>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub potion_discarded: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub potion_used: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub cards_removed: Vec<Card>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub relics_removed: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub cards_enchanted: Vec<CardEnchantment>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub cards_transformed: Vec<CardTransformation>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub upgraded_cards: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub downgraded_cards: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub event_choices: Vec<EventChoice>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub rest_site_choices: Vec<String>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub bought_relics: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub bought_potions: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub bought_colorless: Vec<ModelId>,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub completed_quests: Vec<ModelId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AncientChoice {
    pub title: LocString,
    pub was_chosen: bool,
    // contains TextKey, which is just the second to last part of title.key (splitting at dots)
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardChoice {
    pub was_picked: bool,
    pub card: Card,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelChoice {
    pub choice: ModelId,
    pub was_picked: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardEnchantment {
    pub card: Card,
    pub enchantment: ModelId,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardTransformation {
    pub original_card: Card,
    pub final_card: Card,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct EventChoice {
    pub title: LocString,
    #[serde(default, skip_serializing_if = "crate::serde::is_default")]
    pub variables: HashMap<String, Value>,
}
