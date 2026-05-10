const PI: f32 = 3.14159265358979323846;

struct FFTParams {
    n: u32,
    log2_n: u32,
    stage: u32,
    direction: f32, // 1.0 for forward, -1.0 for inverse
}

struct ComplexArray {
    data: array<vec2<f32>>,
}

@group(0) @binding(0) var<storage, read_write> buffer: ComplexArray;
@group(0) @binding(1) var<uniform> params: FFTParams;

fn complex_mul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x);
}

@compute @workgroup_size(256)
fn bit_reversal(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    if (i >= params.n) { return; }

    let shift = 32u - params.log2_n;
    let rev_i = reverseBits(i) >> shift;

    if (i < rev_i) {
        let temp = buffer.data[i];
        buffer.data[i] = buffer.data[rev_i];
        buffer.data[rev_i] = temp;
    }
}

@compute @workgroup_size(256)
fn butterfly(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let k = global_id.x;
    let half_step = 1u << params.stage;
    let step = half_step << 1u;

    if (k >= params.n / 2u) { return; }

    let group = k / half_step;
    let index = k % half_step;
    
    let even_idx = group * step + index;
    let odd_idx = even_idx + half_step;

    let angle = -params.direction * 2.0 * PI * f32(index) / f32(step);
    let twiddle = vec2<f32>(cos(angle), sin(angle));

    let even_val = buffer.data[even_idx];
    let odd_val = buffer.data[odd_idx];
    let t = complex_mul(twiddle, odd_val);

    buffer.data[even_idx] = even_val + t;
    buffer.data[odd_idx] = even_val - t;
}