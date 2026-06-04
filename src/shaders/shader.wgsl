@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera_pos: vec4<f32>;

@group(1) @binding(0)
var t_basecolor: texture_2d<f32>;
@group(1) @binding(1)
var s_material: sampler;
@group(1) @binding(2)
var t_normal: texture_2d<f32>;
@group(1) @binding(3)
var t_roughness: texture_2d<f32>;
@group(1) @binding(4)
var t_metalness: texture_2d<f32>;
@group(1) @binding(5)
var t_ao: texture_2d<f32>;
@group(1) @binding(6)
var t_emissive: texture_2d<f32>;
@group(1) @binding(7)
var<uniform> material: MaterialUniforms;

struct MaterialUniforms {
    diffuse_color: vec4<f32>,
    specular_color: vec4<f32>,
    ambient_color: vec4<f32>,
    emissive_color: vec4<f32>,
    params: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_tangent: vec3<f32>,
    @location(3) world_bitangent: vec3<f32>,
    @location(4) world_position: vec3<f32>,
}

const PI: f32 = 3.14159265359;
const EPSILON: f32 = 0.0001;

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let ndoth = max(dot(n, h), 0.0);
    let ndoth2 = ndoth * ndoth;
    let denom = ndoth2 * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return ndotv / (ndotv * (1.0 - k) + k);
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    let ndotv = max(dot(n, v), 0.0);
    let ndotl = max(dot(n, l), 0.0);
    return geometry_schlick_ggx(ndotv, roughness) * geometry_schlick_ggx(ndotl, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = view_proj * vec4<f32>(in.position, 1.0);
    out.uv = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    out.world_normal = normalize(in.normal);
    out.world_tangent = normalize(in.tangent);
    out.world_bitangent = normalize(in.bitangent);
    out.world_position = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let basecolor = textureSample(t_basecolor, s_material, in.uv);
    let tn = textureSample(t_normal, s_material, in.uv).rgb * 2.0 - 1.0;
    let roughness_sample = textureSample(t_roughness, s_material, in.uv).g;
    let metalness_sample = textureSample(t_metalness, s_material, in.uv).b;
    let ao_sample = textureSample(t_ao, s_material, in.uv).r;
    let emissive_sample = textureSample(t_emissive, s_material, in.uv);

    let albedo = basecolor.rgb * material.diffuse_color.rgb;
    let roughness = roughness_sample;
    let metalness = metalness_sample;
    let ao = ao_sample;
    let n = normalize(in.world_normal);
    let t = normalize(in.world_tangent);
    let b = normalize(in.world_bitangent);
    let tbn = mat3x3<f32>(t, b, n);
    let normal = normalize(tbn * tn);

    let world_pos = in.world_position;
    let v = normalize(camera_pos.xyz - world_pos);
    let ndotv = max(dot(normal, v), EPSILON);

    let f0 = mix(vec3<f32>(0.04), albedo, metalness);

    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let light_color = vec3<f32>(4.0, 3.8, 3.3);

    let l = light_dir;
    let h = normalize(v + l);
    let ndotl = max(dot(normal, l), 0.0);

    let ndf = distribution_ggx(normal, h, roughness);
    let g = geometry_smith(normal, v, l, roughness);
    let f = fresnel_schlick(clamp(dot(h, v), 0.0, 1.0), f0);

    let specular = (ndf * g * f) / (4.0 * ndotv * ndotl + EPSILON);
    let kd = (1.0 - f) * (1.0 - metalness);
    let diffuse = kd * albedo / PI;

    let direct_light = (diffuse + specular) * light_color * ndotl;

    let ambient_strength = 0.04;
    let ambient = ambient_strength * albedo * ao * material.ambient_color.rgb;

    let emissive = emissive_sample.rgb * material.emissive_color.rgb;

    let color = ambient + direct_light + emissive;

    return vec4<f32>(color, basecolor.a * material.diffuse_color.a);
}
