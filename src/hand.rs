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
pub struct Power(pub u32);
