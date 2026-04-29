struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Ephemerals {
    lum: f32,
    time: f32,
    _padding: vec2<f32>,
    spectrum: array<vec4<f32>, 16>,
};

struct Scaling {
    bg_scaling: vec2<f32>,
    aspect_ratio: f32,
}

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var t_noise: texture_2d<f32>;
@group(0) @binding(3) var s_noise: sampler;
@group(0) @binding(4) var<uniform> ephemerals: Ephemerals;
@group(0) @binding(5) var<uniform> scaling: Scaling;

const PI: f32 = 3.14159265;
const ROTATION: f32 = 1.5708;

const SPEC_RADIUS: f32 = 2.0;
const SPEC_GAUSS: f32 = 0.7;

const BASE_RAD: f32 = 0.48;
const DIM_MOD: f32 = 0.18;
const MIN_RAD: f32 = 0.24;
const WAVE_AMP: f32 = 0.004;

const FALLOFF: f32 = 0.65;
const CORE_OUTER: f32 = 0.004;
const CORE_INNER: f32 = 0.0015;
const GLOW_EXP: f32 = 24.0;
const GLOW_GAIN: f32 = 0.52;

const SWIRL_STRENGTH: f32 = 5.5;
const SWIRL_SPEED: f32 = 0.8;
const GRAIN_STRENGTH: f32 = 0.75;
const NOISE_SPEED: f32 = 0.015;

const INSET_WIDTH: f32 = 0.5;
const SHIFT_STRENGTH: f32 = 0.38;
const BOOST_VALUE: f32 = 0.28;
const HUE_SPEED: f32 = 0.55;

fn get_spectrum_val(index: i32) -> f32 {
    let vec_idx = index >> 2;
    let component = index & 3;
    let data = ephemerals.spectrum[vec_idx];

    if component == 0 { return data.x; }
    if component == 1 { return data.y; }
    if component == 2 { return data.z; }
    return data.w;
}

fn sample_spectrum(position: f32) -> f32 {
    let center = position * 0.5;
    var total = 0.0;
    var weight_sum = 0.0;

    for (var k: f32 = -SPEC_RADIUS; k <= SPEC_RADIUS; k += 1.0) {
        let f_idx = clamp(center + k, 0.0, 63.0);
        let i_idx = i32(f_idx);
        let val = mix(get_spectrum_val(i_idx), get_spectrum_val(i_idx + 1), fract(f_idx));

        let weight = exp(-k * k * SPEC_GAUSS);
        total += val * weight;
        weight_sum += weight;
    }
    return total / max(weight_sum, 0.0001);
}

fn get_circular_amplitude(t: f32) -> f32 {
    let mirror = abs(t * 2.0 - 1.0);
    return sample_spectrum(mirror * 127.0);
}

fn apply_swirl(uv: vec2<f32>, strength: f32, time: f32) -> vec2<f32> {
    let dir = uv - 0.5;
    let radius = length(dir);
    let angle = strength * radius * (sin(time * SWIRL_SPEED) * 0.3 + 1.0) * 0.8;

    let rot = dir + vec2<f32>(dir.y, -dir.x) * angle;
    return 0.5 + rot * inverseSqrt(1.0 + angle * angle);
}

@vertex
fn vs_main(@builtin(vertex_index) id: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(id)) * 3.0;
    let y = f32((i32(id) & 1) * 2 - 1) * 3.0;

    out.clip_pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

fn sample_bg(uv: vec2<f32>) -> vec4<f32> {
    let bg_uv = (uv - 0.5) * scaling.bg_scaling + 0.5;
    let sampled_bg = textureSample(t_diffuse, s_diffuse, bg_uv);

    return vec4<f32>(sampled_bg.rgb * ephemerals.lum, sampled_bg.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let bg_sample = sample_bg(in.uv);
    let ndc_uv = in.uv * 2.0 - 1.0;
    let aspect_uv = vec2<f32>(ndc_uv.x * scaling.aspect_ratio, ndc_uv.y);

    let rotated_uv = rotate_vector(aspect_uv, ROTATION);

    let dist_to_center = length(rotated_uv);
    let angle = (atan2(rotated_uv.y, rotated_uv.x) + PI) * 0.1591549;

    let amplitude = get_circular_amplitude(angle);
    let base_rad = max(BASE_RAD - ephemerals.lum * DIM_MOD, MIN_RAD);
    let ring_wave = base_rad + amplitude * WAVE_AMP;

    let ring_dist = abs(dist_to_center - ring_wave);
    let falloff_mask = 1.0 - smoothstep(0.0, FALLOFF, dist_to_center);

    let core_light = smoothstep(CORE_OUTER, CORE_INNER, ring_dist);
    let glow_light = exp(-ring_dist * GLOW_EXP);

    let energy = clamp(core_light + glow_light * GLOW_GAIN, 0.0, 1.0) * falloff_mask;

    let swirl_uv = apply_swirl(in.uv, SWIRL_STRENGTH, ephemerals.time);
    let noise_scroll = vec2<f32>(0.0, ephemerals.time * NOISE_SPEED);
    let noise_val = textureSample(t_noise, s_noise, swirl_uv + noise_scroll).r;

    let grain = (noise_val - 0.5) * GRAIN_STRENGTH * energy * (1.0 + dist_to_center);
    let noisy_energy = clamp(energy + grain, 0.0, 1.0);

    let inset = smoothstep(ring_wave - 0.13, ring_wave - INSET_WIDTH, dist_to_center);
    let downward_bias = (1.0 - aspect_uv.y) * SHIFT_STRENGTH * inset;

    let hue = fract(angle + ephemerals.time * HUE_SPEED + amplitude * 0.002 + downward_bias * 0.12);
    let sat = 1.0 - amplitude * 0.061;
    let val = noisy_energy * 0.92 + amplitude * 0.75 + downward_bias * BOOST_VALUE;

    let ring_rgb = hsv_to_rgb(hue, sat, val);
    let final_color = bg_sample.rgb + ring_rgb * noisy_energy;

    return vec4<f32>(final_color, bg_sample.a);
}

fn rotate_vector(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let cos_a = cos(angle);
    let sin_a = sin(angle);
    return vec2<f32>(v.x * cos_a - v.y * sin_a, v.x * sin_a + v.y * cos_a);
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let chroma = v * s;
    let x = chroma * (1.0 - abs((h * 6.0) % 2.0 - 1.0));
    let m = v - chroma;

    var rgb: vec3<f32>;
    if h < 0.1667 { rgb = vec3<f32>(chroma, x, 0.0); }
    else if h < 0.3333 { rgb = vec3<f32>(x, chroma, 0.0); }
    else if h < 0.5000 { rgb = vec3<f32>(0.0, chroma, x); }
    else if h < 0.6667 { rgb = vec3<f32>(0.0, x, chroma); }
    else if h < 0.8333 { rgb = vec3<f32>(x, 0.0, chroma); }
    else { rgb = vec3<f32>(chroma, 0.0, x); }

    return rgb + m;
}
