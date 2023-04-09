use bevy::prelude::*;

use crate::board::BoardState;

const ABILITY_MAX: i32 = 4;
const ATTRIBUTE_HEART_OFFSET: f32 = 1.4;
const ATTRIBUTE_GEM_OFFSET_X: f32 = -0.8;
const ATTRIBUTE_GEM_OFFSET_Z: f32 = -1.2;
const ATTRIBUTE_GEM_SCALE: Vec3 = Vec3::new(0.732, 0.732, 0.732);
const ATTRIBUTE_GEM_WIDTH: f32 = 0.25;
const ATTRIBUTE_SCALE: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const ATTRIBUTE_SWORD_OFFSET: f32 = 1.0;
const ATTRIBUTE_WIDTH: f32 = 0.4;
const ATTRIBUTE_X_OFFSET: f32 = 0.6;
pub const CARD_THICKNESS: f32 = 0.05;
pub const CARD_HALF_THICKNESS: f32 = CARD_THICKNESS / 2.0;
pub const CARD_HEIGHT: f32 = 3.0;
pub const CARD_WIDTH: f32 = 2.0;

#[derive(Component, Debug)]
pub struct Attack(pub i32);

#[derive(Component)]
pub struct AttackSigil(pub u32);

pub trait Attribute: std::fmt::Debug {
    fn get(&self) -> i32;
    fn set(&mut self, value: i32);
}

impl Attribute for Attack {
    fn get(&self) -> i32 {
        self.0
    }

    fn set(&mut self, value: i32) {
        self.0 = value;
    }
}

impl Attribute for Cost {
    fn get(&self) -> i32 {
        self.0
    }

    fn set(&mut self, value: i32) {
        self.0 = value;
    }
}

impl Attribute for Health {
    fn get(&self) -> i32 {
        self.0
    }

    fn set(&mut self, value: i32) {
        self.0 = value;
    }
}

#[derive(Component, Debug)]
pub struct Cost(pub i32);

#[derive(Component)]
pub struct CostSigil(pub u32);

#[derive(Component, Debug)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct HealthSigil(pub u32);

#[derive(Component)]
pub struct CardPlaceholder(pub u32);

#[derive(Resource)]
pub struct CardPlaceholderMaterials {
    pub invisable: Handle<StandardMaterial>,
    pub hovered: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct CardAssets {
    pub black_material: Handle<StandardMaterial>,
    pub card_material: Handle<StandardMaterial>,
    pub card_mesh: Handle<Mesh>,
    pub heart_material: Handle<StandardMaterial>,
    pub heart_mesh: Handle<Mesh>,
    pub gem_empty_material: Handle<StandardMaterial>,
    pub gem_material: Handle<StandardMaterial>,
    pub gem_mesh: Handle<Mesh>,
    pub pitchfork_mesh: Handle<Mesh>,
    pub sword_mesh: Handle<Mesh>,
    pub tower_mesh: Handle<Mesh>,
}

pub struct Attributes {
    pub attack: u32,
    pub cost: u32,
    pub health: u32,
}

pub enum CardAbility {
    AttackUpAdjacent,
    HealthUpAdjacent,
    HealthUpAll,
    StrengthInNumbers,
}

impl CardAbility {
    pub fn affects(&self, entity: Entity, card_type: CardType, board: &BoardState) -> Vec<Entity> {
        let mut affects = Vec::new();

        match self {
            Self::AttackUpAdjacent => {
                let (left, right) = board.adjacent(entity);

                if let Some(left) = left {
                    affects.push(left.entity);
                }

                if let Some(right) = right {
                    affects.push(right.entity);
                }
            }
            Self::HealthUpAdjacent => {
                let (left, right) = board.adjacent(entity);

                if let Some(left) = left {
                    affects.push(left.entity);
                }

                if let Some(right) = right {
                    affects.push(right.entity);
                }
            }
            Self::HealthUpAll => {
                for entity in board.others(entity) {
                    affects.push(entity.entity);
                }
            }
            Self::StrengthInNumbers => {
                for entity in board.others_of_type(entity, card_type) {
                    affects.push(entity.entity);
                }
            }
        }

        affects
    }

    pub fn effect(&self) -> CardAbilityEffect {
        match self {
            Self::AttackUpAdjacent | Self::StrengthInNumbers => CardAbilityEffect {
                attack: 1,
                health: 0,
            },
            Self::HealthUpAdjacent | Self::HealthUpAll => CardAbilityEffect {
                attack: 0,
                health: 1,
            },
        }
    }
}

#[derive(Debug)]
pub struct CardAbilityEffect {
    pub attack: i32,
    pub health: i32,
}

impl CardAbilityEffect {
    pub fn apply(&self, attack: &mut Attack, health: &mut Health) {
        attack.0 = (attack.0 + self.attack).min(ABILITY_MAX);
        health.0 = (health.0 + self.health).min(ABILITY_MAX);
    }

    pub fn remove(&self, attack: &mut Attack, health: &mut Health) {
        attack.0 = (attack.0 - self.attack).max(0);
        health.0 = (health.0 - self.health).max(0);
    }
}

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub enum CardType {
    Heart,
    Pitchfork,
    Sword,
    Tower,
}

impl CardType {
    pub fn ability(&self) -> CardAbility {
        match self {
            Self::Heart => CardAbility::HealthUpAll,
            Self::Pitchfork => CardAbility::StrengthInNumbers,
            Self::Sword => CardAbility::AttackUpAdjacent,
            Self::Tower => CardAbility::HealthUpAdjacent,
        }
    }

    pub fn affects(&self, entity: Entity, board: &BoardState) -> Vec<Entity> {
        self.ability().affects(entity, *self, board)
    }

    pub fn attributes(&self) -> Attributes {
        match self {
            Self::Heart => Attributes {
                attack: 1,
                cost: 2,
                health: 1,
            },
            Self::Pitchfork => Attributes {
                attack: 1,
                cost: 1,
                health: 1,
            },
            Self::Sword => Attributes {
                attack: 2,
                cost: 2,
                health: 2,
            },
            Self::Tower => Attributes {
                attack: 1,
                cost: 2,
                health: 3,
            },
        }
    }

    pub fn effect(&self) -> CardAbilityEffect {
        self.ability().effect()
    }

    pub fn material(&self, assets: &CardAssets) -> Handle<StandardMaterial> {
        match self {
            Self::Heart => assets.heart_material.clone(),
            _ => assets.black_material.clone(),
        }
    }

    pub fn mesh(&self, assets: &CardAssets) -> Handle<Mesh> {
        match self {
            Self::Heart => assets.heart_mesh.clone(),
            Self::Pitchfork => assets.pitchfork_mesh.clone(),
            Self::Sword => assets.sword_mesh.clone(),
            Self::Tower => assets.tower_mesh.clone(),
        }
    }
}

#[derive(Component)]
pub struct PendingAbility;

pub trait Sigil {
    fn at_index(index: u32) -> Self;
    fn direction() -> f32 {
        1.0
    }
    fn index(&self) -> u32;
    fn mesh(assets: &CardAssets) -> Handle<Mesh>;
    fn material(assets: &CardAssets) -> Handle<StandardMaterial> {
        assets.black_material.clone()
    }
    fn offset_x() -> f32 {
        ATTRIBUTE_X_OFFSET
    }
    fn offset_y() -> f32 {
        0.0
    }
    fn offset_z() -> f32;
    fn scale() -> Vec3 {
        ATTRIBUTE_SCALE
    }
    fn width() -> f32 {
        ATTRIBUTE_WIDTH
    }
}

impl Sigil for AttackSigil {
    fn at_index(index: u32) -> Self {
        Self(index)
    }

    fn index(&self) -> u32 {
        self.0
    }

    fn mesh(assets: &CardAssets) -> Handle<Mesh> {
        assets.sword_mesh.clone()
    }

    fn offset_y() -> f32 {
        0.025
    }

    fn offset_z() -> f32 {
        ATTRIBUTE_SWORD_OFFSET
    }
}

impl Sigil for CostSigil {
    fn at_index(index: u32) -> Self {
        Self(index)
    }

    fn direction() -> f32 {
        -1.0
    }

    fn index(&self) -> u32 {
        self.0
    }

    fn mesh(assets: &CardAssets) -> Handle<Mesh> {
        assets.gem_mesh.clone()
    }

    fn material(assets: &CardAssets) -> Handle<StandardMaterial> {
        assets.gem_material.clone()
    }

    fn offset_x() -> f32 {
        ATTRIBUTE_GEM_OFFSET_X
    }

    fn offset_z() -> f32 {
        ATTRIBUTE_GEM_OFFSET_Z
    }

    fn scale() -> Vec3 {
        ATTRIBUTE_GEM_SCALE
    }

    fn width() -> f32 {
        ATTRIBUTE_GEM_WIDTH
    }
}

impl Sigil for HealthSigil {
    fn at_index(index: u32) -> Self {
        Self(index)
    }

    fn index(&self) -> u32 {
        self.0
    }

    fn material(assets: &CardAssets) -> Handle<StandardMaterial> {
        assets.heart_material.clone()
    }

    fn mesh(assets: &CardAssets) -> Handle<Mesh> {
        assets.heart_mesh.clone()
    }

    fn offset_z() -> f32 {
        ATTRIBUTE_HEART_OFFSET
    }
}
