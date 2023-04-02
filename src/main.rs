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

mod cards;
mod deck;
mod hand;
mod states;

use cards::*;
use deck::*;
use hand::*;
use states::*;

const BOARD_HEIGHT: f32 = 0.25;
const HAND_Z: f32 = 6.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
enum PlayCardSystemSet {
    PlayCard,
    CardPlayed,
}

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_event::<CardPlayedEvent>()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .insert_resource(ClearColor(Color::rgb(0.06, 0.06, 0.08)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::rgb(0.63, 0.51, 0.51),
        })
        .add_startup_system(setup)
        .add_system(setup_board.in_set(OnUpdate(GameState::Setup)))
        .add_system(
            card_played
                .in_set(PlayCardSystemSet::CardPlayed)
                .in_set(OnUpdate(GameState::PlayerTurn)),
        )
        .add_system(draw_cards.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(hover_card_placeholder.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(hover_hand.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(mark_cards_to_draw.in_schedule(OnEnter(GameState::PlayerTurn)))
        .add_system(pick_from_hand.in_set(OnUpdate(GameState::PlayerTurn)))
        .add_system(
            play_card
                .in_set(PlayCardSystemSet::PlayCard)
                .in_set(OnUpdate(GameState::PlayerTurn))
                .before(PlayCardSystemSet::CardPlayed),
        )
        .run();
}

#[derive(Component)]
struct Picked;

#[derive(Resource)]
struct BoardAssets {
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = asset_server.load("models/board.glb#Mesh0/Primitive0");
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load("textures/board-base-color.png")),
        perceptual_roughness: 1.0,
        metallic_roughness_texture: Some(asset_server.load("textures/board-roughness.png")),
        flip_normal_map_y: true,
        normal_map_texture: Some(asset_server.load("textures/board-normal.png")),
        reflectance: 0.0,
        ..default()
    });
    commands.insert_resource(BoardAssets { material, mesh });

    let invisable_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    let hovered_material = materials.add(StandardMaterial {
        base_color: Color::rgba(0.0, 0.82, 0.09, 0.65),
        alpha_mode: AlphaMode::Add,
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

    commands.insert_resource(CardAssets {
        card_mesh,
        card_material,
        heart_mesh,
        heart_material,
        pitchfork_mesh,
        sword_mesh,
        tower_mesh,
        black_material,
    });

    commands.insert_resource(PlayerState::default());
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PickingCameraBundle::default(),
    ));
}

fn card_played(
    mut ev_played: EventReader<CardPlayedEvent>,
    mut q_hand: Query<(&Hand, &mut Transform)>,
) {
    for ev in ev_played.iter() {
        for (_, mut transform) in q_hand.iter_mut().filter(|(hand, _)| hand.0 > ev.index) {
            transform.translation.x -= CARD_WIDTH;
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
            *transform = Transform::from_xyz(x, CARD_HALF_THICKNESS, HAND_Z);
            x += CARD_WIDTH;

            let mesh = card_type.mesh(&card_assets);
            let material = card_type.material(&card_assets);
            let mut children = Vec::new();

            children.push(
                commands
                    .spawn(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        ..default()
                    })
                    .id(),
            );

            let attributes = card_type.attributes();

            for i in 0..attributes.health {
                let x = i as f32 * ATTRIBUTE_WIDTH - ATTRIBUTE_X_OFFSET;

                children.push(
                    commands
                        .spawn(PbrBundle {
                            mesh: card_assets.heart_mesh.clone(),
                            material: card_assets.heart_material.clone(),
                            transform: Transform::from_xyz(x, 0.0, ATTRIBUTE_HEART_OFFSET)
                                .with_scale(ATTRIBUTE_SCALE),
                            ..default()
                        })
                        .id(),
                );
            }

            for i in 0..attributes.attack {
                let x = i as f32 * ATTRIBUTE_WIDTH - ATTRIBUTE_X_OFFSET;

                children.push(
                    commands
                        .spawn(PbrBundle {
                            mesh: card_assets.sword_mesh.clone(),
                            material: card_assets.black_material.clone(),
                            transform: Transform::from_xyz(x, BOARD_HEIGHT, ATTRIBUTE_SWORD_OFFSET)
                                .with_scale(ATTRIBUTE_SCALE),
                            ..default()
                        })
                        .id(),
                );
            }

            commands
                .entity(entity)
                .remove::<Deck>()
                .remove::<Draw>()
                .insert(Hand(hand_size))
                .insert(PickableBundle::default())
                .push_children(&children);

            hand_size += 1;
        }
    }
}

fn hover_card_placeholder(
    materials: Res<CardPlaceholderMaterials>,
    mut ev_pick: EventReader<PickingEvent>,
    mut q_placeholder: Query<&mut Handle<StandardMaterial>, With<CardPlaceholder>>,
    q_picked: Query<With<Picked>>,
) {
    for ev in ev_pick.iter() {
        if q_picked.iter().next().is_none() {
            return;
        }

        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok(mut material) = q_placeholder.get_mut(*e) {
                    *material = materials.hovered.clone();
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok(mut material) = q_placeholder.get_mut(*e) {
                    *material = materials.invisable.clone();
                }
            }
            _ => {}
        }
    }
}

fn hover_hand(
    mut ev_pick: EventReader<PickingEvent>,
    mut q_hand: Query<&mut Transform, (With<Hand>, Without<Picked>)>,
) {
    for ev in ev_pick.iter() {
        match ev {
            PickingEvent::Hover(HoverEvent::JustEntered(e)) => {
                if let Ok(mut transform) = q_hand.get_mut(*e) {
                    transform.translation.y += 0.5;
                }
            }
            PickingEvent::Hover(HoverEvent::JustLeft(e)) => {
                if let Ok(mut transform) = q_hand.get_mut(*e) {
                    transform.translation.y = CARD_HALF_THICKNESS;
                }
            }
            _ => {}
        }
    }
}

fn mark_cards_to_draw(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    q_deck: Query<(Entity, &Deck), Without<Hand>>,
    q_hand: Query<With<Hand>>,
) {
    let hand_size = q_hand.iter().count() as u32;

    if hand_size >= player_state.max_hand_size {
        return;
    }

    let draw_count = player_state.draw_count();
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
    mut ev_pick: EventReader<PickingEvent>,
    mut q_hand: Query<&mut Transform, With<Hand>>,
) {
    for ev in ev_pick.iter() {
        match ev {
            PickingEvent::Selection(SelectionEvent::JustSelected(e)) => {
                if let Ok(mut transform) = q_hand.get_mut(*e) {
                    transform.translation.z -= 1.0;
                    commands.entity(*e).insert(Picked);
                }
            }
            PickingEvent::Selection(SelectionEvent::JustDeselected(e)) => {
                if let Ok(mut transform) = q_hand.get_mut(*e) {
                    transform.translation.y = CARD_HALF_THICKNESS;
                    transform.translation.z = HAND_Z;
                    commands.entity(*e).remove::<Picked>();
                }
            }
            _ => {}
        }
    }
}

fn play_card(
    mut commands: Commands,
    placeholder_materials: Res<CardPlaceholderMaterials>,
    mut ev_pick: EventReader<PickingEvent>,
    mut ev_played: EventWriter<CardPlayedEvent>,
    mut q_placeholder: Query<(&Transform, &mut Handle<StandardMaterial>), With<CardPlaceholder>>,
    mut q_picked: Query<(Entity, &Hand, &mut Transform), (With<Picked>, Without<CardPlaceholder>)>,
) {
    for ev in ev_pick.iter() {
        if let Ok((picked_entity, hand, mut transform)) = q_picked.get_single_mut() {
            if let PickingEvent::Selection(SelectionEvent::JustSelected(e)) = ev {
                if let Ok((placeholder_transform, mut material)) = q_placeholder.get_mut(*e) {
                    *material = placeholder_materials.invisable.clone();
                    transform.translation = placeholder_transform.translation;

                    let index = hand.0;

                    ev_played.send(CardPlayedEvent {
                        entity: picked_entity,
                        index,
                    });
                    commands.entity(picked_entity).remove::<Picked>();
                    commands.entity(picked_entity).remove::<Hand>();
                }
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
    commands.spawn(PbrBundle {
        mesh: board_assets.mesh.clone(),
        material: board_assets.material.clone(),
        ..default()
    });

    let card_padding = 1.0;
    let y = BOARD_HEIGHT + CARD_HALF_THICKNESS;
    let z_start = 2.0;

    for z in 0..2 {
        for x in 0..4 {
            let z = z_start - z as f32 * (CARD_HEIGHT + card_padding);
            let x = x as f32 * (CARD_WIDTH + card_padding) - 4.5;

            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: card_assets.card_mesh.clone(),
                        material: placeholder_materials.invisable.clone(),
                        transform: Transform::from_xyz(x, y, z),
                        ..default()
                    },
                    CardPlaceholder,
                ))
                .id();

            if z == z_start {
                commands.entity(entity).insert(PickableBundle::default());
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
        ));
    }

    state.set(GameState::PlayerTurn);
}
