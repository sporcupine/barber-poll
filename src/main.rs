use std::f32::consts::PI;

use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::VertexAttributeValues,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

mod ui;
use ui::PollParams;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            EguiPlugin::default(),
        ))
        .init_resource::<PollParams>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (rotate, update_poll_mesh, update_poll_texture, update_camera),
        )
        .add_systems(EguiPrimaryContextPass, ui::ui)
        .insert_resource(ClearColor(Color::srgb(1., 1., 1.)))
        .run();
}

#[derive(Component)]
struct BarberPoll;

#[derive(Resource)]
struct PollTexture(Handle<Image>);

const LIGHT_Z: f32 = 14.0;
const SURFACE_DISTANCE: f32 = 7.0;

fn poll_z(radius: f32) -> f32 {
    LIGHT_Z - SURFACE_DISTANCE - radius
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    params: Res<PollParams>,
) {
    let texture = images.add(stripe_texture());
    commands.insert_resource(PollTexture(texture.clone()));

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        ..default()
    });

    let poll = build_poll_mesh(params.radius, params.height, params.n, params.angle);

    commands.spawn((
        Mesh3d(meshes.add(poll)),
        MeshMaterial3d(debug_material.clone()),
        Transform::from_xyz(0.0, 0.0, poll_z(params.radius)),
        BarberPoll,
    ));

    commands.spawn((
        PointLight {
            shadow_maps_enabled: false,
            intensity: 5_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, LIGHT_Z),
    ));

    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, LIGHT_Z),
    ));
}

fn rotate(
    mut query: Query<&mut Transform, With<BarberPoll>>,
    time: Res<Time>,
    params: Res<PollParams>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * params.speed / 2.);
    }
}

fn update_camera(params: Res<PollParams>, mut query: Query<&mut Projection>) {
    if !params.is_changed() {
        return;
    }
    for mut proj in &mut query {
        if params.orth {
            *proj = Projection::from(OrthographicProjection {
                scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                    viewport_height: 9.0,
                },
                ..OrthographicProjection::default_3d()
            })
        } else {
            *proj = Projection::from(PerspectiveProjection::default())
        }
    }
}

fn update_poll_mesh(
    params: Res<PollParams>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<(&Mesh3d, &mut Transform), With<BarberPoll>>,
) {
    if !params.is_changed() {
        return;
    }
    for (mesh_handle, mut transform) in &mut query {
        if let Some(mut mesh) = meshes.get_mut(&mesh_handle.0) {
            *mesh = build_poll_mesh(params.radius, params.height, params.n, params.angle);
        }
        transform.translation.z = poll_z(params.radius);
    }
}

fn update_poll_texture(
    params: Res<PollParams>,
    texture: Res<PollTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if !params.is_changed() {
        return;
    }
    if let Some(mut image) = images.get_mut(&texture.0) {
        *image = stripe_texture();
    }
}

fn stripe_texture() -> Image {
    const W: usize = 256;
    const BASE: [u8; 4] = [255, 255, 255, 255];
    const STRIPE: [u8; 4] = [230, 20, 20, 255];

    let mut data = Vec::with_capacity(W * 4);
    for x in 0..W {
        data.extend_from_slice(if x < W / 2 { &STRIPE } else { &BASE });
    }

    let mut image = Image::new(
        Extent3d {
            width: W as u32,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::ClampToEdge,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn build_poll_mesh(radius: f32, height: f32, n: usize, angle_deg: f32) -> Mesh {
    let mut mesh: Mesh = Cylinder::new(radius, height).into();

    let positions: Vec<[f32; 3]> = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|a| a.as_float3())
        .map(|p| p.to_vec())
        .unwrap_or_default();
    let normals: Vec<[f32; 3]> = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .and_then(|a| a.as_float3())
        .map(|n| n.to_vec())
        .unwrap_or_default();

    let shear = angle_deg.to_radians().tan() * n as f32 / (2. * PI * radius);

    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for ((uv, normal), pos) in uvs.iter_mut().zip(&normals).zip(&positions) {
            if normal[1].abs() < 0.5 {
                uv[0] = uv[0] * n as f32 + pos[1] * shear;
                uv[1] = 0.5;
            }
        }
    }
    mesh
}
