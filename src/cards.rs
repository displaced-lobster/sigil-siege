use bevy::prelude::*;

use crate::board::{Board, BOARD_HEIGHT};

pub const ABILITY_MAX: i32 = 4;
pub const ATTRIBUTE_HEART_OFFSET: f32 = 1.4;
pub const ATTRIBUTE_SCALE: Vec3 = Vec3::new(0.5, 0.5, 0.5);
pub const ATTRIBUTE_SWORD_OFFSET: f32 = 1.0;
pub const ATTRIBUTE_WIDTH: f32 = 0.4;
pub const ATTRIBUTE_X_OFFSET: f32 = 0.6;
pub const CARD_THICKNESS: f32 = 0.05;
pub const CARD_HALF_THICKNESS: f32 = CARD_THICKNESS / 2.0;
pub const CARD_HEIGHT: f32 = 3.0;
pub const CARD_WIDTH: f32 = 2.0;

#[derive(Component)]
pub struct Attack(pub i32);

#[derive(Component)]
pub struct AttackSigil(pub u32);

pub trait Attribute {
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

impl Attribute for Health {
    fn get(&self) -> i32 {
        self.0
    }

    fn set(&mut self, value: i32) {
        self.0 = value;
    }
}

#[derive(Component)]
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
    pub pitchfork_mesh: Handle<Mesh>,
    pub sword_mesh: Handle<Mesh>,
    pub tower_mesh: Handle<Mesh>,
}

pub struct Attributes {
    pub attack: u32,
    pub health: u32,
}

pub enum CardAbility {
    AttackUpAdjacent,
    HealthUpAdjacent,
    HealthUpAll,
    StrengthInNumbers,
}

impl CardAbility {
    pub fn affects(&self, entity: Entity, board: &Board) -> Vec<Entity> {
        let mut affects = Vec::new();

        match self {
            Self::AttackUpAdjacent => {
                let (left, right) = board.adjacent(entity);

                if let Some(left) = left {
                    affects.push(left);
                }

                if let Some(right) = right {
                    affects.push(right);
                }
            }
            Self::HealthUpAdjacent => {
                let (left, right) = board.adjacent(entity);

                if let Some(left) = left {
                    affects.push(left);
                }

                if let Some(right) = right {
                    affects.push(right);
                }
            }
            Self::HealthUpAll => {
                for entity in board.others(entity) {
                    affects.push(entity);
                }
            }
            _ => {}
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

pub struct CardAbilityEffect {
    pub attack: i32,
    pub health: i32,
}

impl CardAbilityEffect {
    pub fn apply(&self, attack: &mut Attack, health: &mut Health) {
        attack.0 = (attack.0 + self.attack).min(ABILITY_MAX);
        health.0 = (health.0 + self.health).min(ABILITY_MAX);
    }
}

#[derive(Component)]
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

    pub fn affects(&self, entity: Entity, board: &Board) -> Vec<Entity> {
        self.ability().affects(entity, board)
    }

    pub fn attributes(&self) -> Attributes {
        match self {
            Self::Heart => Attributes {
                attack: 1,
                health: 1,
            },
            Self::Pitchfork => Attributes {
                attack: 1,
                health: 1,
            },
            Self::Sword => Attributes {
                attack: 2,
                health: 2,
            },
            Self::Tower => Attributes {
                attack: 1,
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
            Self::Pitchfork => assets.card_material.clone(),
            Self::Sword => assets.card_material.clone(),
            Self::Tower => assets.card_material.clone(),
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
    fn index(&self) -> u32;
    fn mesh(assets: &CardAssets) -> Handle<Mesh>;
    fn material(assets: &CardAssets) -> Handle<StandardMaterial> {
        assets.black_material.clone()
    }
    fn offset_y() -> f32 {
        0.0
    }
    fn offset_z() -> f32;
}

impl Sigil for AttackSigil {
    fn index(&self) -> u32 {
        self.0
    }

    fn mesh(assets: &CardAssets) -> Handle<Mesh> {
        assets.sword_mesh.clone()
    }

    fn offset_y() -> f32 {
        BOARD_HEIGHT
    }

    fn offset_z() -> f32 {
        ATTRIBUTE_SWORD_OFFSET
    }
}

impl Sigil for HealthSigil {
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
