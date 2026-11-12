// GTASARUST Sky Dome / Timecycle Gradient Shader

struct SkyUniforms {
    inv_view_proj: mat4x4<f32>,
    sky_top: vec4<f32>,
    sky_bottom: vec4<f32>,
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> sky: SkyUniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_sky(@builtin(vertex_index) idx: u32) -> VsOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    let p = positions[idx];
    var out: VsOut;
    out.pos = vec4<f32>(p, 0.9999, 1.0);
    out.ndc = p;
    return out;
}

@fragment
fn fs_sky(in: VsOut) -> @location(0) vec4<f32> {
    let far_world = sky.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let near_world = sky.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
    let dir = normalize((far_world.xyz / far_world.w) - (near_world.xyz / near_world.w));

    // Z-up: horizon is dir.z == 0, zenith is dir.z == 1
    let t = clamp(dir.z * 0.85 + 0.15, 0.0, 1.0);
    var col = mix(sky.sky_bottom.rgb, sky.sky_top.rgb, pow(t, 0.65));
    // Below horizon: smooth blend toward distant SA sea/ground haze
    let below = clamp(-dir.z * 3.0, 0.0, 1.0);
    col = mix(col, sky.sky_bottom.rgb * vec3<f32>(0.55, 0.62, 0.68), below);

    // Subtle San Andreas sun halo
    let sun_dot = max(dot(dir, normalize(sky.sun_dir.xyz)), 0.0);
    col += sky.sun_color.rgb * pow(sun_dot, 96.0) * 0.9;
    col += sky.sun_color.rgb * pow(sun_dot, 8.0) * 0.12;

    return vec4<f32>(col, 1.0);
}
