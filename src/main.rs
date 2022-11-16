use std::rc::Rc;
use std::sync::Arc;

use bevy::{
    prelude::*,
    render::renderer::RenderDevice,
    webxr::{Canvas, WebXrContext},
};
use gloo_console as console;
use wasm_bindgen_futures::spawn_local;

pub fn main() {
    console_error_panic_hook::set_once();
    console::log!("main");
    spawn_local(async {
        start().await;
    });
}

pub async fn start() {
    console::log!("start");
    let mut app = App::new();

    let webxr_context = WebXrContext::get_context(bevy::xr::XrSessionMode::ImmersiveVR)
        .await
        .unwrap();

    let gl = webxr_context.canvas.create_webgl2_context();

    let mut state = web_sys::XrRenderStateInit::new();

    let base_layer = web_sys::XrWebGlLayer::new_with_web_gl2_rendering_context(
        &webxr_context.session.borrow(),
        &gl,
    )
    .unwrap();

    state.base_layer(Some(&base_layer));

    webxr_context
        .session
        .borrow()
        .update_render_state_with_state(&state);

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

    app.world
        .insert_resource(RenderDevice::from(Arc::new(device)));
    app.world.insert_resource(Arc::new(queue));
    app.world.insert_resource(adapter_info);
    app.world.insert_non_send_resource(webxr_context);
    app.add_plugins(DefaultPlugins);
    app.add_system(running_test);
    app.run();
}

fn running_test() {
    bevy::log::info!("Hiii!!!!")
}
