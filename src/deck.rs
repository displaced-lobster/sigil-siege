use bevy::prelude::*;
use rand::Rng;

use crate::cards::CardType;

#[derive(Component)]
pub struct Deck(pub u32);

#[derive(Component)]
pub struct Draw;

struct DeckState {
    cards: Vec<CardType>,
}

#[derive(Resource)]
pub struct PlayerState {
    deck_state: DeckState,
    pub max_hand_size: u32,
    pub turn: u32,
}

impl PlayerState {
    pub fn deck_size(&self) -> u32 {
        self.deck_state.cards.len() as u32
    }

    pub fn draw_card(&mut self) -> Option<CardType> {
        if self.deck_size() == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.deck_size());

        Some(self.deck_state.cards.remove(index as usize))
    }

    pub fn draw_count(&self) -> u32 {
        let count = if self.turn == 0 { 3 } else { 1 };

        count.min(self.deck_size())
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        let heart_count = 2;
        let sword_count = 2;
        let tower_count = 2;
        let pitchfork_count = 6;
        let mut cards = Vec::new();

        for _ in 0..heart_count {
            cards.push(CardType::Heart);
        }

        for _ in 0..sword_count {
            cards.push(CardType::Sword);
        }

        for _ in 0..tower_count {
            cards.push(CardType::Tower);
        }

        for _ in 0..pitchfork_count {
            cards.push(CardType::Pitchfork);
        }

        Self {
            deck_state: DeckState { cards },
            max_hand_size: 5,
            turn: 0,
        }
    }
}
