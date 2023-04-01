use bevy::{input::common_conditions::input_toggle_active, prelude::*};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(
            WorldInspectorPlugin::default().run_if(input_toggle_active(true, KeyCode::Escape)),
        )
        .insert_resource(ClearColor(Color::rgb(0.06, 0.06, 0.08)))
        .insert_resource(AmbientLight {
            brightness: 1.0,
            color: Color::rgb(0.63, 0.51, 0.51),
        })
        .add_startup_system(setup)
        .run();
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

    for z in 0..2 {
        for x in 0..4 {
            let z = -(z as f32 * 4.0 - 3.5);
            let x = x as f32 * 3.0 - 5.5;

            commands.spawn(PbrBundle {
                mesh: mesh.clone(),
                transform: Transform::from_xyz(x, 0.0, z),
                ..default()
            });
        }
    }

    let board_thickness = 0.25;
    let card_thickness = 0.05;

    for i in 0..10 {
        let y = i as f32 * card_thickness - board_thickness;

        commands.spawn(PbrBundle {
            mesh: mesh.clone(),
            transform: Transform::from_xyz(7.5, y, 5.5),
            ..default()
        });
    }

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
