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

const float INSTRUCTION_STOP = 0.0;
const float INSTRUCTION_NEW_ENTITY = 1.0;
const float INSTRUCTION_TRANSLATE = 2.0;
const float INSTRUCTION_SPHERE = 3.0;
//const float INSTRUCTION_ = 4.0;
const float INSTRUCTION_UNION = 5.0;
const float INSTRUCTION_DIFFERENCE = 6.0;
const float INSTRUCTION_INTERSECT = 7.0;
//const float INSTRUCTION_STOP = 8.0;
//const float INSTRUCTION_STOP = 9.0;


uniform float[100] scene_description;
/*
float[] scene_description = float[] (
	1.0, // New Entity
	2.0, // translate
	0.0, // x
	0.0, // y
	-5.0, // z
	3.0, // sphere object
	1.0, // radius
	5.0, // union
    
    1.0, // New Entity
	2.0, // translate
	-1.0, // x
	0.0, // y
	-5.0, // z
	3.0, // sphere object
	1.0, // radius
	5.0, // union

    1.0, // New Entity
    2.0, // translate
	0.0, // x
	-1.0, // y
	-4.0, // z
	3.0, // sphere object
	1.0, // radius
	6.0, // difference
    
	0.0 // End
);
* */


struct surface_t {
    int surface_id;
    float sdf;
    vec3 normal;
};


surface_t sphere_sdf(vec3 query_point, int id, float sphere_radius) {
    return surface_t(
        id,
        length(query_point)  - sphere_radius,
        normalize(query_point)
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
    surface_2.normal = -surface_2.normal;
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
	surface_t scene_sdf = surface_t(0, 9999.9, vec3(0.0));
	surface_t obj_sdf = surface_t(0, 9999.9, vec3(0.0));
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
		} else if (data == INSTRUCTION_SPHERE) {
			float radius = scene_description[pointer+1];
			obj_sdf = sphere_sdf(view_point, entity_id, radius);
			pointer += 2;
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
	
	
	
    //surface_t sphere1 = sphere_sdf(transform(world_position, translation(-vec3(0.0, 0.0, 5.0))), 1, 1.0);
    //surface_t sphere2 = sphere_sdf(transform(world_position, translation(-vec3(0.0, 1.0, 4.0))), 2, 1.0);
    //surface_t sphere3 = sphere_sdf(transform(world_position, translation(-vec3(1.0, 0.0, 5.0))), 3, 1.0);
    
    //surface_t body = sphere1;
    //body = surface_union(body, sphere3);
    //body = surface_difference(body, sphere2);
    
    //for (int i; i<100; i++) {
    // 	surface_t new_sphere = sphere_sdf(transform(world_position, translation(-vec3(sin(float(i) + iTime), float(i) / 100.0, 5.0))), i+4, 1.0);
    //	body = surface_union(body, new_sphere);
    //}
    
    return scene_sdf;
    
    
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    const int steps = 10; // Steps to converge to surface
    float tolerance = 0.01; // It's a surface if the end result is within this distance of a surface.
    
    mat4 projection_matrix = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 0.0
    );
    
    // camera coords (from -1 to 1)
    vec2 cam_coords = screen_pos.xy;//(fragCoord/iResolution.xy - vec2(0.5)) * 2.0;
    cam_coords.x *= resolution.x / resolution.y;
    
    vec3 ray_start_position = (projection_matrix * vec4(cam_coords.x, cam_coords.y, 0.0, 0.0)).xyz;
    vec3 ray_end_position = (projection_matrix * vec4(cam_coords.x, cam_coords.y, 1.0, 0.0)).xyz;
    vec3 ray_direction = ray_end_position - ray_start_position;
    
    vec4 color = vec4(0.0, 0.0, 0.0, 1.0);
        
    vec3 sample_point = ray_start_position;
    surface_t results = world(sample_point);
    float dist = 0.0;
    
    for (int i=0; i<steps; i += 1) {
        dist += results.sdf;
        sample_point += ray_direction * results.sdf;
        results = world(sample_point);
    }
    
    if (results.sdf < tolerance) {
        // We hit a surface
        color = vec4(
            results.normal.x,
            results.normal.y,
            float(results.surface_id),
            dist
        );
    }

    // Output to screen
    fragColor = color;
}


void main() {
       mainImage(FragColor, screen_pos.xy);
}
