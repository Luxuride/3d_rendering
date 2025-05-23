use eframe::glow;
use std::sync::Arc;
use glow::HasContext;

pub struct SimpleOpenGLApp {
    gl: Arc<glow::Context>,
    program: glow::Program,
    vertex_array: glow::VertexArray,
    rotation: f32,
}

impl SimpleOpenGLApp {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        use glow::HasContext as _;
        
        // Safe because we're in the same thread as the main thread that created the context
        let (program, vertex_array) = unsafe {
            // Create shader program
            let program = gl.create_program().expect("Cannot create program");
            
            // Vertex shader - basic pass-through with rotation
            let vertex_shader_source = r#"
                #version 330 core
                in vec2 position;
                in vec3 color;
                out vec3 v_color;
                uniform float u_rotation;
                
                void main() {
                    // Apply rotation
                    float cos_theta = cos(u_rotation);
                    float sin_theta = sin(u_rotation);
                    mat2 rotation = mat2(
                        cos_theta, -sin_theta,
                        sin_theta, cos_theta
                    );
                    
                    // Position in clip space
                    gl_Position = vec4(rotation * position, 0.0, 1.0);
                    
                    // Pass color to fragment shader
                    v_color = color;
                }
            "#;
            
            // Fragment shader - simple solid color output
            let fragment_shader_source = r#"
                #version 330 core
                in vec3 v_color;
                out vec4 frag_color;
                
                void main() {
                    frag_color = vec4(v_color, 1.0);
                }
            "#;
            
            // Compile and link shaders
            let shader_sources = [
                (glow::VERTEX_SHADER, vertex_shader_source),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ];
            
            let shaders: Vec<glow::Shader> = shader_sources
                .iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl.create_shader(*shader_type).expect("Cannot create shader");
                    gl.shader_source(shader, shader_source);
                    gl.compile_shader(shader);
                    
                    if !gl.get_shader_compile_status(shader) {
                        panic!("Failed to compile shader: {}", gl.get_shader_info_log(shader));
                    }
                    
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();
            
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("Failed to link program: {}", gl.get_program_info_log(program));
            }
            
            // Clean up shader objects
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            
            // Create vertex array and buffer
            let vertex_array = gl.create_vertex_array().expect("Cannot create vertex array");
            gl.bind_vertex_array(Some(vertex_array));
            
            // Define a simple triangle with position (x,y) and color (r,g,b)
            let vertices: &[f32] = &[
                // position    // color
                -0.5, -0.5,    1.0, 0.0, 0.0,  // bottom left (red)
                 0.5, -0.5,    0.0, 1.0, 0.0,  // bottom right (green)
                 0.0,  0.5,    0.0, 0.0, 1.0,  // top (blue)
            ];
            
            let vertex_buffer = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices),
                glow::STATIC_DRAW,
            );
            
            // Define vertex attributes 
            let stride = 5 * std::mem::size_of::<f32>() as i32;
            
            // Position attribute (x, y) - 2 floats
            let position_attrib = gl.get_attrib_location(program, "position").unwrap();
            gl.enable_vertex_attrib_array(position_attrib);
            gl.vertex_attrib_pointer_f32(
                position_attrib,
                2,                          // 2 components (x, y)
                glow::FLOAT,
                false,                      // not normalized
                stride,                     // stride
                0,                          // offset
            );
            
            // Color attribute (r, g, b) - 3 floats
            let color_attrib = gl.get_attrib_location(program, "color").unwrap();
            gl.enable_vertex_attrib_array(color_attrib);
            gl.vertex_attrib_pointer_f32(
                color_attrib,
                3,                          // 3 components (r, g, b)
                glow::FLOAT,
                false,                      // not normalized
                stride,                     // stride
                2 * std::mem::size_of::<f32>() as i32, // offset
            );
            
            // Clean up bindings
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            
            (program, vertex_array)
        };
        
        Self {
            gl,
            program,
            vertex_array,
            rotation: 0.0,
        }
    }
    
    // Renders the triangle with the current rotation
    pub fn render(&self, gl: &glow::Context, rotation: f32) {
        unsafe {
            use glow::HasContext as _;
            
            // Set up OpenGL state for rendering
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.clear_color(0.1, 0.1, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            
            // Use our shader program
            gl.use_program(Some(self.program));
            
            // Set the rotation uniform
            let rotation_loc = gl.get_uniform_location(self.program, "u_rotation").unwrap();
            gl.uniform_1_f32(Some(&rotation_loc), rotation);
            
            // Bind and draw the triangle
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            // Clean up
            gl.bind_vertex_array(None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
        }
    }
    
    // Updates the rotation for animation
    pub fn update_rotation(&mut self) {
        self.rotation = (self.rotation + 0.05) % (std::f32::consts::TAU);
    }
    
    // Gets the current rotation value
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    
    // Gets the program ID - used for callbacks
    pub fn program(&self) -> glow::Program {
        self.program
    }
    
    // Gets the vertex array ID - used for callbacks
    pub fn vertex_array(&self) -> glow::VertexArray {
        self.vertex_array
    }
    
    // Cleanup OpenGL resources
    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            use glow::HasContext as _;
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }
} 