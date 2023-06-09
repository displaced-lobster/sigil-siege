use bevy::prelude::*;
use bevy_mod_picking::{
    HoverEvent,
    InteractablePickingPlugin,
    PickableBundle,
    PickingCameraBundle,
    PickingEvent,
    PickingPlugin,
};
use bevy_tweening::{
    lens::TransformPositionLens,
    Animator,
    EaseFunction,
    Tween,
    TweenCompleted,
    TweeningPlugin,
};
use rand::Rng;
use std::time::Duration;

mod board;
mod cards;
mod deck;
mod hand;
mod menu;
mod players;
mod states;

use board::*;
use cards::*;
use deck::*;
use hand::*;
use menu::*;
use players::*;
use states::*;

const ATTACK_TARGET_HEIGHT: f32 = 1.0;
const CAMERA_MENU_OFFSET: Vec3 = Vec3::new(0.0, 9.0, 1.0);
const HAND_Z: f32 = 6.5;
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
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(TweeningPlugin)
        .insert_resource(ClearColor(Color::rgb(0.06, 0.06, 0.08)))
        .add_startup_system(setup)
        .add_system(
            apply_ability::<Opponent, OpponentBoard>.in_set(OnUpdate(GameState::OpponentPlayCards)),
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
        .add_system(cleanup_game.in_schedule(OnEnter(GameState::StartGame)))
        .add_system(cleanup_system)
        .add_system(click_config_button)
        .add_system(click_play_button)
        .add_system(draw_cards.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(draw_cards_opponent.in_schedule(OnEnter(GameState::OpponentPlayCards)))
        .add_system(end_turn.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(end_turn_opponent.in_set(OnUpdate(GameState::OpponentTurn)))
        .add_system(game_over.in_set(OnUpdate(GameState::Lose)))
        .add_system(game_over.in_set(OnUpdate(GameState::Win)))
        .add_system(hover_button)
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
            receive_ability::<Opponent, OpponentBoard>
                .in_set(OnUpdate(GameState::OpponentPlayCards)),
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
        .add_system(setup_game.in_set(OnUpdate(GameState::StartGame)))
        .add_system(
            slide_hand
                .in_set(PlayCardSystemSet::CardPlayed)
                .in_set(OnUpdate(GameState::PlayerTurn)),
        )
        .add_system(spend_power.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(update_player_health)
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
    let block_mesh = asset_server.load("models/block.glb#Mesh0/Primitive0");
    let block_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/stone-base-color.png")),
        perceptual_roughness: 1.0,
        reflectance: 0.0,
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
        block_material,
        block_mesh,
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
        invisable: invisable_material.clone(),
        hovered: hovered_material,
    });

    let card_mesh = asset_server.load("models/card.glb#Mesh0/Primitive0");
    let card_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/card-base-color.png")),
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
        card_mesh: card_mesh.clone(),
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
    commands.insert_resource(OpponentState::default().with_deck_size(20).with_health(20));
    commands.insert_resource(PlayerState::default());

    let menu_translation = Vec3::new(50.0, 0.0, 50.0);
    let button_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.45, 0.11, 0.15),
        unlit: true,
        ..default()
    });
    let button_material_active = materials.add(StandardMaterial {
        base_color: Color::rgb(0.11, 0.45, 0.15),
        unlit: true,
        ..default()
    });

    commands
        .spawn((
            Menu,
            SpatialBundle {
                transform: Transform::from_translation(menu_translation),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: asset_server.load("models/sigil-siege-text.glb#Mesh0/Primitive0"),
                material: materials.add(StandardMaterial {
                    base_color: Color::WHITE,
                    base_color_texture: Some(asset_server.load("textures/text-base-color.png")),
                    perceptual_roughness: 1.0,
                    unlit: true,
                    ..default()
                }),
                ..default()
            });

            parent.spawn((
                MenuSelection::Small,
                ActiveSelection,
                Button,
                PickableBundle::default(),
                PbrBundle {
                    mesh: asset_server.load("models/small-btn.glb#Mesh0/Primitive0"),
                    material: button_material_active.clone(),
                    ..default()
                },
            ));

            parent.spawn((
                MenuSelection::Medium,
                Button,
                PickableBundle::default(),
                PbrBundle {
                    mesh: asset_server.load("models/medium-btn.glb#Mesh0/Primitive0"),
                    material: button_material.clone(),
                    ..default()
                },
            ));

            parent.spawn((
                MenuSelection::Large,
                Button,
                PickableBundle::default(),
                PbrBundle {
                    mesh: asset_server.load("models/large-btn.glb#Mesh0/Primitive0"),
                    material: button_material.clone(),
                    ..default()
                },
            ));

            parent.spawn((
                PlayButton,
                Button,
                PickableBundle::default(),
                PbrBundle {
                    mesh: asset_server.load("models/play-btn.glb#Mesh0/Primitive0"),
                    material: button_material.clone(),
                    ..default()
                },
            ));

            parent.spawn((
                GameOverText::Lose,
                PbrBundle {
                    mesh: asset_server.load("models/you-lost-text.glb#Mesh0/Primitive0"),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));

            parent.spawn((
                GameOverText::Win,
                PbrBundle {
                    mesh: asset_server.load("models/you-won-text.glb#Mesh0/Primitive0"),
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));
        });

    commands.insert_resource(MenuMaterials {
        button_material,
        button_material_active,
        button_material_hovered: materials.add(StandardMaterial {
            base_color: Color::rgb(0.45, 0.45, 0.15),
            unlit: true,
            ..default()
        }),
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(menu_translation + CAMERA_MENU_OFFSET)
                .looking_at(menu_translation, Vec3::Y),
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

    const DIAL_OFFSET: f32 = -7.5;

    commands.spawn(PbrBundle {
        mesh,
        material,
        ..default()
    });
    commands
        .spawn((
            PbrBundle {
                mesh: dial_mesh,
                material: dial_material,
                transform: Transform::from_xyz(DIAL_OFFSET, 0.0, 0.0),
                ..default()
            },
            TurnDial,
            PickableBundle::default(),
        ))
        .with_children(|parent| {
            parent.spawn((PbrBundle {
                mesh: arrow_mesh.clone(),
                material: arrow_material.clone(),
                ..default()
            },));
        });

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
                        mesh: card_mesh.clone(),
                        material: invisable_material.clone(),
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

        let target = if let Some(across) = attacking.across(attacked.state(), entity) {
            commands.entity(across.entity).insert(Damage(attack));

            q_attacked.get(across.entity).unwrap().translation
        } else {
            player_state.take_damage(attack);
            ev_attacked.send(S::attacked_event(attack.max(0) as u32));

            q_target.get_single().unwrap().translation
        };

        let attack_tween = Tween::new(
            EaseFunction::QuadraticIn,
            Duration::from_millis(200),
            TransformPositionLens {
                start: transform.translation,
                end: target,
            },
        );
        let return_tween = Tween::new(
            EaseFunction::QuadraticOut,
            Duration::from_millis(500),
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
    mut q_text: Query<(&GameOverText, &mut Visibility), (Without<Camera>, Without<Menu>)>,
) {
    for _ in ev_attacked.iter() {
        if player_state.get_health() <= 0 {
            state.set(GameState::Lose);

            for (text, mut visibility) in q_text.iter_mut() {
                match text {
                    GameOverText::Lose => *visibility = Visibility::Visible,
                    GameOverText::Win => *visibility = Visibility::Hidden,
                }
            }

            break;
        }
    }
}

fn check_win_condition(
    mut ev_attacked: EventReader<AttackedEvent>,
    opponent_state: Res<OpponentState>,
    mut state: ResMut<NextState<GameState>>,
    mut q_text: Query<(&GameOverText, &mut Visibility), (Without<Camera>, Without<Menu>)>,
) {
    for _ in ev_attacked.iter() {
        if opponent_state.get_health() <= 0 {
            state.set(GameState::Win);

            for (text, mut visibility) in q_text.iter_mut() {
                match text {
                    GameOverText::Lose => *visibility = Visibility::Hidden,
                    GameOverText::Win => *visibility = Visibility::Visible,
                }
            }

            break;
        }
    }
}

fn cleanup_game(
    mut commands: Commands,
    q_attack_target: Query<Entity, With<AttackTarget>>,
    q_card: Query<Entity, (With<CardType>, Without<AttackTarget>)>,
    q_deck: Query<Entity, (With<Deck>, Without<AttackTarget>, Without<CardType>)>,
    q_health: Query<
        Entity,
        (
            With<PlayerHealth>,
            Without<AttackTarget>,
            Without<CardType>,
            Without<Deck>,
        ),
    >,
    q_power: Query<
        Entity,
        (
            With<Power>,
            Without<AttackTarget>,
            Without<CardType>,
            Without<Deck>,
            Without<PlayerHealth>,
        ),
    >,
) {
    for entity in q_attack_target.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in q_card.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in q_deck.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in q_health.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in q_power.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_system(mut commands: Commands, mut q_cleanup: Query<(Entity, &CleanUp)>) {
    for (entity, _) in q_cleanup.iter_mut() {
        commands.entity(entity).despawn_recursive();
    }
}

fn click_config_button(
    mut commands: Commands,
    materials: Res<MenuMaterials>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_active: Query<(Entity, &mut Handle<StandardMaterial>), With<ActiveSelection>>,
    mut q_config: Query<
        (Entity, &mut Handle<StandardMaterial>),
        (With<MenuSelection>, Without<ActiveSelection>),
    >,
) {
    for ev in ev_pick.iter() {
        if let PickingEvent::Clicked(e) = ev {
            if let Ok((entity, mut material)) = q_config.get_mut(*e) {
                for (entity, mut material) in q_active.iter_mut() {
                    *material = materials.button_material.clone();
                    commands.entity(entity).remove::<ActiveSelection>();
                }

                *material = materials.button_material_active.clone();
                commands.entity(entity).insert(ActiveSelection);
            }
        }
    }
}

fn click_play_button(
    mut commands: Commands,
    mut ev_pick: EventReader<PickingEvent>,
    mut state: ResMut<NextState<GameState>>,
    q_play_btn: Query<With<PlayButton>>,
    mut q_camera: Query<&mut Transform, With<Camera>>,
    q_selection: Query<&MenuSelection, With<ActiveSelection>>,
) {
    const CAMERA_BOARD_OFFSET: Vec3 = Vec3::new(0.0, 9.0, 15.0);

    for ev in ev_pick.iter() {
        if let PickingEvent::Clicked(e) = ev {
            if q_play_btn.get(*e).is_ok() {
                let config = q_selection.single().game_config();

                commands.insert_resource(PlayerState::default().with_deck_size(config.deck_size));
                commands.insert_resource(
                    OpponentState::default()
                        .with_deck_size(config.deck_size)
                        .with_health(config.opponent_hp as i32),
                );

                let mut transform = q_camera.single_mut();

                *transform = Transform::from_translation(CAMERA_BOARD_OFFSET)
                    .looking_at(Vec3::ZERO, Vec3::Y);

                state.set(GameState::StartGame);
            }
        }
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
                Duration::from_millis(500),
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

fn draw_cards_opponent(mut state: ResMut<OpponentState>) {
    state.draw_cards();
}

fn game_over(
    mut player: ResMut<PlayerState>,
    q_acting: Query<(With<PerformingAction>, Without<Camera>, Without<Menu>)>,
    q_menu: Query<&Transform, (With<Menu>, Without<Camera>)>,
    mut q_camera: Query<&mut Transform, With<Camera>>,
) {
    if q_acting.iter().next().is_none() && !player.sent_to_menu {
        player.sent_to_menu = true;

        let menu = q_menu.single();
        let mut transform = q_camera.single_mut();

        *transform = Transform::from_translation(menu.translation + CAMERA_MENU_OFFSET)
            .looking_at(menu.translation, Vec3::Y);
    }
}

fn hover_button(
    materials: Res<MenuMaterials>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_button: Query<
        (&mut Handle<StandardMaterial>, Option<&ActiveSelection>),
        (With<Button>, Without<ActiveSelection>),
    >,
) {
    for ev in ev_pick.iter() {
        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok((mut material, _)) = q_button.get_mut(*e) {
                    *material = materials.button_material_hovered.clone();
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok((mut material, active)) = q_button.get_mut(*e) {
                    if active.is_some() {
                        *material = materials.button_material_active.clone();
                    } else {
                        *material = materials.button_material.clone();
                    }
                }
            }
            _ => {}
        }
    }
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
        if let PickingEvent::Clicked(e) = ev {
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
    let mut hand_size = q_hand.iter().count() as u32;

    if hand_size >= player_state.max_hand_size {
        return;
    }

    let draw_count = player_state.draw_count();
    player_state.turn += 1;
    let mut sorted_deck = q_deck.iter().collect::<Vec<_>>();

    sorted_deck.sort_by(|(_, deck_a), (_, deck_b)| deck_a.0.partial_cmp(&deck_b.0).unwrap());

    for _ in 0..draw_count {
        if hand_size < player_state.max_hand_size {
            if let Some((entity, __)) = sorted_deck.pop() {
                commands.entity(entity).insert(Draw);
            }
        }

        hand_size += 1;
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
        if let PickingEvent::Clicked(e) = ev {
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
            if let PickingEvent::Clicked(e) = ev {
                if let Ok((placeholder, placeholder_transform, mut material)) =
                    q_placeholder.get_mut(*e)
                {
                    let index = placeholder.0;

                    if board.unoccupied(index) {
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
            Duration::from_millis(600),
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
    mut q_pending: Query<(Entity, &mut Attack, &mut Health), (With<C>, With<PendingAbility>)>,
    q_cards: Query<(Entity, &CardType), (With<C>, Without<Hand>, Without<PendingAbility>)>,
) {
    for (entity, mut attack, mut health) in q_pending.iter_mut() {
        for (other_entity, card_type) in q_cards.iter() {
            if card_type
                .affects(other_entity, board.state())
                .contains(&entity)
            {
                card_type.effect().apply(&mut attack, &mut health);
            }
        }

        commands.entity(entity).remove::<PendingAbility>();
    }
}

fn remove_killed<C: Component, B: Board>(
    mut commands: Commands,
    mut board: ResMut<B>,
    q_killed: Query<(Entity, &CardType), (With<C>, With<Killed>)>,
    mut q_cards: Query<(&mut Attack, &mut Health), (With<C>, Without<Killed>)>,
) {
    if let Some((entity, card_type)) = q_killed.iter().next() {
        let affects = card_type.affects(entity, board.state());
        let effect = card_type.effect();

        for entity in affects {
            if let Ok((mut attack, mut health)) = q_cards.get_mut(entity) {
                effect.remove(&mut attack, &mut health);

                if health.0 <= 0 {
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

fn setup_game(
    mut commands: Commands,
    board_assets: Res<BoardAssets>,
    card_assets: Res<CardAssets>,
    opponent_state: Res<OpponentState>,
    player_state: Res<PlayerState>,
    mut state: ResMut<NextState<GameState>>,
) {
    const BLOCK_POSITIONS: [(f32, f32); 8] = [
        (0.0, BLOCK_SIZE),
        (BLOCK_SIZE, BLOCK_SIZE),
        (BLOCK_SIZE, 0.0),
        (BLOCK_SIZE, -BLOCK_SIZE),
        (0.0, -BLOCK_SIZE),
        (-BLOCK_SIZE, -BLOCK_SIZE),
        (-BLOCK_SIZE, 0.0),
        (-BLOCK_SIZE, BLOCK_SIZE),
    ];

    const PLAYER_HEALTH_OFFSET_Z: f32 = HAND_Z - 1.7;
    const PLAYER_HEALTH_WIDTH: f32 = 0.7;

    commands
        .spawn((
            SpatialBundle {
                transform: Transform::from_xyz(0.0, ATTACK_TARGET_HEIGHT, -6.0),
                ..default()
            },
            AttackTarget,
            Opponent,
        ))
        .with_children(|parent| {
            let mut rng = rand::thread_rng();
            let mut i = 0;
            let mut y = -ATTACK_TARGET_HEIGHT;

            for block_index in 0..opponent_state.get_health() {
                if i == BLOCK_POSITIONS.len() {
                    i = 0;
                    y += BLOCK_SIZE;
                }

                let (x, z) = BLOCK_POSITIONS[i];
                let rotation = rng.gen_range(0..4) as f32 * 90.0;

                parent.spawn((
                    PbrBundle {
                        mesh: board_assets.block_mesh.clone(),
                        material: board_assets.block_material.clone(),
                        transform: Transform::from_xyz(x, y, z)
                            .with_rotation(Quat::from_rotation_y(rotation.to_radians())),
                        ..default()
                    },
                    PlayerHealth(block_index as u32),
                    Opponent,
                ));

                i += 1;
            }
        });

    commands.spawn((
        TransformBundle {
            local: Transform::from_xyz(0.0, 3.0, 6.0),
            ..default()
        },
        AttackTarget,
        Player,
    ));

    for i in 0..player_state.get_health() {
        let x = i as f32 * PLAYER_HEALTH_WIDTH - 5.5;

        commands.spawn((
            PbrBundle {
                mesh: card_assets.heart_mesh.clone(),
                material: card_assets.heart_material.clone(),
                transform: Transform::from_xyz(x, 0.0, PLAYER_HEALTH_OFFSET_Z),
                ..default()
            },
            PlayerHealth(i as u32),
            Player,
        ));
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

fn update_player_health(
    mut commands: Commands,
    mut ev_attacked: EventReader<AttackedEvent>,
    q_block: Query<(Entity, &PlayerHealth), With<Opponent>>,
    q_hearts: Query<(Entity, &PlayerHealth), (With<Player>, Without<Opponent>)>,
) {
    let mut blocks = None;
    let mut hearts = None;

    for ev in ev_attacked.iter() {
        match ev {
            AttackedEvent::Opponent(damage) => {
                if blocks.is_none() {
                    let mut blocks_vec = q_block.iter().collect::<Vec<_>>();

                    blocks_vec.sort_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap());
                    blocks = Some(blocks_vec);
                }

                if let Some(blocks) = blocks.as_mut() {
                    for _ in 0..*damage {
                        if let Some((entity, _)) = blocks.pop() {
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                }
            }
            AttackedEvent::Player(damage) => {
                if hearts.is_none() {
                    let mut hearts_vec = q_hearts.iter().collect::<Vec<_>>();

                    hearts_vec.sort_by(|a, b| a.1 .0.partial_cmp(&b.1 .0).unwrap());
                    hearts = Some(hearts_vec);
                }

                if let Some(hearts) = hearts.as_mut() {
                    for _ in 0..*damage {
                        if let Some((entity, _)) = hearts.pop() {
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                }
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
            continue;
        }

        match sigil_count > attribute.get() {
            true => {
                while !sigils.is_empty() && sigils.len() as i32 > attribute.get() {
                    if let Some((entity, _)) = sigils.pop() {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            false => {
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
