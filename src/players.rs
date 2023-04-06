use bevy::prelude::*;

#[derive(Component)]
pub struct Damage(pub i32);

#[derive(Component, Default)]
pub struct Opponent;

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct CleanUp;
