use bevy::prelude::*;

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
