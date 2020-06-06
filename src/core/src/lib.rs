use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};

mod shader;

use web_sys::{
    window, HtmlCanvasElement, WebGlBuffer, WebGlRenderingContext,
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    fn alert(s: &str);
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .expect("no global window?!")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
pub struct Core {
    app: Rc<RefCell<App>>,
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        log("WASM Started");

        let app = App::new();
        match app {
            Ok(app) => {
                let app = Rc::new(RefCell::new(app));
                log("App Created");
                Self { app }
            }
            Err(err) => {
                log(&format!("{:?}", &err));
                alert("Failed to create app");
                panic!("Failed to create app");
            }
        }
    }

    #[wasm_bindgen]
    pub fn start(&mut self) {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let app = self.app.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // Set the body's text content to how many times this
            // requestAnimationFrame callback has fired.
            app.borrow_mut().update();

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
        log("App Started");
    }
}


struct Buffers {
    position: WebGlBuffer,
}

struct App {
    gl_context: WebGlRenderingContext,
    shader_info: shader::SdfShader,
    buffers: Buffers,
}

fn upload_array_f32(gl: &WebGlRenderingContext, vertices: Vec<f32>) {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap()
        .buffer();

    let vertices_location = vertices.as_ptr() as u32 / 4;

    let vert_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(vertices_location, vertices_location + vertices.len() as u32);

    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vert_array,
        WebGlRenderingContext::STATIC_DRAW,
    );
}

fn init_buffers(gl: &WebGlRenderingContext) -> Result<Buffers, wasm_bindgen::JsValue> {
    let position_buffer = gl
        .create_buffer()
        .expect("Failed to create position buffer");

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    let positions: Vec<f32> = vec![-1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];

    upload_array_f32(gl, positions);

    Ok(Buffers {
        position: position_buffer,
    })
}

#[derive(Debug)]
enum AppError {
    MiscJsError(JsValue),
    NoCanvas,
    NoViewport,
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

fn get_canvas(selector: &str) -> Result<Option<HtmlCanvasElement>, JsValue> {
    let window = window().unwrap();
    let document = window.document().unwrap();

    let canvas: HtmlCanvasElement = match document.query_selector(selector)? {
        Some(container) => container.dyn_into()?,
        None => {
            return Ok(None);
        }
    };
    Ok(Some(canvas))
}

fn get_gl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, JsValue> {
    Ok(canvas.get_context("webgl")?.unwrap().dyn_into()?)
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let canvas = match get_canvas("#viewport_3d")? {
            Some(canvas) => canvas,
            None => {
                return Err(AppError::NoCanvas);
            }
        };

        canvas.set_width(512 as u32);
        canvas.set_height(512 as u32);

        let gl: WebGlRenderingContext = get_gl_context(&canvas)?;
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.enable(WebGlRenderingContext::DEPTH_TEST);

        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

        if gl.is_null() {
            return Err(AppError::NoWebGl);
        }

		let shader_info = shader::SdfShader::new(&gl)?;
		let buffers = init_buffers(&gl)?;

        Ok(Self {
            gl_context: gl,
            shader_info: shader_info,
            buffers,
        })
    }

    pub fn update(&mut self) {
        self.gl_context.clear(
            WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT,
        );

        {
            let num_components = 2; // pull out 2 values per iteration
            let data_type = WebGlRenderingContext::FLOAT; // the data in the buffer is 32bit floats
            let normalize = false; // don't normalize
            let stride = 0; // how many bytes to get from one set of values to the next
                            // 0 = use type and numComponents above
            let offset = 0; // how many bytes inside the buffer to start from
            self.gl_context.bind_buffer(
                WebGlRenderingContext::ARRAY_BUFFER,
                Some(&self.buffers.position),
            );
            self.gl_context.vertex_attrib_pointer_with_i32(
                self.shader_info.attrib_vertex_positions,
                num_components,
                data_type,
                normalize,
                stride,
                offset,
            );
            self.gl_context
                .enable_vertex_attrib_array(self.shader_info.attrib_vertex_positions);
        }

        // Tell WebGL to use our program when drawing
        self.gl_context.use_program(Some(&self.shader_info.program));

        {
            let offset = 0;
            let vertex_count = 4;
            self.gl_context.draw_arrays(
                WebGlRenderingContext::TRIANGLE_STRIP,
                offset,
                vertex_count,
            );
        }
    }
}
