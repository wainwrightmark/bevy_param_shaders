#define_import_path bps::fill_with_border

fn fill(d: f32, color: vec4<f32>, border: f32, border_color: vec4<f32>) -> vec4<f32> {

    let global_amount = sd_fill_alpha_fwidth(d);

    let border_d = abs(d + border)  - border;
    let border_amount = sd_fill_alpha_fwidth(border_d);
    let rgba = mix(color.rgba, border_color, border_amount );

    return vec4<f32>(rgba.rgb, rgba.a * global_amount);
}

fn sd_fill_alpha_fwidth(distance: f32) -> f32 {
    let aaf = 0.71 * fwidth(distance);
    return smoothstep(aaf, -aaf, distance);
}