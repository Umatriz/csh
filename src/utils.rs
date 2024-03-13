use bevy::{
    math::{Vec2, Vec3},
    render::color::Color,
};

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    (1.0 - t) * a + b * t
}

pub fn vec_lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a * (1.0 - t) + b * t
}

pub fn inv_lerp(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

pub fn color_lerp(color0: Color, color1: Color, t: f32) -> Color {
    let r0 = color0.r();
    let g0 = color0.g();
    let b0 = color0.b();
    let r1 = color1.r();
    let g1 = color1.g();
    let b1 = color1.b();

    Color::rgb(r0 + t * (r1 - r0), g0 + t * (g1 - g0), b0 + t * (b1 - b0))
}

pub fn color_lerp_linear(color0: Color, color1: Color, t: f32) -> Color {
    let l0 = color0.as_rgba_linear();
    let r0 = l0.r();
    let g0 = l0.g();
    let b0 = l0.b();

    let l1 = color1.as_rgba_linear();
    let r1 = l1.r();
    let g1 = l1.g();
    let b1 = l1.b();

    Color::rgb_linear(r0 + t * (r1 - r0), g0 + t * (g1 - g0), b0 + t * (b1 - b0))
}

pub fn squared_distance(point1: Vec3, point2: Vec3) -> f32 {
    (point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2) + (point2.z - point1.z).powi(2)
}
