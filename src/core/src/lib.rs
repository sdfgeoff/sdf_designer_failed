/// This file handles bindings from the DOM to the application

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::{JsCast};

mod shader;
mod full_screen_quad;
mod app;

use web_sys::{
    window, HtmlElement, HtmlCanvasElement,
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
    app: Rc<RefCell<app::App>>,
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        log("WASM Started");
        
        let window = window().unwrap();
		let document = window.document().unwrap();
        
        
		let canvas: HtmlCanvasElement = match document.query_selector("#viewport_3d").unwrap() {
			Some(container) => container.dyn_into().unwrap(),
			None => {
				log(&format!("No Canvas"));
                alert("No Canvas");
                panic!("Failed to create app");
			}
		};
		
		let overlay: HtmlElement = match document.query_selector("#overlay").unwrap() {
			Some(container) => container.dyn_into().unwrap(),
			None => {
				log(&format!("No Overlay"));
                alert("No Overlay");
                panic!("Failed to create overlay");
			}
		};
		overlay.set_inner_text(""); // Clear loading spinner

        match app::App::new(canvas) {
            Ok(ap) => {
                log("App Created");
                
                let ap = Rc::new(RefCell::new(ap));
                
                // Set up bindings
                
                
                Self { app: ap }
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

        let ap = self.app.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // Set the body's text content to how many times this
            // requestAnimationFrame callback has fired.
            ap.borrow_mut().update();

            // Schedule ourself for another requestAnimationFrame callback.
            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
        log("App Started");
    }
}


