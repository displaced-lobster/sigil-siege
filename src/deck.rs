use bevy::prelude::*;
use rand::Rng;

use crate::{cards::CardType, players::AttackedEvent, states::GameState};

#[derive(Component)]
pub struct Deck(pub u32);

#[derive(Component)]
pub struct Draw;

struct DeckState {
    cards: Vec<CardType>,
}

impl DeckState {
    fn draw(&mut self) -> Option<CardType> {
        if self.cards.is_empty() {
            return None;
        }

        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.cards.len());

        Some(self.cards.remove(index))
    }

    fn size(&self) -> u32 {
        self.cards.len() as u32
    }
}

impl Default for DeckState {
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

        Self { cards }
    }
}

#[derive(Resource)]
pub struct OpponentState {
    pub available_power: i32,
    deck_state: DeckState,
    hand: Vec<CardType>,
    health: i32,
    pub max_hand_size: u32,
    pub max_power: u32,
    pub power: u32,
    pub turn: u32,
}

impl OpponentState {
    pub fn can_play_card(&self) -> bool {
        self.hand
            .iter()
            .any(|card| self.available_power >= card.attributes().cost as i32)
    }

    pub fn draw_cards(&mut self) {
        let count = self.draw_count();

        for _ in 0..count {
            if self.max_hand_size > self.hand.len() as u32 {
                if let Some(card) = self.draw_card() {
                    self.hand.push(card);
                }
            }
        }
    }

    pub fn play_card(&mut self) -> Option<CardType> {
        self.hand.pop()
    }
}

impl Default for OpponentState {
    fn default() -> Self {
        Self {
            available_power: 0,
            deck_state: DeckState::default(),
            hand: Vec::new(),
            health: 12,
            max_hand_size: 5,
            max_power: 6,
            power: 0,
            turn: 0,
        }
    }
}

impl PlayableState for OpponentState {
    fn attacked_event() -> AttackedEvent {
        AttackedEvent::Opponent
    }

    fn deck_size(&self) -> u32 {
        self.deck_state.size()
    }

    fn draw_card(&mut self) -> Option<CardType> {
        self.deck_state.draw()
    }

    fn draw_count(&self) -> u32 {
        let count = if self.turn == 0 { 3 } else { 1 };

        count.min(self.deck_size())
    }

    fn get_available_power(&self) -> i32 {
        self.available_power
    }

    fn get_health(&self) -> i32 {
        self.health
    }

    fn get_max_power(&self) -> u32 {
        self.max_power
    }

    fn get_power(&self) -> u32 {
        self.power
    }

    fn next_turn() -> GameState {
        GameState::OpponentTurn
    }

    fn set_available_power(&mut self, power: i32) {
        self.available_power = power;
    }

    fn set_power(&mut self, power: i32) {
        self.power = power as u32;
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }
}

pub trait PlayableState: Resource {
    fn attacked_event() -> AttackedEvent;
    fn deck_size(&self) -> u32;
    fn draw_card(&mut self) -> Option<CardType>;
    fn draw_count(&self) -> u32;
    fn get_available_power(&self) -> i32;
    fn get_health(&self) -> i32;
    fn get_max_power(&self) -> u32;
    fn get_power(&self) -> u32;
    fn next_turn() -> GameState;
    fn set_available_power(&mut self, power: i32);
    fn set_power(&mut self, power: i32);
    fn show_power() -> bool {
        false
    }
    fn take_damage(&mut self, damage: i32);
}

#[derive(Resource)]
pub struct PlayerState {
    pub available_power: i32,
    deck_state: DeckState,
    pub health: i32,
    pub max_hand_size: u32,
    pub max_power: u32,
    pub power: u32,
    pub turn: u32,
}

impl PlayableState for PlayerState {
    fn attacked_event() -> AttackedEvent {
        AttackedEvent::Player
    }

    fn deck_size(&self) -> u32 {
        self.deck_state.size()
    }

    fn draw_card(&mut self) -> Option<CardType> {
        self.deck_state.draw()
    }

    fn draw_count(&self) -> u32 {
        let count = if self.turn == 0 { 3 } else { 1 };

        count.min(self.deck_size())
    }

    fn get_available_power(&self) -> i32 {
        self.available_power
    }

    fn get_health(&self) -> i32 {
        self.health
    }

    fn get_max_power(&self) -> u32 {
        self.max_power
    }

    fn get_power(&self) -> u32 {
        self.power
    }

    fn next_turn() -> GameState {
        GameState::PlayerTurn
    }

    fn set_available_power(&mut self, power: i32) {
        self.available_power = power;
    }

    fn set_power(&mut self, power: i32) {
        self.power = power as u32;
    }

    fn show_power() -> bool {
        true
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            available_power: 0,
            deck_state: DeckState::default(),
            health: 12,
            max_hand_size: 5,
            max_power: 5,
            power: 0,
            turn: 0,
        }
    }
}
