use bevy::prelude::*;

#[derive(Component)]
pub struct Damage(pub i32);

#[derive(Component, Default)]
pub struct Opponent;

#[derive(Component, Default)]
pub struct Player;

#[derive(Component)]
pub struct CleanUp;

#[derive(PartialEq, Eq)]
pub enum AttackedEvent {
    Player(u32),
    Opponent(u32),
}

#[derive(Component)]
pub struct Killed;

#[derive(Component)]
pub struct Attacker;

#[derive(Component)]
pub struct PerformingAction;

#[derive(Component)]
pub struct AttackTarget;

#[derive(Component)]
pub struct Block(pub u32);
