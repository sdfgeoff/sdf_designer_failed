//~ use crate::app::App;
//~ use crate::app::Msg;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
	HtmlElement,
	HtmlCanvasElement,
	window,
	WebGlRenderingContext
};

pub static APP_DIV_ID: &'static str = "viewport_3d";

pub static CANVAS_WIDTH: i32 = 512;
pub static CANVAS_HEIGHT: i32 = 512;

pub fn create_webgl_context() -> Result<WebGlRenderingContext, JsValue> {
    let canvas = init_canvas()?;

    let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;

    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.enable(WebGlRenderingContext::DEPTH_TEST);

    Ok(gl)
}

fn init_canvas() -> Result<HtmlCanvasElement, JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas: HtmlCanvasElement = document.create_element("canvas").unwrap().dyn_into()?;

    canvas.set_width(CANVAS_WIDTH as u32);
    canvas.set_height(CANVAS_HEIGHT as u32);

    let app_div: HtmlElement = match document.get_element_by_id(APP_DIV_ID) {
        Some(container) => container.dyn_into()?,
        None => {
			panic!();            
        }
    };

    app_div.append_child(&canvas)?;
    Ok(canvas)
}
