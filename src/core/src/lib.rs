mod canvas;

use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

use web_sys::{
	WebGlRenderingContext,
	WebGlShader,
	WebGlProgram
};


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;



fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}


fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
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
		Self {
			app: Rc::new(RefCell::new(App::new()))
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

struct App {
	counter: u64,
	gl_context: WebGlRenderingContext,
}


fn load_shader(gl: &WebGlRenderingContext, shader_type: u32, shader_text: &str) -> Result<WebGlShader, ()>{
	let shader = gl.create_shader(shader_type).expect("Failed to create shader");
	gl.shader_source(&shader, shader_text);
	gl.compile_shader(&shader);
	if !gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS).is_truthy() {
		gl.delete_shader(Some(&shader));
		return Err(());
	}
	Ok(shader)
}

fn init_shader_program(gl: &WebGlRenderingContext, vert_source: &str, frag_source: &str) -> Result<WebGlProgram, ()> {
	let vert_shader = load_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vert_source)?;
	let frag_shader = load_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, frag_source)?;

	let shader_program = gl.create_program().expect("Failed to create shader program");
	gl.attach_shader(&shader_program, &vert_shader);
	gl.attach_shader(&shader_program, &frag_shader);
	
	gl.link_program(&shader_program);
	
	if !(gl.get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)) .is_truthy() {
		gl.delete_program(Some(&shader_program));
		gl.delete_shader(Some(&vert_shader));
		gl.delete_shader(Some(&frag_shader));
		return Err(());
	}
	
	Ok(shader_program)
}


impl App {
	pub fn new() -> Self {
		Self {
			counter: 0,
			gl_context: canvas::create_webgl_context().expect("failed to create gl context"),
		}
	}
	
	pub fn update(&mut self) {
		if self.counter == 0{
			// TODO: think of a better way. Probably put this in new?
			let shader_program = init_shader_program(
					&self.gl_context, 
					include_str!("shaders/simple.vert"),
					include_str!("shaders/simple.frag")
			);
		}
		self.counter += 1;
		
		self.gl_context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
	}
}

