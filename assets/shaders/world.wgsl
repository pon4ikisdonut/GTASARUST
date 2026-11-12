// GTASARUST World & Entity Shader (Z-up San Andreas World Space)

struct FrameUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,      // normalized direction toward sun
    sun_color: vec4<f32>,    // rgb + intensity
    ambient_color: vec4<f32>,
    fog_color: vec4<f32>,
    fog_params: vec4<f32>,   // x = start, y = end, z = time_of_day_hours, w = weather_rain
};

@group(0) @binding(0)
var<uniform> frame: FrameUniforms;

@group(1) @binding(0)
var base_tex: texture_2d<f32>;

@group(1) @binding(1)
var base_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct InstanceInput {
    @location(4) model_c0: vec4<f32>,
    @location(5) model_c1: vec4<f32>,
    @location(6) model_c2: vec4<f32>,
    @location(7) model_c3: vec4<f32>,
    @location(8) tint: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(v: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(inst.model_c0, inst.model_c1, inst.model_c2, inst.model_c3);
    let world_pos4 = model * vec4<f32>(v.position, 1.0);
    let normal_mat = mat3x3<f32>(inst.model_c0.xyz, inst.model_c1.xyz, inst.model_c2.xyz);

    var out: VertexOutput;
    out.clip_position = frame.view_proj * world_pos4;
    out.world_pos = world_pos4.xyz;
    out.world_normal = normalize(normal_mat * v.normal);
    out.uv = v.uv;
    out.color = v.color * inst.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(base_tex, base_sampler, in.uv);
    if texel.a < 0.05 {
        discard;
    }

    let n = normalize(in.world_normal);
    let l = normalize(frame.sun_dir.xyz);
    let ndotl = max(dot(n, l), 0.0);

    let diffuse = frame.sun_color.rgb * frame.sun_color.w * ndotl;
    let lit = (frame.ambient_color.rgb + diffuse) * texel.rgb * in.color.rgb;

    let dist = distance(in.world_pos, frame.camera_pos.xyz);
    let fog_range = max(frame.fog_params.y - frame.fog_params.x, 1.0);
    let fog_factor = clamp((dist - frame.fog_params.x) / fog_range, 0.0, 1.0);

    let final_rgb = mix(lit, frame.fog_color.rgb, fog_factor);
    return vec4<f32>(final_rgb, texel.a * in.color.a);
}
