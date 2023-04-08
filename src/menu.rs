use bevy::prelude::*;

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct Button;

#[derive(Resource)]
pub struct MenuMaterials {
    pub button_material: Handle<StandardMaterial>,
    pub button_material_active: Handle<StandardMaterial>,
    pub button_material_hovered: Handle<StandardMaterial>,
}

pub struct GameConfig {
    pub deck_size: u32,
    pub opponent_hp: u32,
}

#[derive(Clone, Copy, Component)]
pub enum MenuSelection {
    Small,
    Medium,
    Large,
}

impl MenuSelection {
    pub fn game_config(&self) -> GameConfig {
        match self {
            MenuSelection::Small => GameConfig {
                deck_size: 12,
                opponent_hp: 20,
            },
            MenuSelection::Medium => GameConfig {
                deck_size: 24,
                opponent_hp: 48,
            },
            MenuSelection::Large => GameConfig {
                deck_size: 36,
                opponent_hp: 60,
            },
        }
    }
}

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct ActiveSelection;
