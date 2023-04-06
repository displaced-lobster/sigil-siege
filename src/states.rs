use bevy::prelude::States;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Setup,
    PlayerTurn,
    PlayerAttacking,
    OpponentTurn,
    OpponentAttacking,
    Win,
    Lose,
}
