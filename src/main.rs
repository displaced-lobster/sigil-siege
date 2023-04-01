use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{
    DefaultPickingPlugins,
    HoverEvent,
    PickableBundle,
    PickingCameraBundle,
    PickingEvent,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
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

    let card_height = 3.0;
    let card_width = 2.0;
    let card_padding = 1.0;

    for z in 0..2 {
        for x in 0..4 {
            let z = -(z as f32 * (card_height + card_padding) - 3.5);
            let x = x as f32 * (card_width + card_padding) - 5.5;

            let entity = commands
                .spawn((
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: invisable_material.clone(),
                        transform: Transform::from_xyz(x, 0.0, z),
                        ..default()
                    },
                    CardPlaceholder,
                ))
                .id();

            if z == 3.5 {
                commands.entity(entity).insert(PickableBundle::default());
            }
        }
    }

    let board_thickness = 0.25;
    let card_thickness = 0.05;

    for i in 0..10 {
        let y = i as f32 * card_thickness - board_thickness;

        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(),
                transform: Transform::from_xyz(7.5, y, 5.5),
                ..default()
            },
            Deck,
        ));
    }

    for i in 0..3 {
        let x = i as f32 * card_width - 5.5;

        commands.spawn((
            PbrBundle {
                mesh: mesh.clone(),
                transform: Transform::from_xyz(x, 0.0, 7.5),
                ..default()
            },
            Hand,
            PickableBundle::default(),
        ));
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
) {
    for ev in ev_pick.iter() {
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
    mut q_hand: Query<&mut Transform, With<Hand>>,
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
                    transform.translation.y = 0.0;
                }
            }
            _ => {}
        }
    }
}
