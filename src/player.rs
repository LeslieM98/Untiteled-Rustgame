use crate::actor::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerMarker;
#[derive(Component)]
struct CameraBaseMarker;
#[derive(Component)]
struct PlayerCameraMarker;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub actor: Actor,
    pub player: PlayerMarker,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            actor: Default::default(),
            player: PlayerMarker,
        }
    }
}

pub fn move_player(
    mut query: Query<&mut Transform, With<PlayerMarker>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    const VELOCITY: f32 = 1.0;
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::W) {
        direction.x += 1.0;
    }
    if input.pressed(KeyCode::S) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::A) {
        direction.z -= 1.0;
    }
    if input.pressed(KeyCode::D) {
        direction.z += 1.0;
    }
    if input.pressed(KeyCode::Space) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::LControl) {
        direction.y -= 1.0;
    }

    if direction.length() > 0.001 {
        direction = direction.normalize();
        let mut transform = query.get_single_mut().unwrap();
        transform.translation += direction * VELOCITY * time.delta_seconds();
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tex_handle = asset_server.load("PNG/Green/texture_04.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(tex_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: false,
        ..default()
    });

    let pbr = PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
        material: material_handle,
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    };
    let player_bundle = PlayerBundle {
        actor: Actor { pbr, ..default() },
        ..default()
    };

    // Camera Node
    let camera_base_transform = (
        Transform::default(),
        GlobalTransform::default(),
        CameraBaseMarker,
    );

    // Camera
    let camera = (
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PlayerCameraMarker,
    );

    let camera_entity = commands.spawn(camera).id();
    let camera_base_entity = commands
        .spawn(camera_base_transform)
        .add_child(camera_entity)
        .id();
    commands.spawn(player_bundle).add_child(camera_base_entity);
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_player).add_system(move_player);
    }
}
