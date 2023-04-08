use bevy::prelude::States;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Setup,
    StartGame,
    PlayerTurn,
    PlayerAttacking,
    OpponentPlayCards,
    OpponentTurn,
    OpponentAttacking,
    Win,
    Lose,
}

impl GameState {
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Setup => Some(Self::StartGame),
            Self::StartGame => Some(Self::PlayerTurn),
            Self::PlayerTurn => Some(Self::PlayerAttacking),
            Self::PlayerAttacking => Some(Self::OpponentPlayCards),
            Self::OpponentPlayCards => Some(Self::OpponentTurn),
            Self::OpponentTurn => Some(Self::OpponentAttacking),
            Self::OpponentAttacking => Some(Self::PlayerTurn),
            Self::Win => None,
            Self::Lose => None,
        }
    }
}
