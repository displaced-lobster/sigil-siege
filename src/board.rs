use bevy::prelude::*;

#[derive(Resource)]
pub struct Board {
    player_board: [Option<Entity>; 4],
    opponent_board: [Option<Entity>; 4],
}

impl Board {
    pub fn new() -> Self {
        Self {
            player_board: [None; 4],
            opponent_board: [None; 4],
        }
    }

    pub fn place(&mut self, index: u32, entity: Entity) {
        self.player_board[index as usize] = Some(entity);
    }

    pub fn unoccupied(&self, index: u32) -> bool {
        self.player_board[index as usize].is_none()
    }
}

#[derive(Resource)]
pub struct BoardAssets {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}
