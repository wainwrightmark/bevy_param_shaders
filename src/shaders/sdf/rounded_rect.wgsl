#define_import_path bps::rounded_rect
fn sdf(p: vec2<f32>, width: f32, height: f32, rounding: f32) -> f32 {
    return sd_rounded_box(p, vec2<f32>(width, height), vec4<f32>(rounding * min(width, height)));
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var r_2 = r;
    // swizzle assigment isn't supported yet
    // r_2.xy = select(r_2.zw, r_2.xy, p.x > 0.);
    let tmp = select(r_2.zw, r_2.xy, p.x > 0.);
    r_2.x = tmp.x;
    r_2.y = tmp.y;
    r_2.x = select(r_2.y, r_2.x, p.y > 0.);
    let q = abs(p) - b + r_2.x;
    return min(
        max(q.x, q.y),
        0.
    ) + length(max(q, vec2<f32>(0.))) - r_2.x;
}