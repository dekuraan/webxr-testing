use std::{cell::RefCell, rc::Rc};

use gloo_console::log;
use gloo_utils::window;
use js_sys::{Boolean, Object, Reflect};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, XrFrame, XrRenderStateInit,
    XrSession, XrSessionMode, XrWebGlLayer,
};

pub fn main() {
    console_error_panic_hook::set_once();
    spawn_local(async {
        start().await.unwrap_throw();
    });
}

pub async fn start() -> Result<(), JsValue> {
    let window = window();
    let navigator = window.navigator();
    let xr_system = navigator.xr();
    let session_supported =
        JsFuture::from(xr_system.is_session_supported(XrSessionMode::ImmersiveVr))
            .await?
            .dyn_into::<Boolean>()?
            .value_of();
    {
        log!("supported", session_supported)
    };

    let session: XrSession = JsFuture::from(xr_system.request_session(XrSessionMode::ImmersiveVr))
        .await?
        .dyn_into::<XrSession>()?;

    let document = window.document().unwrap();

    let canvas = document
        .get_element_by_id("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();

    let gl_attribs = Object::new();

    // satisfying rust-analyzer proc macro
    Reflect::set(
        &gl_attribs,
        &JsValue::from_str("xrCompatible"),
        &JsValue::TRUE,
    )
    .unwrap();

    let gl = Rc::new(
        canvas
            .get_context_with_context_options("webgl2", &gl_attribs)?
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()?,
    );

    let mut state = XrRenderStateInit::new();

    let base_layer = XrWebGlLayer::new_with_web_gl2_rendering_context(&session, &gl)?;

    state.base_layer(Some(&base_layer));

    session.update_render_state_with_state(&state);

    //Draw

    type XrFrameHandler = Closure<dyn FnMut(f64, XrFrame)>;
    let session = Rc::new(RefCell::new(session));
    let f: Rc<RefCell<Option<XrFrameHandler>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    let closure_session = session.clone();
    *g.borrow_mut() = Some(Closure::new(move |_time: f64, _frame: XrFrame| {
        let session = closure_session.borrow();

        let base_layer = session.render_state().base_layer().unwrap();
        //Reflect hack
        let framebuffer: WebGlFramebuffer =
            js_sys::Reflect::get(&base_layer, &"framebuffer".into())
                .unwrap()
                .into();
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        gl.clear_color(1.0, 1.0, 0.0, 1.0);
        gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        session.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    }));

    session
        .borrow()
        .request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref());
    Ok(())
}
