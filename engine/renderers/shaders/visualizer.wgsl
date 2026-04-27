struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct Globals {
    brightness: f32,
    time: f32,
    _pad0: vec2<f32>,

    spectrum: array<vec4<f32>, 16>,
};


@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;
@group(0) @binding(2) var<uniform> globals: Globals;

const PI: f32 = 3.1415926;
const ROTATION_OFFSET: f32 = 1.570795;
const ASPECT: f32 = 16.0 / 9.0;

fn spectrum_get(i: i32) -> f32 {
    let vi = i / 4;
    let ci = i % 4;
    let v = globals.spectrum[vi];

    if ci == 0 { return v.x; }
    if ci == 1 { return v.y; }
    if ci == 2 { return v.z; }
    return v.w;
}

fn spectrum_sample(i: f32) -> f32 {
    let center = i * 0.5;

    var sum: f32 = 0.0;
    var weight_sum: f32 = 0.0;

    let radius = 3.0;

    var k = -radius;
    loop {
        if k > radius { break; }

        let sample_index = center + k;

        let fi = clamp(sample_index, 0.0, 63.0);
        let i0 = i32(fi);
        let i1 = min(i0 + 1, 63);

        let t = fi - f32(i0);

        let a = spectrum_get(i0);
        let b = spectrum_get(i1);

        let v = mix(a, b, t);

        let w = exp(-k * k * 0.6);

        sum = sum + v * w;
        weight_sum = weight_sum + w;

        k = k + 1.0;
    }

    return sum / weight_sum;
}

fn spectrum_circle(t: f32) -> f32 {
    let mirrored = abs(t * 2.0 - 1.0);

    let index = mirrored * 127.0;

    return spectrum_sample(index);
}

fn rotate(v: vec2<f32>, a: f32) -> vec2<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec2<f32>(
        v.x * c - v.y * s,
        v.x * s + v.y * c
    );
}

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(1 - i32(i)) * 3.0;
    let y = f32((i32(i) & 1) * 2 - 1) * 3.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    out.tex_coords = vec2<f32>(
        x * 0.5 + 0.5,
        1.0 - (y * 0.5 + 0.5)
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let bg = vec4<f32>(tex.rgb * globals.brightness, tex.a);

    let uv0 = in.tex_coords * 2.0 - 1.0;
    let uv = vec2<f32>(uv0.x * ASPECT, uv0.y);


    let uv_r = rotate(uv, ROTATION_OFFSET);

    let r = length(uv_r);
    let angle = atan2(uv_r.y, uv_r.x);

    let t = (angle + PI) / (2.0 * PI);


    let amp = spectrum_circle(t);

    let base_radius = 0.45;
    let wave = base_radius + amp * 0.006;

    let dist = abs(r - wave);

    let core = smoothstep(0.006, 0.002, dist);
    let glow = exp(-dist * 32.0);

    let energy = clamp(core + glow * 0.5, 0.0, 1.0);

    let hue_shift = globals.time + amp * 0.00002;
    let hue = fract(t + hue_shift);

    let saturation = 0.9;

    let value = energy + amp * 0.6;

    let color = hsv2rgb(hue, saturation, value);

    let final_rgb = bg.rgb + color * energy;

    return vec4<f32>(final_rgb, bg.a);
}

fn hsv2rgb(h: f32, s: f32, v: f32) -> vec3<f32> {
    let c = v * s;
    let x = c * (1.0 - abs((h * 6.0) % 2.0 - 1.0));
    let m = v - c;

    var rgb: vec3<f32>;

    if (h < 0.1666667) {
        rgb = vec3<f32>(c, x, 0.0);
    } else if (h < 0.3333333) {
        rgb = vec3<f32>(x, c, 0.0);
    } else if (h < 0.5) {
        rgb = vec3<f32>(0.0, c, x);
    } else if (h < 0.6666667) {
        rgb = vec3<f32>(0.0, x, c);
    } else if (h < 0.8333333) {
        rgb = vec3<f32>(x, 0.0, c);
    } else {
        rgb = vec3<f32>(c, 0.0, x);
    }

    return rgb + vec3<f32>(m);
}
