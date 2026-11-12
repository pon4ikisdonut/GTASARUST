// GTASARUST World Instanced Shader (Z-up San Andreas Coordinate System)
// Implements GTA:SA CustomBuildingDNPipeline (day/night pre-lit vertex blending),
// dynamic entity lighting (Amb_Obj + Sun), alpha cutout, and TimeCycle fog.

struct FrameUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    ambient_color: vec4<f32>,
    ambient_obj_color: vec4<f32>,
    fog_color: vec4<f32>,
    fog_params: vec4<f32>,   // x = fog_start, y = fog_end, z = time_hours, w = rain_intensity
    dn_and_extra: vec4<f32>, // x = dn_balance (0..1), y..w reserved
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
    @location(4) night_color: vec4<f32>,
};

struct InstanceInput {
    @location(5) model_c0: vec4<f32>,
    @location(6) model_c1: vec4<f32>,
    @location(7) model_c2: vec4<f32>,
    @location(8) model_c3: vec4<f32>,
    @location(9) tint: vec4<f32>,
    @location(10) light_params: vec4<f32>, // x: mode (0=building prelit, 1=dynamic obj, 2=unlit/water), y: alpha_cutoff
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) night_color: vec4<f32>,
    @location(5) light_params: vec4<f32>,
};

@vertex
fn vs_main(vert: VertexInput, inst: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(inst.model_c0, inst.model_c1, inst.model_c2, inst.model_c3);
    let world_p = model * vec4<f32>(vert.position, 1.0);
    let world_n = normalize((model * vec4<f32>(vert.normal, 0.0)).xyz);

    var out: VertexOutput;
    out.clip_pos = frame.view_proj * world_p;
    out.world_pos = world_p.xyz;
    out.world_normal = world_n;
    let is_water = inst.light_params.x > 2.5;
    out.uv = select(
        vert.uv,
        vert.uv + vec2<f32>(frame.fog_params.z * 0.09, frame.fog_params.z * 0.06),
        is_water,
    );
    out.color = vert.color * inst.tint;
    out.night_color = vert.night_color * inst.tint;
    out.light_params = inst.light_params;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(base_tex, base_sampler, in.uv);
    let alpha = texel.a * in.color.a;
    let cutoff = select(0.02, in.light_params.y, in.light_params.y > 0.0);
    if alpha < cutoff {
        discard;
    }

    let n = normalize(in.world_normal);
    let l = normalize(frame.sun_dir.xyz);
    let ndotl = max(dot(n, l), 0.0);

    var lit: vec3<f32>;
    if in.light_params.x > 1.5 {
        // Unlit / Water / Emissive markers
        lit = texel.rgb * in.color.rgb;
    } else if in.light_params.x > 0.5 {
        // Dynamic entities (CJ, NPCs, vehicles): Amb_Obj + directional sun fill
        let half_lambert = dot(n, l) * 0.5 + 0.5;
        let sun_term = frame.sun_color.rgb * frame.sun_color.w * (0.48 * ndotl + 0.22 * half_lambert);
        let amb_term = max(frame.ambient_obj_color.rgb * 0.85, vec3<f32>(0.24, 0.24, 0.26));
        lit = (amb_term + sun_term) * texel.rgb * in.color.rgb;
    } else {
        // Static world buildings & terrain: GTA:SA Day/Night pre-lit vertex color pipeline
        let dn = clamp(frame.dn_and_extra.x, 0.0, 1.0);
        let prelit = mix(in.color.rgb, in.night_color.rgb, dn);
        let sun_mod = mix(0.90, 1.15, ndotl) * mix(1.0, 0.80, dn);
        let amb_lift = frame.ambient_color.rgb * 0.35;
        lit = texel.rgb * (prelit * sun_mod + amb_lift);
    }

    let dist = distance(in.world_pos, frame.camera_pos.xyz);
    let fog_start = frame.fog_params.x;
    let fog_end = max(frame.fog_params.y, fog_start + 1.0);
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);

    let final_rgb = mix(lit, frame.fog_color.rgb, fog_factor);
    return vec4<f32>(final_rgb, alpha);
}
