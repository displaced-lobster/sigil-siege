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

const ATTRIBUTE_HEART_OFFSET: f32 = 1.4;
const ATTRIBUTE_SCALE: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const ATTRIBUTE_SWORD_OFFSET: f32 = 1.0;
const ATTRIBUTE_WIDTH: f32 = 0.4;
const ATTRIBUTE_X_OFFSET: f32 = 0.6;
const BOARD_HEIGHT: f32 = 0.25;
const CARD_THICKNESS: f32 = 0.05;
const CARD_HALF_THICKNESS: f32 = CARD_THICKNESS / 2.0;
const CARD_HEIGHT: f32 = 3.0;
const CARD_WIDTH: f32 = 2.0;

const HAND_Z: f32 = 6.0;

fn main() {
    App::new()
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
        .add_system(hover_card_placeholder)
        .add_system(hover_hand)
        .add_system(pick_from_hand)
        .add_system(play_card)
        .run();
}

#[derive(Component)]
struct CardPlaceholder;

#[derive(Resource)]
struct CardPlaceholderMaterials {
    invisable: Handle<StandardMaterial>,
    hovered: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Deck;

#[derive(Component)]
struct Hand;

#[derive(Component)]
struct Picked;

struct Attributes {
    attack: u32,
    health: u32,
}

enum CardType {
    Heart,
    Pitchfork,
    Sword,
    Tower,
}

impl CardType {
    fn attributes(&self) -> Attributes {
        match self {
            Self::Heart => Attributes {
                attack: 1,
                health: 1,
            },
            Self::Pitchfork => Attributes {
                attack: 1,
                health: 1,
            },
            Self::Sword => Attributes {
                attack: 2,
                health: 2,
            },
            Self::Tower => Attributes {
                attack: 1,
                health: 3,
            },
        }
    }
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

    commands.spawn(PbrBundle {
        mesh,
        material,
        ..default()
    });

    let mesh = asset_server.load("models/card.glb#Mesh0/Primitive0");
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
        invisable: invisable_material.clone(),
        hovered: hovered_material,
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
                        mesh: mesh.clone(),
                        material: invisable_material.clone(),
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

    for i in 0..10 {
        let y = i as f32 * CARD_THICKNESS + CARD_HALF_THICKNESS;

        commands.spawn((
            PbrBundle {
                mesh: card_mesh.clone(),
                material: card_material.clone(),
                transform: Transform::from_xyz(8.0, y, 5.0)
                    .with_rotation(Quat::from_rotation_z(180.0_f32.to_radians())),
                ..default()
            },
            Deck,
        ));
    }

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

    let mesh_materials = [
        (heart_mesh.clone(), heart_material.clone(), CardType::Heart),
        (pitchfork_mesh, black_material.clone(), CardType::Pitchfork),
        (sword_mesh.clone(), black_material.clone(), CardType::Sword),
        (tower_mesh, black_material.clone(), CardType::Tower),
    ];

    for (i, (mesh, material, card_type)) in mesh_materials.iter().enumerate() {
        let x = i as f32 * CARD_WIDTH - 5.0;

        commands
            .spawn((
                PbrBundle {
                    mesh: card_mesh.clone(),
                    material: card_material.clone(),
                    transform: Transform::from_xyz(x, CARD_HALF_THICKNESS, HAND_Z),
                    ..default()
                },
                Hand,
                PickableBundle::default(),
            ))
            .with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    ..default()
                });

                let attributes = card_type.attributes();

                for i in 0..attributes.health {
                    let x = i as f32 * ATTRIBUTE_WIDTH - ATTRIBUTE_X_OFFSET;

                    parent.spawn(PbrBundle {
                        mesh: heart_mesh.clone(),
                        material: heart_material.clone(),
                        transform: Transform::from_xyz(x, 0.0, ATTRIBUTE_HEART_OFFSET)
                            .with_scale(ATTRIBUTE_SCALE),
                        ..default()
                    });
                }

                for i in 0..attributes.attack {
                    let x = i as f32 * ATTRIBUTE_WIDTH - ATTRIBUTE_X_OFFSET;

                    parent.spawn(PbrBundle {
                        mesh: sword_mesh.clone(),
                        material: black_material.clone(),
                        transform: Transform::from_xyz(x, BOARD_HEIGHT, ATTRIBUTE_SWORD_OFFSET)
                            .with_scale(ATTRIBUTE_SCALE),
                        ..default()
                    });
                }
            });
    }

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PickingCameraBundle::default(),
    ));
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
    mut q_placeholder: Query<(&Transform, &mut Handle<StandardMaterial>), With<CardPlaceholder>>,
    mut q_picked: Query<(Entity, &mut Transform), (With<Picked>, Without<CardPlaceholder>)>,
) {
    for ev in ev_pick.iter() {
        if let Ok((picked_entity, mut transform)) = q_picked.get_single_mut() {
            if let PickingEvent::Selection(SelectionEvent::JustSelected(e)) = ev {
                if let Ok((placeholder_transform, mut material)) = q_placeholder.get_mut(*e) {
                    *material = placeholder_materials.invisable.clone();
                    transform.translation = placeholder_transform.translation;

                    commands.entity(picked_entity).remove::<Picked>();
                    commands.entity(picked_entity).remove::<Hand>();
                }
            }
        }
    }
}
