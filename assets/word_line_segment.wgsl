#define_import_path sdf::word_line_segment

fn sdf(p: vec2<f32>, line_width: f32, point2: vec2<f32>, progress: f32) -> f32 {

    let b = point2;
    let a = b * -1.;

    return line(p, a, b, line_width, progress);
}

fn line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>, line_width: f32, line_proportion: f32) -> f32 {
    let ba = (b - a) * line_proportion;
    let pa = (p - a);
    let k: f32 = saturate(dot(pa, ba) / dot(ba, ba));
    let len = length(pa - (ba * k));
    let len_a = len - line_width;
    return len_a;
}

