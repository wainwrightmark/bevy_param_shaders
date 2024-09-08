#define_import_path bps::rect
fn sdf(p: vec2<f32>, width: f32, height: f32) -> f32 {
    return sd_box(p, vec2<f32>(width, height));
}

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = abs(p) - b;
    return length(max(d, vec2<f32>(0.))) + min(max(d.x, d.y), 0.);
}