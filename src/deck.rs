use bevy::prelude::*;
use rand::Rng;

use crate::{cards::CardType, players::AttackedEvent};

#[derive(Component)]
pub struct Deck(pub u32);

#[derive(Component)]
pub struct Draw;

struct DeckState {
    cards: Vec<CardType>,
}

impl DeckState {
    fn new(size: u32) -> Self {
        let mut rng = rand::thread_rng();
        let mut cards = Vec::new();

        for _ in 0..size {
            let n = rng.gen_range(0..12);

            match n {
                0 | 1 | 2 => {
                    cards.push(CardType::Heart);
                }
                3 | 4 => {
                    cards.push(CardType::Sword);
                }
                5 | 6 => {
                    cards.push(CardType::Tower);
                }
                _ => {
                    cards.push(CardType::Pitchfork);
                }
            }
        }

        Self { cards }
    }
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

#[derive(Resource)]
pub struct OpponentState {
    pub available_power: i32,
    deck_state: DeckState,
    hand: Vec<CardType>,
    health: i32,
    pub max_power: u32,
    pub power: u32,
    pub turn: u32,
}

impl OpponentState {
    pub fn can_play_card(&self) -> bool {
        self.deck_state
            .cards
            .iter()
            .any(|card| self.available_power >= card.attributes().cost as i32)
    }

    pub fn play_card(&mut self) -> Option<CardType> {
        let card_index = self
            .deck_state
            .cards
            .iter()
            .position(|card| self.available_power >= card.attributes().cost as i32)?;

        Some(self.deck_state.cards.remove(card_index))
    }
}

impl OpponentState {
    pub fn draw_cards(&mut self) {
        while self.hand.len() < 6 && !self.deck_state.cards.is_empty() {
            self.hand.push(self.deck_state.draw().unwrap());
        }
    }
}

impl Default for OpponentState {
    fn default() -> Self {
        Self {
            available_power: 0,
            deck_state: DeckState::new(12),
            hand: Vec::new(),
            health: 12,
            max_power: 5,
            power: 0,
            turn: 0,
        }
    }
}

impl PlayableState for OpponentState {
    fn attacked_event(damage: u32) -> AttackedEvent {
        AttackedEvent::Opponent(damage)
    }

    fn deck_size(&self) -> u32 {
        self.deck_state.size()
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

    fn set_available_power(&mut self, power: i32) {
        self.available_power = power;
    }

    fn set_power(&mut self, power: i32) {
        self.power = power as u32;
    }

    fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
    }

    fn with_deck_size(mut self, size: u32) -> Self {
        self.deck_state = DeckState::new(size);
        self
    }

    fn with_health(mut self, health: i32) -> Self {
        self.health = health;
        self
    }
}

pub trait PlayableState: Resource {
    fn attacked_event(damage: u32) -> AttackedEvent;
    fn deck_size(&self) -> u32;
    fn draw_card(&mut self) -> Option<CardType> {
        None
    }
    fn draw_count(&self) -> u32 {
        0
    }
    fn get_available_power(&self) -> i32;
    fn get_health(&self) -> i32;
    fn get_max_power(&self) -> u32;
    fn get_power(&self) -> u32;
    fn set_available_power(&mut self, power: i32);
    fn set_power(&mut self, power: i32);
    fn show_power() -> bool {
        false
    }
    fn take_damage(&mut self, damage: i32);
    fn with_deck_size(self, size: u32) -> Self;
    fn with_health(self, health: i32) -> Self;
}

#[derive(Resource)]
pub struct PlayerState {
    pub available_power: i32,
    deck_state: DeckState,
    pub health: i32,
    pub max_hand_size: u32,
    pub max_power: u32,
    pub power: u32,
    pub sent_to_menu: bool,
    pub turn: u32,
}

impl PlayableState for PlayerState {
    fn attacked_event(damage: u32) -> AttackedEvent {
        AttackedEvent::Player(damage)
    }

    fn deck_size(&self) -> u32 {
        self.deck_state.size()
    }

    fn draw_card(&mut self) -> Option<CardType> {
        self.deck_state.draw()
    }

    fn draw_count(&self) -> u32 {
        let count = if self.turn == 0 { 3 } else { 2 };

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

    fn with_deck_size(mut self, size: u32) -> Self {
        self.deck_state = DeckState::new(size);
        self
    }

    fn with_health(mut self, health: i32) -> Self {
        self.health = health;
        self
    }
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            available_power: 0,
            deck_state: DeckState::new(12),
            health: 10,
            max_hand_size: 5,
            max_power: 5,
            power: 0,
            sent_to_menu: false,
            turn: 0,
        }
    }
}
