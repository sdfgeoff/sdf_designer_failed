/// This is the first layer of state in the application
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, WebGlRenderingContext,
};

use crate::full_screen_quad;
use crate::shader;


pub struct App {
	canvas: HtmlCanvasElement,
    gl_context: WebGlRenderingContext,
    shader_info: shader::SdfShader,
    buffers: full_screen_quad::FullScreenQuad,
}


#[derive(Debug)]
pub enum AppError {
    MiscJsError(JsValue),
    NoWebGl,
    ShaderError(shader::ShaderError),
}

impl From<JsValue> for AppError {
    fn from(val: JsValue) -> AppError {
        AppError::MiscJsError(val)
    }
}

impl From<shader::ShaderError> for AppError {
    fn from(val: shader::ShaderError) -> AppError {
        AppError::ShaderError(val)
    }
}


fn get_gl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, JsValue> {
    Ok(canvas.get_context("webgl")?.unwrap().dyn_into()?)
}


impl App {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, AppError> {
        let gl: WebGlRenderingContext = get_gl_context(&canvas)?;
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.enable(WebGlRenderingContext::DEPTH_TEST);

        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

        if gl.is_null() {
            return Err(AppError::NoWebGl);
        }

		let shader_info = shader::SdfShader::new(&gl)?;
		let buffers = full_screen_quad::FullScreenQuad::new(&gl)?;

        Ok(Self {
            gl_context: gl,
            shader_info: shader_info,
            buffers,
            canvas
        })
    }

    pub fn update(&mut self) {
        self.gl_context.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        // Tell WebGL to use our program when drawing
        self.gl_context.use_program(Some(&self.shader_info.program));

        self.buffers.render(&self.gl_context, &self.shader_info)
    }
}
