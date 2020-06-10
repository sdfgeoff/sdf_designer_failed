use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader, WebGlUniformLocation, window};


#[derive(Debug)]
pub enum ShaderError {
    ShaderAllocError,
    ShaderProgramAllocError,
    ShaderGetInfoError,
    MissingUniform(String),
    ShaderCompileError {
        shader_type: u32,
        compiler_output: String,
    },
    ShaderLinkError(),
}

pub trait Shader {
    fn get_attrib_position(&self) -> u32;
}

pub struct SdfShader {
    pub program: WebGlProgram,
    pub attrib_vertex_positions: u32,
    uniform_resolution: WebGlUniformLocation,
    uniform_scene_description: WebGlUniformLocation,
}

fn get_uniform_location(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
    name: &str,
) -> Result<WebGlUniformLocation, ShaderError> {
    gl.get_uniform_location(&program, name)
        .ok_or(ShaderError::MissingUniform(name.to_string()))
}



enum Instruction {
	Stop = 0,
	NewEntity = 1,
	
	Sphere = 100,
	Box = 101,
	
	Translate = 200,
	Rotate = 201,
	Scale = 202,
	
	Union = 300,
	Difference = 301,
	Intersect = 302,
	
	
}

impl From<Instruction> for f32 {
	fn from(val: Instruction) -> f32 {
		(val as i32) as f32
	}
}




impl SdfShader {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, ShaderError> {
        let program = init_shader_program(
            gl,
            include_str!("shaders/simple.vert"),
            include_str!("shaders/simple.frag"),
        )?;

        let attrib_vertex_positions = gl.get_attrib_location(&program, "aVertexPosition");
        let uniform_resolution = get_uniform_location(&gl, &program, "resolution")?;
        let uniform_scene_description = get_uniform_location(&gl, &program, "scene_description")?;

        Ok(Self {
            program,
            attrib_vertex_positions: attrib_vertex_positions as u32,
            uniform_resolution: uniform_resolution,
            uniform_scene_description: uniform_scene_description
        })
    }

    pub fn set_resolution(&self, gl: &WebGl2RenderingContext, x: u32, y: u32) {
        gl.uniform2f(Some(&self.uniform_resolution), x as f32, y as f32);
    }
    
    pub fn set_scene(&self, gl: &WebGl2RenderingContext) {
		
		let now = window().unwrap().performance().unwrap().now();
	
		gl.uniform1fv_with_f32_array(
			Some(&self.uniform_scene_description),
			&[
				Instruction::NewEntity.into(), // New Entity
				Instruction::Translate.into(), // translate
				0.0, // x
				0.0, // y
				-5.0, // z
				Instruction::Rotate.into(), // translate
				0.0, // x
				1.0, // y
				(now / 500.0).sin() as f32, // z
				Instruction::Box.into(), // sphere object
				1.0, // x
				0.5, // y
				1.0, // z
				Instruction::Union.into(), // union
				
				Instruction::NewEntity.into(), // New Entity
				Instruction::Translate.into(), // translate
				-1.0, // x
				0.0, // y
				-5.0, // z
				Instruction::Sphere.into(), // sphere object
				1.0, // radius
				Instruction::Union.into(), // union

				Instruction::NewEntity.into(), // New Entity
				Instruction::Translate.into(), // translate
				0.0, // x
				-1.0, // y
				-4.0, // z
				Instruction::Sphere.into(), // sphere object
				1.0, // radius
				Instruction::Difference.into(), // difference
				
				Instruction::Stop.into() // End
			]
		)
		
	}
}

impl Shader for SdfShader {
    fn get_attrib_position(&self) -> u32 {
        return self.attrib_vertex_positions;
    }
}

fn load_shader(
    gl: &WebGl2RenderingContext,
    shader_type: u32,
    shader_text: &str,
) -> Result<WebGlShader, ShaderError> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or(ShaderError::ShaderAllocError)?;
    gl.shader_source(&shader, shader_text);
    gl.compile_shader(&shader);
    if !gl
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .is_truthy()
    {
        let compiler_output = &gl
            .get_shader_info_log(&shader)
            .ok_or(ShaderError::ShaderGetInfoError)?;
        gl.delete_shader(Some(&shader));
        return Err(ShaderError::ShaderCompileError {
            shader_type,
            compiler_output: compiler_output.to_string(),
        });
    }
    Ok(shader)
}

pub fn init_shader_program(
    gl: &WebGl2RenderingContext,
    vert_source: &str,
    frag_source: &str,
) -> Result<WebGlProgram, ShaderError> {
    let vert_shader = load_shader(gl, WebGl2RenderingContext::VERTEX_SHADER, vert_source)?;
    let frag_shader = load_shader(gl, WebGl2RenderingContext::FRAGMENT_SHADER, frag_source)?;

    let shader_program = gl
        .create_program()
        .ok_or(ShaderError::ShaderProgramAllocError)?;
    gl.attach_shader(&shader_program, &vert_shader);
    gl.attach_shader(&shader_program, &frag_shader);

    gl.link_program(&shader_program);

    if !(gl.get_program_parameter(&shader_program, WebGl2RenderingContext::LINK_STATUS)).is_truthy()
    {
        gl.delete_program(Some(&shader_program));
        gl.delete_shader(Some(&vert_shader));
        gl.delete_shader(Some(&frag_shader));
        return Err(ShaderError::ShaderLinkError());
    }

    Ok(shader_program)
}
