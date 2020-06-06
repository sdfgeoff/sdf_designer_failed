use wasm_bindgen::prelude::wasm_bindgen;
/// This is the first layer of state in the application
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use crate::full_screen_quad;
use crate::shader;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn alert(s: &str);
}

pub struct App {
    canvas: HtmlCanvasElement,
    gl_context: WebGlRenderingContext,
    sdf_shader: shader::SdfShader,
    quad: full_screen_quad::FullScreenQuad,

    width: u32,
    height: u32,
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
            sdf_shader: shader_info,
            quad: buffers,
            canvas,
            width: 0,
            height: 0,
        })
    }

    fn check_resize(&mut self) {
        let width = self.canvas.client_width() as u32;
        let height = self.canvas.client_height() as u32;
        if width != self.width || self.height != self.height {
            self.width = width;
            self.height = height;
            self.canvas.set_width(self.width);
            self.canvas.set_height(self.height);
            self.gl_context
                .viewport(0, 0, self.width as i32, self.height as i32);
            log(&format!("Resized to {}:{}", self.width, self.height));
        }
    }

    pub fn update(&mut self) {
        self.check_resize();

        self.gl_context.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        // Tell WebGL to use our program when drawing
        self.gl_context.use_program(Some(&self.sdf_shader.program));
        self.sdf_shader
            .set_resolution(&self.gl_context, self.width, self.height);

        self.quad.render(&self.gl_context, &self.sdf_shader)
    }
}
