use bevy::prelude::*;
use rand::Rng;

use crate::cards::CardType;

pub const BOARD_HEIGHT: f32 = 0.25;

pub trait Board: Resource {
    fn adjacent(&self, entity: Entity) -> (Option<BoardPlacement>, Option<BoardPlacement>);
    fn others(&self, entity: Entity) -> Vec<BoardPlacement>;
    fn others_of_type(&self, entity: Entity, card_type: CardType) -> Vec<BoardPlacement>;
    fn place(&mut self, index: u32, entity: Entity, card_type: CardType);
    fn state(&self) -> &BoardState;
    fn unoccupied(&self, index: u32) -> bool;
}

#[derive(Resource)]
pub struct BoardState {
    board: [Option<BoardPlacement>; 4],
}

impl BoardState {
    pub fn new() -> Self {
        Self { board: [None; 4] }
    }

    pub fn adjacent(&self, entity: Entity) -> (Option<BoardPlacement>, Option<BoardPlacement>) {
        let index = self
            .board
            .iter()
            .filter_map(|e| *e)
            .position(|e| e.entity == entity);

        if let Some(index) = index {
            let left = if index == 0 {
                None
            } else {
                self.board[index - 1]
            };

            let right = if index == 3 {
                None
            } else {
                self.board[index + 1]
            };

            (left, right)
        } else {
            (None, None)
        }
    }

    pub fn others(&self, entity: Entity) -> impl Iterator<Item = BoardPlacement> + '_ {
        self.board
            .iter()
            .filter_map(|e| *e)
            .filter(move |e| e.entity != entity)
    }

    pub fn others_of_type(
        &self,
        entity: Entity,
        card_type: CardType,
    ) -> impl Iterator<Item = BoardPlacement> + '_ {
        self.board
            .iter()
            .filter_map(|e| *e)
            .filter(move |e| e.entity != entity && e.card_type == card_type)
    }

    pub fn place(&mut self, index: u32, entity: Entity, card_type: CardType) {
        self.board[index as usize] = Some(BoardPlacement { entity, card_type });
    }

    pub fn unoccupied(&self, index: u32) -> bool {
        self.board[index as usize].is_none()
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

#[derive(Resource)]
pub struct OpponentBoard {
    board_state: BoardState,
}

impl OpponentBoard {
    pub fn new() -> Self {
        Self {
            board_state: BoardState::new(),
        }
    }

    pub fn has_empty_place(&self) -> bool {
        self.board_state.board.iter().any(|e| e.is_none())
    }

    pub fn random_empty_place(&self) -> Option<u32> {
        if self.board_state.board.into_iter().any(|e| e.is_none()) {
            let mut rng = rand::thread_rng();
            let mut index = rng.gen_range(0..4);

            while self.board_state.board[index].is_some() {
                index = (index + 1) % 4;
            }

            Some(index as u32)
        } else {
            None
        }
    }
}

impl Board for OpponentBoard {
    fn adjacent(&self, entity: Entity) -> (Option<BoardPlacement>, Option<BoardPlacement>) {
        self.board_state.adjacent(entity)
    }
    fn others(&self, entity: Entity) -> Vec<BoardPlacement> {
        self.board_state.others(entity).collect()
    }
    fn others_of_type(&self, entity: Entity, card_type: CardType) -> Vec<BoardPlacement> {
        self.board_state.others_of_type(entity, card_type).collect()
    }
    fn place(&mut self, index: u32, entity: Entity, card_type: CardType) {
        self.board_state.place(index, entity, card_type);
    }
    fn state(&self) -> &BoardState {
        &self.board_state
    }
    fn unoccupied(&self, index: u32) -> bool {
        self.board_state.unoccupied(index)
    }
}

#[derive(Resource)]
pub struct PlayerBoard {
    board_state: BoardState,
}

impl PlayerBoard {
    pub fn new() -> Self {
        Self {
            board_state: BoardState::new(),
        }
    }
}

impl Board for PlayerBoard {
    fn adjacent(&self, entity: Entity) -> (Option<BoardPlacement>, Option<BoardPlacement>) {
        self.board_state.adjacent(entity)
    }
    fn others(&self, entity: Entity) -> Vec<BoardPlacement> {
        self.board_state.others(entity).collect()
    }
    fn others_of_type(&self, entity: Entity, card_type: CardType) -> Vec<BoardPlacement> {
        self.board_state.others_of_type(entity, card_type).collect()
    }
    fn place(&mut self, index: u32, entity: Entity, card_type: CardType) {
        self.board_state.place(index, entity, card_type);
    }
    fn state(&self) -> &BoardState {
        &self.board_state
    }
    fn unoccupied(&self, index: u32) -> bool {
        self.board_state.unoccupied(index)
    }
}

#[derive(Component)]
pub struct TurnDial;
