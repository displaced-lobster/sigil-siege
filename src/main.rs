use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{
    HoverEvent,
    InteractablePickingPlugin,
    PickableBundle,
    PickingCameraBundle,
    PickingEvent,
    PickingPlugin,
    SelectionEvent,
};
use bevy_tweening::{
    lens::TransformPositionLens,
    Animator,
    EaseFunction,
    Tween,
    TweenCompleted,
    TweeningPlugin,
};
use std::time::Duration;

mod board;
mod cards;
mod deck;
mod hand;
mod players;
mod states;

use board::*;
use cards::*;
use deck::*;
use hand::*;
use players::*;
use states::*;

const HAND_Z: f32 = 6.0;
const TWEEN_EVENT_REMOVE_PERFORM_ACTION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum PlayCardSystemSet {
    PlayCard,
    CardPlayed,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_event::<AttackedEvent>()
        .add_event::<CardPlayedEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(TweeningPlugin)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .insert_resource(ClearColor(Color::rgb(0.06, 0.06, 0.08)))
        .add_startup_system(setup)
        .add_system(setup_board.in_set(OnUpdate(GameState::Setup)))
        .add_system(
            apply_ability::<Opponent, OpponentBoard>.run_if(resource_exists::<OpponentBoard>()),
        )
        .add_system(apply_ability::<Player, PlayerBoard>.run_if(resource_exists::<PlayerBoard>()))
        .add_system(apply_damage.in_schedule(OnExit(GameState::OpponentAttacking)))
        .add_system(apply_damage.in_schedule(OnExit(GameState::PlayerAttacking)))
        .add_system(
            attack::<Opponent, OpponentBoard, PlayerBoard, PlayerState>
                .in_set(OnUpdate(GameState::OpponentAttacking)),
        )
        .add_system(
            attack::<Player, PlayerBoard, OpponentBoard, OpponentState>
                .in_set(OnUpdate(GameState::PlayerAttacking)),
        )
        .add_system(attack_finished::<Opponent>.in_set(OnUpdate(GameState::OpponentAttacking)))
        .add_system(attack_finished::<Player>.in_set(OnUpdate(GameState::PlayerAttacking)))
        .add_system(check_lose_condition.run_if(resource_exists::<PlayerState>()))
        .add_system(check_win_condition.run_if(resource_exists::<OpponentState>()))
        .add_system(cleanup_system)
        .add_system(draw_cards.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(draw_opponent_cards.in_schedule(OnEnter(GameState::OpponentPlayCards)))
        .add_system(end_turn.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(end_turn_opponent.in_set(OnUpdate(GameState::OpponentTurn)))
        .add_system(hover_card_placeholder.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(hover_dial.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(hover_hand.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(
            mark_attackers::<OpponentBoard>.in_schedule(OnEnter(GameState::OpponentAttacking)),
        )
        .add_system(mark_attackers::<PlayerBoard>.in_schedule(OnEnter(GameState::PlayerAttacking)))
        .add_system(mark_cards_to_draw.in_schedule(OnEnter(GameState::PlayerTurn)))
        .add_system(pick_from_hand.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(
            play_card
                .in_set(PlayCardSystemSet::PlayCard)
                .in_set(OnUpdate(GameState::PlayerTurn))
                .before(PlayCardSystemSet::CardPlayed),
        )
        .add_system(play_opponent_cards.in_set(OnUpdate(GameState::OpponentPlayCards)))
        .add_system(
            receive_ability::<Opponent, OpponentBoard>.run_if(resource_exists::<OpponentBoard>()),
        )
        .add_system(receive_ability::<Player, PlayerBoard>.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(
            remove_killed::<Opponent, OpponentBoard>.run_if(resource_exists::<OpponentBoard>()),
        )
        .add_system(remove_killed::<Player, PlayerBoard>.run_if(resource_exists::<PlayerBoard>()))
        .add_system(remove_perform_action)
        .add_system(reset_dial.in_schedule(OnEnter(GameState::PlayerTurn)))
        .add_system(reset_hand.in_schedule(OnEnter(GameState::PlayerTurn)))
        .add_system(
            reset_power::<Opponent, OpponentState>
                .in_schedule(OnEnter(GameState::OpponentPlayCards)),
        )
        .add_system(reset_power::<Player, PlayerState>.in_schedule(OnEnter(GameState::PlayerTurn)))
        .add_system(
            slide_hand
                .in_set(PlayCardSystemSet::CardPlayed)
                .in_set(OnUpdate(GameState::PlayerTurn)),
        )
        .add_system(spend_power.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(update_sigils::<Attack, AttackSigil>)
        .add_system(update_sigils::<Cost, CostSigil>)
        .add_system(update_sigils::<Health, HealthSigil>)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arrow_mesh = asset_server.load("models/arrow.glb#Mesh0/Primitive0");
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.45, 0.11, 0.15),
        ..default()
    });
    let dial_mesh = asset_server.load("models/dial.glb#Mesh0/Primitive0");
    let dial_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.12, 0.12, 0.12),
        metallic: 1.0,
        perceptual_roughness: 0.2,
        ..default()
    });
    let mesh = asset_server.load("models/board.glb#Mesh0/Primitive0");
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/board-base-color.png")),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        ..default()
    });
    commands.insert_resource(BoardAssets {
        arrow_material,
        arrow_mesh,
        dial_material,
        dial_mesh,
        material,
        mesh,
    });

    let invisable_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        ..default()
    });
    let hovered_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 1.0, 0.9, 0.0499),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        ..default()
    });

    commands.insert_resource(CardPlaceholderMaterials {
        invisable: invisable_material,
        hovered: hovered_material,
    });

    let card_mesh = asset_server.load("models/card.glb#Mesh0/Primitive0");
    let card_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/card-base-color.png")),
        perceptual_roughness: 1.0,
        metallic: 1.0,
        metallic_roughness_texture: Some(asset_server.load("textures/card-metallic-roughness.png")),
        flip_normal_map_y: true,
        normal_map_texture: Some(asset_server.load("textures/card-normal.png")),
        reflectance: 0.0,
        ..default()
    });

    let heart_mesh = asset_server.load("models/heart.glb#Mesh0/Primitive0");
    let gem_mesh = asset_server.load("models/gem.glb#Mesh0/Primitive0");
    let pitchfork_mesh = asset_server.load("models/pitchfork.glb#Mesh0/Primitive0");
    let sword_mesh = asset_server.load("models/sword.glb#Mesh0/Primitive0");
    let tower_mesh = asset_server.load("models/tower.glb#Mesh0/Primitive0");
    let heart_material = materials.add(StandardMaterial {
        base_color: Color::RED,
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    let black_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        perceptual_roughness: 1.0,
        ..default()
    });
    let gem_material = materials.add(StandardMaterial {
        base_color: Color::PURPLE,
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });
    let gem_empty_material = materials.add(StandardMaterial {
        base_color: Color::GRAY,
        metallic: 1.0,
        perceptual_roughness: 0.0,
        ..default()
    });

    commands.insert_resource(CardAssets {
        card_mesh,
        card_material,
        heart_mesh,
        heart_material,
        gem_empty_material,
        gem_mesh,
        gem_material,
        pitchfork_mesh,
        sword_mesh,
        tower_mesh,
        black_material,
    });
    commands.insert_resource(OpponentState::default());
    commands.insert_resource(PlayerState::default());
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 9.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PickingCameraBundle::default(),
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            color: Color::WHITE,
            intensity: 5000.0,
            range: 80.0,
            radius: 12.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 9.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn apply_ability<C: Component, B: Board>(
    board: Res<B>,
    mut ev_played: EventReader<CardPlayedEvent>,
    mut q_cards: Query<(&CardType, &mut Attack, &mut Health), With<C>>,
) {
    for ev in ev_played.iter() {
        let (affects, effect) = if let Ok((card_type, _, _)) = q_cards.get_mut(ev.entity) {
            (
                Some(card_type.affects(ev.entity, board.state())),
                Some(card_type.effect()),
            )
        } else {
            (None, None)
        };

        info!("affects: {:?}, effect: {:?}", affects, effect);

        if let (Some(affects), Some(effect)) = (affects, effect) {
            for entity in affects {
                if let Ok((_, mut attack, mut health)) = q_cards.get_mut(entity) {
                    effect.apply(&mut attack, &mut health);
                }
            }
        }
    }
}

fn apply_damage(mut commands: Commands, mut q_damage: Query<(Entity, &Damage, &mut Health)>) {
    for (entity, damage, mut health) in q_damage.iter_mut() {
        health.0 -= damage.0;
        commands.entity(entity).remove::<Damage>();

        if health.0 <= 0 {
            info!("Card killed by damage");
            commands.entity(entity).insert(Killed);
        }
    }
}

fn attack<C: Component, A: Board, B: Board, S: PlayableState>(
    mut commands: Commands,
    attacking: Res<A>,
    attacked: Res<B>,
    mut player_state: ResMut<S>,
    mut ev_attacked: EventWriter<AttackedEvent>,
    q_attacking: Query<(Entity, &Attack, &Transform), (With<Attacker>, With<C>)>,
    q_attacked: Query<&Transform, (Without<AttackTarget>, Without<C>)>,
    q_target: Query<&Transform, (With<AttackTarget>, Without<Attacker>, Without<C>)>,
) {
    for (entity, attack, transform) in q_attacking.iter() {
        let attack = attack.get();
        info!("Attacking with {}", attack);

        let target = if let Some(across) = attacking.across(attacked.state(), entity) {
            info!("Attacking across");
            commands.entity(across.entity).insert(Damage(attack));

            q_attacked.get(across.entity).unwrap().translation
        } else {
            info!("Attacking player");
            player_state.take_damage(attack);
            info!("Player health: {}", player_state.get_health());
            ev_attacked.send(S::attacked_event());

            q_target.get_single().unwrap().translation
        };

        let attack_tween = Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(100),
            TransformPositionLens {
                start: transform.translation,
                end: target,
            },
        );
        let return_tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(250),
            TransformPositionLens {
                start: target,
                end: transform.translation,
            },
        )
        .with_completed_event(TWEEN_EVENT_REMOVE_PERFORM_ACTION);

        commands.entity(entity).remove::<Attacker>().insert((
            Animator::new(attack_tween.then(return_tween)),
            PerformingAction,
        ));
    }
}

fn attack_finished<C: Component>(
    current_state: Res<State<GameState>>,
    mut state: ResMut<NextState<GameState>>,
    q_attacker: Query<(With<Attacker>, With<C>)>,
    q_perform_action: Query<(With<PerformingAction>, With<C>)>,
) {
    if q_attacker.iter().next().is_none() && q_perform_action.iter().next().is_none() {
        let next_state = current_state.0.next().unwrap();
        state.set(next_state);
    }
}

fn check_lose_condition(
    mut ev_attacked: EventReader<AttackedEvent>,
    player_state: Res<PlayerState>,
    mut state: ResMut<NextState<GameState>>,
) {
    for _ in ev_attacked
        .iter()
        .filter(|ev| **ev == AttackedEvent::Player)
    {
        if player_state.get_health() <= 0 {
            info!("Lose!");
            state.set(GameState::Lose);
        }
    }
}

fn check_win_condition(
    mut ev_attacked: EventReader<AttackedEvent>,
    opponent_state: Res<OpponentState>,
    mut state: ResMut<NextState<GameState>>,
) {
    for _ in ev_attacked
        .iter()
        .filter(|ev| **ev == AttackedEvent::Opponent)
    {
        if opponent_state.get_health() <= 0 {
            info!("Win!");
            state.set(GameState::Win);
        }
    }
}

fn cleanup_system(mut commands: Commands, mut q_cleanup: Query<(Entity, &CleanUp)>) {
    for (entity, _) in q_cleanup.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}

fn draw_cards(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    mut player_state: ResMut<PlayerState>,
    mut q_draw: Query<(Entity, &mut Transform), (With<Draw>, With<Deck>, Without<Hand>)>,
    q_hand: Query<With<Hand>>,
) {
    let mut hand_size = q_hand.iter().count() as u32;
    let mut x = hand_size as f32 * CARD_WIDTH - 5.0;

    for (entity, mut transform) in q_draw.iter_mut() {
        if let Some(card_type) = player_state.draw_card() {
            *transform = transform.with_rotation(Quat::from_rotation_z(0.0));

            let end = Vec3::new(x, CARD_HALF_THICKNESS, HAND_Z);
            let tween = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(250),
                TransformPositionLens {
                    start: transform.translation,
                    end,
                },
            );

            x += CARD_WIDTH;

            let mesh = card_type.mesh(&card_assets);
            let material = card_type.material(&card_assets);
            let attributes = card_type.attributes();
            let child = commands
                .spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                })
                .id();

            commands
                .entity(entity)
                .remove::<Deck>()
                .remove::<Draw>()
                .insert((
                    card_type,
                    Hand(hand_size),
                    Attack(attributes.attack as i32),
                    Cost(attributes.cost as i32),
                    Health(attributes.health as i32),
                    Animator::new(tween),
                ))
                .insert(PickableBundle::default())
                .push_children(&[child]);

            hand_size += 1;
        }
    }
}

fn draw_opponent_cards(mut opponent_state: ResMut<OpponentState>) {
    opponent_state.draw_cards();
    opponent_state.turn += 1;
}

fn hover_card_placeholder(
    materials: Res<CardPlaceholderMaterials>,
    board: Res<PlayerBoard>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_placeholder: Query<(&CardPlaceholder, &mut Handle<StandardMaterial>)>,
    q_picked: Query<With<Picked>>,
) {
    for ev in ev_pick.iter() {
        if q_picked.iter().next().is_none() {
            return;
        }

        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok((placeholder, mut material)) = q_placeholder.get_mut(*e) {
                    if board.unoccupied(placeholder.0) {
                        *material = materials.hovered.clone();
                    }
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok((_, mut material)) = q_placeholder.get_mut(*e) {
                    *material = materials.invisable.clone();
                }
            }
            _ => {}
        }
    }
}

fn hover_dial(
    mut ev_pick: EventReader<PickingEvent>,
    mut q_dial: Query<&mut Transform, With<TurnDial>>,
) {
    for ev in ev_pick.iter() {
        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok(mut transform) = q_dial.get_mut(*e) {
                    *transform =
                        transform.with_rotation(Quat::from_rotation_y(90.0_f32.to_radians()));
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok(mut transform) = q_dial.get_mut(*e) {
                    *transform = transform.with_rotation(Quat::from_rotation_y(0.0));
                }
            }
            _ => {}
        }
    }
}

fn hover_hand(
    player_state: Res<PlayerState>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_hand: Query<(&Cost, &mut Transform), (With<Hand>, Without<Picked>)>,
) {
    for ev in ev_pick.iter() {
        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok((cost, mut transform)) = q_hand.get_mut(*e) {
                    if cost.0 <= player_state.available_power {
                        transform.translation.y += 0.5;
                    }
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok((_, mut transform)) = q_hand.get_mut(*e) {
                    transform.translation.y = CARD_HALF_THICKNESS;
                }
            }
            _ => {}
        }
    }
}

fn end_turn(
    mut ev_pick: EventReader<PickingEvent>,
    mut state: ResMut<NextState<GameState>>,
    mut q_dial: Query<&mut Transform, With<TurnDial>>,
) {
    for ev in ev_pick.iter() {
        if let PickingEvent::Selection(SelectionEvent::JustSelected(e)) = ev {
            if let Ok(mut transform) = q_dial.get_mut(*e) {
                *transform = transform.with_rotation(Quat::from_rotation_y(180.0_f32.to_radians()));
                state.set(GameState::PlayerAttacking);
            }
        }
    }
}

fn end_turn_opponent(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::OpponentAttacking);
}

fn mark_attackers<B: Board>(mut commands: Commands, board: Res<B>) {
    for placement in board.all() {
        commands.entity(placement.entity).insert(Attacker);
    }
}

fn mark_cards_to_draw(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    q_deck: Query<(Entity, &Deck), Without<Hand>>,
    q_hand: Query<With<Hand>>,
) {
    let hand_size = q_hand.iter().count() as u32;

    if hand_size >= player_state.max_hand_size {
        return;
    }

    let draw_count = player_state.draw_count();
    player_state.turn += 1;
    let mut sorted_deck = q_deck.iter().collect::<Vec<_>>();

    sorted_deck.sort_by(|(_, deck_a), (_, deck_b)| deck_a.0.partial_cmp(&deck_b.0).unwrap());

    for _ in 0..draw_count {
        if let Some((entity, __)) = sorted_deck.pop() {
            commands.entity(entity).insert(Draw);
        }
    }
}

fn pick_from_hand(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_hand: Query<(&Cost, &mut Transform), (With<Hand>, Without<Picked>)>,
    mut q_picked: Query<(Entity, &mut Transform), With<Picked>>,
) {
    for ev in ev_pick.iter() {
        if let PickingEvent::Selection(SelectionEvent::JustSelected(e)) = ev {
            if let Ok((cost, mut transform)) = q_hand.get_mut(*e) {
                if cost.0 <= player_state.available_power {
                    for (entity, mut picked_transform) in q_picked.iter_mut() {
                        picked_transform.translation.z += 1.0;
                        picked_transform.translation.y = CARD_HALF_THICKNESS;
                        commands.entity(entity).remove::<Picked>();
                    }

                    transform.translation.z -= 1.0;
                    commands.entity(*e).insert(Picked);
                }
            }
        }
    }
}

fn play_card(
    mut commands: Commands,
    placeholder_materials: Res<CardPlaceholderMaterials>,
    mut board: ResMut<PlayerBoard>,
    mut ev_pick: EventReader<PickingEvent>,
    mut ev_played: EventWriter<CardPlayedEvent>,
    mut q_placeholder: Query<(&CardPlaceholder, &Transform, &mut Handle<StandardMaterial>)>,
    mut q_picked: Query<
        (Entity, &Hand, &CardType, &mut Transform),
        (With<Picked>, Without<CardPlaceholder>),
    >,
) {
    for ev in ev_pick.iter() {
        if let Ok((picked_entity, hand, card_type, mut transform)) = q_picked.get_single_mut() {
            if let PickingEvent::Selection(SelectionEvent::JustSelected(e)) = ev {
                if let Ok((placeholder, placeholder_transform, mut material)) =
                    q_placeholder.get_mut(*e)
                {
                    let index = placeholder.0;

                    if board.unoccupied(index) {
                        info!("Playing card at index {}", index);
                        *material = placeholder_materials.invisable.clone();
                        transform.translation = placeholder_transform.translation;
                        board.place(index, picked_entity, *card_type);

                        ev_played.send(CardPlayedEvent {
                            entity: picked_entity,
                            index: hand.0,
                        });
                        commands
                            .entity(picked_entity)
                            .remove::<Picked>()
                            .remove::<Hand>()
                            .insert(PendingAbility);
                    }
                }
            }
        }
    }
}

fn play_opponent_cards(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    mut state: ResMut<NextState<GameState>>,
    mut board: ResMut<OpponentBoard>,
    mut opponent_state: ResMut<OpponentState>,
    mut ev_played: EventWriter<CardPlayedEvent>,
    q_placeholder: Query<(&CardPlaceholder, &Transform), With<Opponent>>,
    q_killed: Query<(With<Killed>, With<Opponent>, Without<CardPlaceholder>)>,
    q_acting: Query<(
        With<PerformingAction>,
        With<Opponent>,
        Without<CardPlaceholder>,
        Without<Killed>,
    )>,
) {
    if q_killed.iter().next().is_some() {
        return;
    }

    if board.has_empty_place() && opponent_state.can_play_card() {
        let card = opponent_state.play_card().unwrap();
        let index = board.random_empty_place().unwrap();
        let (_, transform) = q_placeholder.iter().find(|(p, _)| p.0 == index).unwrap();
        let end = transform.translation;
        let start = end + Vec3::new(0.0, 0.0, -10.0);
        let transform = transform.with_translation(start);
        let tween = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(300),
            TransformPositionLens { start, end },
        )
        .with_completed_event(TWEEN_EVENT_REMOVE_PERFORM_ACTION);
        let attributes = card.attributes();
        let mesh = card.mesh(&card_assets);
        let material = card.material(&card_assets);
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh: card_assets.card_mesh.clone(),
                    material: card_assets.card_material.clone(),
                    transform,
                    ..default()
                },
                card,
                Attack(attributes.attack as i32),
                Health(attributes.health as i32),
                Opponent,
                PendingAbility,
                Animator::new(tween),
                PerformingAction,
            ))
            .with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                });
            })
            .id();

        board.place(index, entity, card);
        ev_played.send(CardPlayedEvent { entity, index });
        opponent_state.available_power -= attributes.cost as i32;
    } else if q_acting.iter().next().is_none() {
        state.set(GameState::OpponentPlayCards.next().unwrap());
    }
}

fn receive_ability<C: Component, B: Board>(
    mut commands: Commands,
    board: Res<B>,
    mut ev_played: EventReader<CardPlayedEvent>,
    mut q_pending: Query<(&mut Attack, &mut Health), With<PendingAbility>>,
    q_cards: Query<(Entity, &CardType), (With<C>, Without<Hand>, Without<PendingAbility>)>,
) {
    for ev in ev_played.iter() {
        if let Ok((mut attack, mut health)) = q_pending.get_mut(ev.entity) {
            for (entity, card_type) in q_cards.iter() {
                if card_type
                    .affects(entity, board.state())
                    .contains(&ev.entity)
                {
                    card_type.effect().apply(&mut attack, &mut health);
                }
            }

            commands.entity(ev.entity).remove::<PendingAbility>();
        }
    }
}

fn remove_killed<C: Component, B: Board>(
    mut commands: Commands,
    mut board: ResMut<B>,
    q_killed: Query<(Entity, &CardType), (With<C>, With<Killed>)>,
    mut q_cards: Query<(&mut Attack, &mut Health), (With<C>, Without<Killed>)>,
) {
    if let Some((entity, card_type)) = q_killed.iter().next() {
        info!("Killed card found");
        let affects = card_type.affects(entity, board.state());
        let effect = card_type.effect();

        for entity in affects {
            if let Ok((mut attack, mut health)) = q_cards.get_mut(entity) {
                effect.remove(&mut attack, &mut health);

                if health.0 <= 0 {
                    info!("Card killed by side-effect");
                    commands.entity(entity).insert(Killed);
                }
            }
        }

        board.remove(entity);
        commands.entity(entity).despawn_recursive();
    }
}

fn remove_perform_action(mut commands: Commands, mut ev_completed: EventReader<TweenCompleted>) {
    for ev in ev_completed.iter() {
        if ev.user_data == TWEEN_EVENT_REMOVE_PERFORM_ACTION {
            commands.entity(ev.entity).remove::<PerformingAction>();
        }
    }
}

fn reset_dial(mut q_dial: Query<&mut Transform, With<TurnDial>>) {
    for mut transform in q_dial.iter_mut() {
        *transform = transform.with_rotation(Quat::from_rotation_y(0.0));
    }
}

fn reset_hand(mut q_hand: Query<&mut Hand>) {
    let mut hand = q_hand.iter_mut().collect::<Vec<_>>();

    hand.sort_by(|a, b| a.0.cmp(&b.0));

    for (i, hand) in hand.iter_mut().enumerate() {
        hand.0 = i as u32;
    }
}

fn reset_power<C: Component + Default, P: PlayableState>(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    mut player_state: ResMut<P>,
    mut q_power: Query<(&mut Power, &mut Handle<StandardMaterial>), With<C>>,
) {
    const POWER_OFFSET_X: f32 = 6.4;
    const POWER_OFFSET_Z: f32 = 3.5;
    const POWER_HEIGHT: f32 = 1.0;

    for (mut power, mut material) in q_power.iter_mut() {
        power.available = true;
        *material = card_assets.gem_material.clone();
    }

    let power = player_state
        .get_max_power()
        .min(player_state.get_power() + 1) as i32;

    player_state.set_power(power);
    player_state.set_available_power(power);

    if P::show_power() {
        let displayed_power = q_power.iter().count() as i32;
        let delta = player_state.get_power() as i32 - displayed_power;

        if delta > 0 {
            let index = displayed_power;
            let offset_z = -index as f32 * POWER_HEIGHT + POWER_OFFSET_Z;

            for i in 0..delta {
                let z = -i as f32 * POWER_HEIGHT + offset_z;

                commands.spawn((
                    PbrBundle {
                        mesh: card_assets.gem_mesh.clone(),
                        material: card_assets.gem_material.clone(),
                        transform: Transform::from_xyz(POWER_OFFSET_X, 0.0, z)
                            .with_scale(Vec3::splat(1.8)),
                        ..default()
                    },
                    Power::new((index + i) as u32),
                    C::default(),
                ));
            }
        }
    }
}

fn setup_board(
    mut commands: Commands,
    board_assets: Res<BoardAssets>,
    card_assets: Res<CardAssets>,
    placeholder_materials: Res<CardPlaceholderMaterials>,
    player_state: Res<PlayerState>,
    mut state: ResMut<NextState<GameState>>,
) {
    const DIAL_OFFSET: f32 = -7.5;

    commands.spawn(PbrBundle {
        mesh: board_assets.mesh.clone(),
        material: board_assets.material.clone(),
        ..default()
    });
    commands
        .spawn((
            PbrBundle {
                mesh: board_assets.dial_mesh.clone(),
                material: board_assets.dial_material.clone(),
                transform: Transform::from_xyz(DIAL_OFFSET, 0.0, 0.0),
                ..default()
            },
            TurnDial,
            PickableBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn((PbrBundle {
                mesh: board_assets.arrow_mesh.clone(),
                material: board_assets.arrow_material.clone(),
                ..default()
            },));
        });

    commands.spawn((
        TransformBundle {
            local: Transform::from_xyz(0.0, 3.0, -6.0),
            ..default()
        },
        AttackTarget,
        Opponent,
    ));

    commands.spawn((
        TransformBundle {
            local: Transform::from_xyz(0.0, 3.0, 6.0),
            ..default()
        },
        AttackTarget,
        Player,
    ));

    let card_padding = 1.0;
    let y = BOARD_HEIGHT + CARD_HALF_THICKNESS;
    let z_start = 2.0;

    for z in 0..2 {
        for x in 0..4 {
            let index = x as u32;
            let z = z_start - z as f32 * (CARD_HEIGHT + card_padding);
            let x = x as f32 * (CARD_WIDTH + card_padding) - 4.5;

            if z == z_start {
                commands.spawn((
                    PbrBundle {
                        mesh: card_assets.card_mesh.clone(),
                        material: placeholder_materials.invisable.clone(),
                        transform: Transform::from_xyz(x, y, z),
                        ..default()
                    },
                    CardPlaceholder(index),
                    Player,
                    PickableBundle::default(),
                ));
            } else {
                commands.spawn((
                    TransformBundle {
                        local: Transform::from_xyz(x, y, z),
                        ..default()
                    },
                    CardPlaceholder(index),
                    Opponent,
                ));
            }
        }
    }

    for i in 0..player_state.deck_size() {
        let y = i as f32 * CARD_THICKNESS + CARD_HALF_THICKNESS;

        commands.spawn((
            PbrBundle {
                mesh: card_assets.card_mesh.clone(),
                material: card_assets.card_material.clone(),
                transform: Transform::from_xyz(8.0, y, 5.0)
                    .with_rotation(Quat::from_rotation_z(180.0_f32.to_radians())),
                ..default()
            },
            Deck(i),
            Player,
        ));
    }

    commands.insert_resource(OpponentBoard::new());
    commands.insert_resource(PlayerBoard::new());
    state.set(GameState::PlayerTurn);
}

fn slide_hand(
    mut ev_played: EventReader<CardPlayedEvent>,
    mut q_hand: Query<(&Hand, &mut Transform)>,
    q_card: Query<With<Player>>,
) {
    for ev in ev_played.iter() {
        if q_card.get(ev.entity).is_err() {
            continue;
        }

        for (_, mut transform) in q_hand.iter_mut().filter(|(hand, _)| hand.0 > ev.index) {
            transform.translation.x -= CARD_WIDTH;
        }
    }
}

fn spend_power(
    card_assets: Res<CardAssets>,
    mut ev_played: EventReader<CardPlayedEvent>,
    mut player_state: ResMut<PlayerState>,
    q_cost: Query<&Cost>,
    mut q_power: Query<(&mut Power, &mut Handle<StandardMaterial>)>,
) {
    let mut power_spent = 0;

    for ev in ev_played.iter() {
        if let Ok(cost) = q_cost.get(ev.entity) {
            player_state.available_power -= cost.get();
            power_spent += cost.get();
        }
    }

    if power_spent > 0 {
        let mut power_vec = q_power
            .iter_mut()
            .filter(|(power, _)| power.available)
            .collect::<Vec<_>>();

        power_vec.sort_by(|a, b| a.0.index.partial_cmp(&b.0.index).unwrap());

        for _ in 0..power_spent {
            if let Some((mut power, mut material)) = power_vec.pop() {
                power.available = false;
                *material = card_assets.gem_empty_material.clone();
            }
        }
    }
}

fn update_sigils<A: Attribute + Component, S: Sigil + Component>(
    mut commands: Commands,
    card_assets: Res<CardAssets>,
    q_card: Query<(Entity, &A), Changed<A>>,
    q_sigil: Query<(Entity, &Parent, &S), Without<A>>,
) {
    for (entity, attribute) in q_card.iter() {
        info!("Changes detected for {:?}", attribute);
        let mut sigils = q_sigil
            .iter()
            .filter(|(_, parent, _)| parent.get() == entity)
            .map(|(entity, _, sigil)| (entity, sigil))
            .collect::<Vec<_>>();

        sigils.sort_by(|(_, sigil_a), (_, sigil_b)| {
            sigil_a.index().partial_cmp(&sigil_b.index()).unwrap()
        });

        let sigil_count = sigils.len() as i32;

        if sigil_count == attribute.get() {
            info!("No changes needed for {:?}", attribute);
            continue;
        }

        match sigil_count > attribute.get() {
            true => {
                info!("Removing sigils for {:?}", attribute);
                while !sigils.is_empty() && sigils.len() as i32 > attribute.get() {
                    if let Some((entity, _)) = sigils.pop() {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            false => {
                info!("Adding sigils for {:?}", attribute);
                let offset = S::direction() * sigil_count as f32 * S::width() - S::offset_x();
                let children = (0..attribute.get() - sigil_count)
                    .map(|i| {
                        let x = offset + i as f32 * S::width() * S::direction();
                        let index = sigil_count as u32 + i as u32;

                        commands
                            .spawn((
                                PbrBundle {
                                    mesh: S::mesh(&card_assets),
                                    material: S::material(&card_assets),
                                    transform: Transform::from_xyz(x, S::offset_y(), S::offset_z())
                                        .with_scale(S::scale()),
                                    ..default()
                                },
                                S::at_index(index),
                            ))
                            .id()
                    })
                    .collect::<Vec<_>>();

                commands.entity(entity).push_children(&children);
            }
        }
    }
}
