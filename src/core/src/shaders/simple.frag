#version 300 es
// This is a test of signed distance fields where instead of each SDF function
// only returning the distance, it returns a struct containing
// the surface normals and a surface ID as well
// The aim of this is to allow simple CAD style rendering, edge
// drawing, etc.
//
// The output of this shader is an image containing the normals, distance from camera
// and the surface ID.

precision mediump float;
in vec4 screen_pos;
out vec4 FragColor;

uniform vec2 resolution;

const int MAX_STEPS = 30;
const float SURFACE_DISTANCE = 0.001;
const float VIEW_DISTANCE = 20.0;


// Core instructions
const float INSTRUCTION_STOP = (0.0);
const float INSTRUCTION_NEW_ENTITY = (1.0);


// Shapes
const float INSTRUCTION_SPHERE = (100.0);
const float INSTRUCTION_BOX = (101.0);

// Transformations
const float INSTRUCTION_TRANSLATE = (200.0);
const float INSTRUCTION_ROTATE = (201.0);
const float INSTRUCTION_SCALE = (202.0);


// Operations
const float INSTRUCTION_UNION = (300.0);
const float INSTRUCTION_DIFFERENCE = (301.0);
const float INSTRUCTION_INTERSECT = (302.0);


uniform float[1000] scene_description;


struct surface_t {
    int surface_id;
    float sdf;
};


surface_t sphere_sdf(vec3 query_point, int id, float sphere_radius) {
    return surface_t(
        id,
        length(query_point)  - sphere_radius
    );
}

surface_t box_sdf(vec3 query_point, int id, vec3 dimensions) {
	vec3 q = abs(query_point) - dimensions;  
  
    return surface_t(
        id,
        length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0)
    );
}

surface_t surface_union(surface_t surface_1, surface_t surface_2) {
    // Join surface_1 to surface_2
    if (surface_1.sdf < surface_2.sdf){
        return surface_1;
    } else {
     	return surface_2;   
    }
}
surface_t surface_intersect(surface_t surface_1, surface_t surface_2) {
    // Find the volume both surface occupy
    if (surface_1.sdf > surface_2.sdf){
        return surface_1;
    } else {
     	return surface_2;   
    }
}

surface_t surface_difference(surface_t surface_1, surface_t surface_2) {
    // subtract surface_2 from surface_1
    // invert surface 2:
    surface_2.sdf = -surface_2.sdf;
    return surface_intersect(surface_1, surface_2);
}



vec3 transform(vec3 inp, mat4 offset) {
    return (offset * vec4(inp, 1.0)).xyz;
}

mat4 translation(vec3 trans) {
    // Construct a translation matrix
    return mat4 (
        vec4(1.0, 0.0, 0.0, 0.0), 
        vec4(0.0, 1.0, 0.0, 0.0), 
        vec4(0.0, 0.0, 1.0, 0.0), 
        vec4(trans.x, trans.y, trans.z, 1.0)
    );
    
}

surface_t world(vec3 world_position) {
	
	
	int pointer = 0;
	int entity_id = 0;
	surface_t scene_sdf = surface_t(0, 9999.9);
	surface_t obj_sdf = surface_t(0, 9999.9);
	vec3 view_point = world_position;
	
	
	for(int i = 0; i < 9999; i++) {
		float data = scene_description[pointer];
		if (data == INSTRUCTION_STOP) {
			// Scene ends
			break;
		} else if (data == INSTRUCTION_NEW_ENTITY) {
			// New entity to work with
			view_point = world_position;
			entity_id += 1;
			pointer += 1;
			

		} else if (data == INSTRUCTION_TRANSLATE) {
			// Perform translation
			vec3 offset = vec3(
				scene_description[pointer+1],
				scene_description[pointer+2],
				scene_description[pointer+3]
			);
			view_point = transform(view_point, translation(offset));
			pointer += 4;
		
		} else if (data == INSTRUCTION_ROTATE) {
			// Perform translation
			vec3 offset = vec3(
				scene_description[pointer+1], // Euler X
				scene_description[pointer+2], // Euler Y
				scene_description[pointer+3] // Euler Z
			);
			vec3 c = cos(offset);
			vec3 s = sin(offset);
			
			mat4 mat = mat4(
				vec4(c.z*c.x, -c.z*s.x*c.y + s.z*s.y, c.z*s.x*s.y + s.z*c.y, 0.0),
				vec4(s.x, c.x*c.y, -c.x*s.y, 0.0),
				vec4(-s.z*c.x, s.z*s.x*c.y + c.z*s.y, -s.z*s.x*s.y + c.z*c.y, 0.0),
				vec4(0.0, 0.0, 0.0, 1.0)
			);
			
			view_point = transform(view_point, mat);
			pointer += 4;
		
		
		} else if (data == INSTRUCTION_SPHERE) {
			float radius = scene_description[pointer+1];
			obj_sdf = sphere_sdf(view_point, entity_id, radius);
			pointer += 2;
		} else if (data == INSTRUCTION_BOX) {
			vec3 dimensions = vec3(
				scene_description[pointer+1],
				scene_description[pointer+2],
				scene_description[pointer+3]
			);
			obj_sdf = box_sdf(view_point, entity_id, dimensions);
			pointer += 4;


		} else if (data == INSTRUCTION_UNION) {
			scene_sdf = surface_union(scene_sdf, obj_sdf);
			pointer += 1;
		} else if (data == INSTRUCTION_DIFFERENCE) {
			scene_sdf = surface_difference(scene_sdf, obj_sdf);
			pointer += 1;
		} else if (data == INSTRUCTION_INTERSECT) {
			scene_sdf = surface_intersect(scene_sdf, obj_sdf);
			pointer += 1;
		}
	}
    
    return scene_sdf;
    
    
}


vec3 gen_color(float t){
	return vec3(
		0.5 + 0.5 * cos(2.0 * 3.1415 * (1.0 * t + 0.0)),
		0.5 + 0.5 * cos(2.0 * 3.1415 * (1.0 * t + 0.33)),
		0.5 + 0.5 * cos(2.0 * 3.1415 * (1.0 * t + 0.66))
	);
}


void mainImage( out vec4 fragColor, in vec2 fragCoord )
{   
    // camera coords (from -1 to 1)
    vec2 cam_coords = screen_pos.xy;//(fragCoord/iResolution.xy - vec2(0.5)) * 2.0;
    cam_coords.x *= resolution.x / resolution.y;
    
    vec3 ray_start_position = vec3(cam_coords, 0.0);
    vec3 ray_direction = vec3(0.0, 0.0, 1.0);
        
    vec3 sample_point = ray_start_position;
    surface_t results = world(sample_point);
    float dist = 0.0;
    
    for (int i=0; i<MAX_STEPS; i += 1) {
        dist += results.sdf;
        sample_point += ray_direction * results.sdf;
        results = world(sample_point);
        
        if (results.sdf < SURFACE_DISTANCE || dist > VIEW_DISTANCE) {
			break;
		}
    }
    
    
    // There is banding caused by the difference in direction between
    // calculation of normals of adjacent 2x2 pixel squares.
    // It would be better to do this in a separate pass or
    // with some other method
    vec3 normal = vec3(
		dFdx(dist),
		dFdy(dist),
		0.0
    );
    
    normal *= resolution.y / 10.0;
    //normal = pow(normal, vec3(0.5));
    normal.z = sqrt(1.0-dot(normal.xy, normal.xy));
    
    
    vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
    
    if (results.sdf < SURFACE_DISTANCE) {
        // We hit a surface
        
        float lighting = dot(normal, vec3(0.2, 0.3, 0.7));
        lighting = pow(lighting, 2.0);
        
        //~ color = vec4(
            //~ normal.x,
            //~ normal.y,
            //~ float(results.surface_id),
            //~ dist
        //~ );

        color = vec4(gen_color(float(results.surface_id) / 4.0), 1.0);
        color *= vec4(vec3(lighting), 1.0);
        color = clamp(color, 0.0, 1.0);
    }

    // Output to screen
    fragColor = color;
}


void main() {
       mainImage(FragColor, screen_pos.xy);
}
