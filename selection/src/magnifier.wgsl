struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) box_uv: vec2<f32>,
}

// All vec4 to guarantee 16-byte alignment throughout
struct Params {
    // xy = cursor position in global screen pixels, zw = screenshot size in pixels
    cursor_screen: vec4<f32>,
    // xy = magnifier box top-left in window pixels, zw = box size in pixels
    box_data: vec4<f32>,
    // x = zoom factor, yzw = padding
    zoom_pad: vec4<f32>,
}

@group(0) @binding(0) var screenshot_tex: texture_2d<f32>;
@group(0) @binding(1) var screenshot_samp: sampler;
@group(0) @binding(2) var<uniform> params: Params;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Triangle strip quad: TL, TR, BL, BR
    // Derive UV from vertex index via bit manipulation (naga disallows dynamic array indexing)
    let box_uv = vec2<f32>(f32(vi & 1u), f32((vi >> 1u) & 1u));

    let screen_px = params.box_data.xy + box_uv * params.box_data.zw;
    let screen_size = params.cursor_screen.zw;

    // Screen pixels → NDC (flip Y: screen Y↓ = NDC Y↑)
    let ndc = vec2<f32>(
        (screen_px.x / screen_size.x) * 2.0 - 1.0,
        1.0 - (screen_px.y / screen_size.y) * 2.0,
    );

    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.box_uv = box_uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let cursor    = params.cursor_screen.xy;
    let screen_sz = params.cursor_screen.zw;
    let box_size  = params.box_data.zw;
    let zoom      = params.zoom_pad.x;

    // White border (2 px)
    let bx = 2.0 / box_size.x;
    let by = 2.0 / box_size.y;
    if in.box_uv.x < bx || in.box_uv.x > 1.0 - bx ||
       in.box_uv.y < by || in.box_uv.y > 1.0 - by {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }

    // Red crosshair at center (1 source-pixel wide)
    let cx = 1.0 / box_size.x;
    let cy = 1.0 / box_size.y;
    if abs(in.box_uv.x - 0.5) < cx || abs(in.box_uv.y - 0.5) < cy {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    // Sample screenshot at zoomed coordinate
    // Center of magnifier = cursor; each output pixel = 1/zoom source pixels
    let rel = (in.box_uv - vec2<f32>(0.5)) * box_size / zoom;
    let source_px = cursor + rel;
    let uv = source_px / screen_sz;

    if uv.x < 0.0 || uv.y < 0.0 || uv.x > 1.0 || uv.y > 1.0 {
        return vec4<f32>(0.1, 0.1, 0.1, 1.0);
    }
    return textureSample(screenshot_tex, screenshot_samp, uv);
}
