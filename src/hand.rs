use bevy::prelude::*;

pub struct CardPlayedEvent {
    pub entity: Entity,
    pub index: u32,
}

#[derive(Component)]
pub struct Hand(pub u32);

#[derive(Component)]
pub struct Picked;

#[derive(Component)]
pub struct Power {
    pub index: u32,
    pub available: bool,
}

impl Power {
    pub fn new(index: u32) -> Self {
        Self {
            index,
            available: true,
        }
    }
}
