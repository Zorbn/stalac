struct InstanceInput {
    @location(4) model_matrix_0: vec4<f32>,
    @location(5) model_matrix_1: vec4<f32>,
    @location(6) model_matrix_2: vec4<f32>,
    @location(7) model_matrix_3: vec4<f32>,
    @location(8) tex_index: u32,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec3<f32>,
    @location(3) tex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tex_index: u32,
    @location(2) vertex_color: vec3<f32>,
    @location(3) light_level: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let light_pos = vec3(0.0, 0.0, 0.0);

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.tex_index = instance.tex_index + model.tex_index;
    out.vertex_color = model.color;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.light_level = distance(out.clip_position.xyz, light_pos);
    return out;
}

@group(0) @binding(0)
var t_diffuse_array: binding_array<texture_2d<f32>>;
@group(0)@binding(1)
var s_diffuse_array: binding_array<sampler>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse_array[in.tex_index], s_diffuse_array[in.tex_index], in.tex_coords);

    if color.a < 0.5 {
        discard;
    }

    let lit_color = vec4(in.vertex_color * color.rgb / (abs(in.light_level) + 1.0), color.a);

    return lit_color;
}