use bevy::prelude::Color;

/// Remaps a value from one range to another range.
pub(crate) fn remap(value: f32, original_min: f32, original_max: f32, target_min: f32, target_max: f32) -> f32 {
    target_min + (value - original_min) * (target_max - target_min) / (original_max - original_min)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

pub(crate) fn lerp_color( a: Color, b: Color, t: f32) -> Color {
    Color::rgb(
        lerp(a.r(), b.r(), t),
        lerp(a.g(), b.g(), t),
        lerp(a.b(), b.b(), t),
    )
}