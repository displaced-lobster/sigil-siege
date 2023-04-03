use bevy::prelude::*;

pub const BOARD_HEIGHT: f32 = 0.25;

#[derive(Resource)]
pub struct Board {
    player_board: [Option<Entity>; 4],
}

impl Board {
    pub fn new() -> Self {
        Self {
            player_board: [None; 4],
        }
    }

    pub fn adjacent(&self, entity: Entity) -> (Option<Entity>, Option<Entity>) {
        let index = self.player_board.iter().position(|e| *e == Some(entity));

        if let Some(index) = index {
            let left = if index == 0 {
                None
            } else {
                self.player_board[index - 1]
            };

            let right = if index == 3 {
                None
            } else {
                self.player_board[index + 1]
            };

            (left, right)
        } else {
            (None, None)
        }
    }

    pub fn others(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.player_board
            .iter()
            .filter_map(|e| *e)
            .filter(move |e| *e != entity)
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
