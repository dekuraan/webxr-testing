use std::rc::Rc;
use std::sync::Arc;

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    render::{
        camera::Viewport,
        renderer::{RenderAdapterInfo, RenderDevice, RenderQueue},
    },
    webxr::{initialize_webxr, Canvas, FramebufferUuid, InitializedState, WebXrContext},
};
use gloo_console as console;
use wasm_bindgen_futures::spawn_local;
use wgpu::{Adapter, Device, Queue};

pub fn main() {
    console::log!("main");
    spawn_local(async {
        start().await;
    });
}

pub async fn start() {
    let mut app = App::new();
    app.insert_resource(initialize_webxr().await);
    app.add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()));
    app.add_system(rotate);
    app.add_startup_system(setup);
    app.run();
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    frame: NonSend<web_sys::XrFrame>,
    id: Res<FramebufferUuid>,
) {
    let id = id.0;
    let base_layer: web_sys::XrWebGlLayer = frame.session().render_state().base_layer().unwrap();
    let resolution = UVec2::new(
        base_layer.framebuffer_width(),
        base_layer.framebuffer_height(),
    );
    let physical_size = UVec2::new(resolution.x / 2, resolution.y);
    let left_viewport = Viewport {
        physical_position: UVec2::ZERO,
        physical_size,
        ..default()
    };
    let right_viewport = Viewport {
        physical_position: UVec2::new(resolution.x / 2, 0),
        physical_size,
        ..default()
    };

    //left camera
    commands.spawn((Camera3dBundle {
        camera_3d: Camera3d { ..default() },
        camera: Camera {
            target: RenderTarget::TextureView(id),
            viewport: Some(left_viewport),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    //right camera
    commands.spawn((Camera3dBundle {
        camera_3d: Camera3d {
            //Viewport does not affect ClearColor, so we set the right camera to a None Clear Color
            clear_color: ClearColorConfig::None,
            ..default()
        },
        camera: Camera {
            target: RenderTarget::TextureView(id),
            priority: 1,
            viewport: Some(right_viewport),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });

    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    //cube
    commands.spawn(PbrBundle {
        mesh: cube_handle,
        material: debug_material.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 1.5)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 5.0)),
        ..default()
    });
}

fn rotate(mut mesh_q: Query<(&mut Transform), (With<Handle<Mesh>>)>, time: Res<Time>) {
    for mut tf in &mut mesh_q {
        tf.rotate_y(time.delta_seconds() / 2.);
    }
}

fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
    )
}
