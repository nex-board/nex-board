use crate::{loader::Config, ScrollingSpeed, Showing, TextScroll};
use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::tailwind::{SLATE_900, YELLOW_300},
    prelude::*,
};

pub fn spawn_text(
    cmds: &mut Commands,
    text: &str,
    duration: &f32,
    text_font: TextFont,
    config: &Config,
    scrolling_speed: &mut ScrollingSpeed,
) {
    let text_offset = crate::text::calc_text_offset(text, config.text_size, config.window_width);
    println!("Offset: {}", text_offset);
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(Color::Srgba(SLATE_900)),
        Transform::from_translation(Vec3::new(text_offset, 0.0, 0.0)),
        TextLayout::default(),
        TextScroll,
        Showing,
    ))
    .insert(NoFrustumCulling);
    scrolling_speed.speed =
        crate::text::calc_speed(text_offset * 2.0, duration, config.window_width);
}

pub fn spawn_static_text(
    cmds: &mut Commands,
    text: &str,
    text_font: TextFont,
) {
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(Color::Srgba(SLATE_900)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        TextLayout::default(),
        Showing,
    ))
    .insert(NoFrustumCulling);
}
