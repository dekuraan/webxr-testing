use std::rc::Rc;
use std::sync::Arc;

use bevy::{
    prelude::*,
    render::renderer::{RenderAdapterInfo, RenderDevice, RenderQueue},
    webxr::{Canvas, WebXrContext},
};
use gloo_console as console;
use wasm_bindgen_futures::spawn_local;

pub fn main() {
    console::log!("main");
    spawn_local(async {
        // console_error_panic_hook::set_once();
        start().await;
    });
}

pub async fn start() {
    console::log!("start");
    let mut app = App::new();
    console::log!("post app creation");

    let webxr_context = WebXrContext::get_context(bevy::xr::XrSessionMode::ImmersiveVR)
        .await
        .unwrap();

    console::log!("post context");

    let webgl2_context = webxr_context.canvas.create_webgl2_context();

    console::log!("post gl");

    let mut layer_init = web_sys::XrWebGlLayerInit::new();

    // WGpu Setup
    let instance = wgpu::Instance::new(wgpu::Backends::GL);

    let surface = unsafe { instance.create_surface(&webxr_context.canvas) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("No suitable GPU adapters found on the system!");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("device"),
                features: adapter.features(),
                limits: adapter.limits(),
            },
            None,
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");
    let adapter_info = adapter.get_info();

    wasm_bindgen_futures::JsFuture::from(webgl2_context.make_xr_compatible())
        .await
        .expect("Failed to make the webgl context xr-compatible");

    let xr_gl_layer = web_sys::XrWebGlLayer::new_with_web_gl2_rendering_context_and_layer_init(
        &webxr_context.session,
        &webgl2_context,
        &layer_init,
    )
    .unwrap();

    let mut render_state_init = web_sys::XrRenderStateInit::new();
    render_state_init
        .depth_near(0.001)
        .base_layer(Some(&xr_gl_layer));

    webxr_context
        .session
        .update_render_state_with_state(&render_state_init);

    app.world
        .insert_resource(RenderDevice::from(Arc::new(device)));
    app.world.insert_resource(RenderQueue(Arc::new(queue)));
    app.world.insert_resource(RenderAdapterInfo(adapter_info));
    app.world.insert_non_send_resource(webxr_context);
    app.add_plugins(DefaultPlugins);
    app.add_system(running_test);
    console::log!("pre run");
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn((
        Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::BLUE),
                ..default()
            },
            camera: Camera {
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        first_pass_layer,
    ));
    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn(PbrBundle {
        mesh: cube_handle,
        material: material_handle,
        transform: Transform::from_xyz(0.0, 0.0, 1.5)
            .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
        ..default()
    });
}

fn running_test() {
    bevy::log::info!("Hiii!!!!")
}
