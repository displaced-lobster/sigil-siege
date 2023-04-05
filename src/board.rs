use bevy::prelude::*;

use crate::cards::CardType;

pub const BOARD_HEIGHT: f32 = 0.25;

#[derive(Resource)]
pub struct Board {
    player_board: [Option<BoardPlacement>; 4],
}

impl Board {
    pub fn new() -> Self {
        Self {
            player_board: [None; 4],
        }
    }

    pub fn adjacent(&self, entity: Entity) -> (Option<BoardPlacement>, Option<BoardPlacement>) {
        let index = self
            .player_board
            .iter()
            .filter_map(|e| *e)
            .position(|e| e.entity == entity);

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

    pub fn others(&self, entity: Entity) -> impl Iterator<Item = BoardPlacement> + '_ {
        self.player_board
            .iter()
            .filter_map(|e| *e)
            .filter(move |e| e.entity != entity)
    }

    pub fn others_of_type(
        &self,
        entity: Entity,
        card_type: CardType,
    ) -> impl Iterator<Item = BoardPlacement> + '_ {
        self.player_board
            .iter()
            .filter_map(|e| *e)
            .filter(move |e| e.entity != entity && e.card_type == card_type)
    }

    pub fn place(&mut self, index: u32, entity: Entity, card_type: CardType) {
        self.player_board[index as usize] = Some(BoardPlacement { entity, card_type });
    }

    pub fn unoccupied(&self, index: u32) -> bool {
        self.player_board[index as usize].is_none()
    }
}

#[derive(Resource)]
pub struct BoardAssets {
    pub arrow_material: Handle<StandardMaterial>,
    pub arrow_mesh: Handle<Mesh>,
    pub dial_material: Handle<StandardMaterial>,
    pub dial_mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Clone, Copy)]
pub struct BoardPlacement {
    pub entity: Entity,
    pub card_type: CardType,
}

#[derive(Component)]
pub struct TurnDial;
