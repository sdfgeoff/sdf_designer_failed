use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast, JsValue};
use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;


use web_sys::{
	WebGlRenderingContext,
	WebGlShader,
	WebGlBuffer,
	WebGlProgram,
	WebGlUniformLocation,
	HtmlCanvasElement,
	
	window
};


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
extern {
    fn alert(s: &str);
}



fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window().expect("no global window?!")
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}


#[wasm_bindgen]
pub struct Core {
	app: Rc<RefCell<App>>
}


#[wasm_bindgen]
impl Core {
	#[wasm_bindgen(constructor)]
	pub fn new() -> Self {
		//body().set_text_content(Some("WASM Started"));
		let app = App::new().unwrap_or_else(|err| {alert(&err.as_string().unwrap()); panic!("Failed to create app")});
		Self {
			app: Rc::new(RefCell::new(app))
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
	}
}


struct ShaderInfo {
	program: WebGlProgram,
	attrib_vertex_positions: u32,
}

struct Buffers {
	position: WebGlBuffer,
}

struct App {
	gl_context: WebGlRenderingContext,
	shader_info: ShaderInfo,
	buffers: Buffers
}


fn load_shader(gl: &WebGlRenderingContext, shader_type: u32, shader_text: &str) -> Result<WebGlShader, wasm_bindgen::JsValue>{
	let shader = gl.create_shader(shader_type).expect("Failed to create shader");
	gl.shader_source(&shader, shader_text);
	gl.compile_shader(&shader);
	if !gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).is_truthy() {
		alert(&gl.get_shader_info_log(&shader).expect("No error?"));
		gl.delete_shader(Some(&shader));
		return Err(JsValue::from_str("Shader compile error"));
	}
	Ok(shader)
}

fn init_shader_program(gl: &WebGlRenderingContext, vert_source: &str, frag_source: &str) -> Result<WebGlProgram, wasm_bindgen::JsValue> {
	let vert_shader = load_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vert_source)?;
	let frag_shader = load_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, frag_source)?;

	let shader_program = gl.create_program().expect("Failed to create shader program");
	gl.attach_shader(&shader_program, &vert_shader);
	gl.attach_shader(&shader_program, &frag_shader);
	
	gl.link_program(&shader_program);
	
	if !(gl.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)).is_truthy() {
		gl.delete_program(Some(&shader_program));
		gl.delete_shader(Some(&vert_shader));
		gl.delete_shader(Some(&frag_shader));
		return Err(JsValue::from_str("Failed to create shader program"));
	}
	
	Ok(shader_program)
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
	let position_buffer = gl.create_buffer().expect("Failed to create position buffer");
	
	gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
	
	let positions: Vec<f32> = vec!(
		-1.0, 1.0,
		1.0, 1.0,
		-1.0, -1.0,
		1.0, -1.0
	);
	
	upload_array_f32(gl, positions);
	
	
	Ok(Buffers {
		position: position_buffer
	})
}

impl App {
	pub fn new() -> Result<Self, wasm_bindgen::JsValue> {
		
		let window = window().unwrap();
		let document = window.document().unwrap();
		
		let canvas: HtmlCanvasElement = match document.query_selector("#viewport_3d")? {
			Some(container) => container.dyn_into()?,
			None => {
				return Err("No Canvas".into());            
			}
		};
		canvas.set_width(512 as u32);
		canvas.set_height(512 as u32);
		
		let gl: WebGlRenderingContext = canvas.get_context("webgl")?.unwrap().dyn_into()?;
		gl.clear_color(0.0, 0.0, 0.0, 1.0);
		gl.enable(WebGlRenderingContext::DEPTH_TEST);
		
		gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
		
		
		if gl.is_null() {
			return Err("No Webgl".into());
		}
		
		let shader_program = init_shader_program(
				&gl, 
				include_str!("shaders/simple.vert"),
				include_str!("shaders/simple.frag")
		)?;
		
		let buffers = init_buffers(&gl)?;
		
		let attrib_vertex_positions = gl.get_attrib_location(&shader_program, "aVertexPosition");

		let shader_info = ShaderInfo {
			program: shader_program,
			attrib_vertex_positions: attrib_vertex_positions as u32
		};
		
		Ok(Self {
			gl_context: gl,
			shader_info: shader_info,
			buffers,
		})
	}
	
	pub fn update(&mut self) {
		self.gl_context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
		
		let canvas: HtmlCanvasElement = self.gl_context.canvas().expect("Canvas????").dyn_into().unwrap();
		
		{
			let num_components = 2;  // pull out 2 values per iteration
			let data_type = WebGlRenderingContext::FLOAT;    // the data in the buffer is 32bit floats
			let normalize = false;  // don't normalize
			let stride = 0;         // how many bytes to get from one set of values to the next
									  // 0 = use type and numComponents above
			let offset = 0;         // how many bytes inside the buffer to start from
			self.gl_context.bind_buffer(
				WebGlRenderingContext::ARRAY_BUFFER,
				Some(&self.buffers.position)
			);
			self.gl_context.vertex_attrib_pointer_with_i32(
				self.shader_info.attrib_vertex_positions,
				num_components,
				data_type,
				normalize,
				stride,
				offset);
			self.gl_context.enable_vertex_attrib_array(
				self.shader_info.attrib_vertex_positions);
		}

		// Tell WebGL to use our program when drawing
		self.gl_context.use_program(Some(&self.shader_info.program));

		{
			let offset = 0;
			let vertex_count = 4;
			self.gl_context.draw_arrays(WebGlRenderingContext::TRIANGLE_STRIP, offset, vertex_count);
		}
	}
}
