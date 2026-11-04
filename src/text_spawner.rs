use crate::{Fonts, LoopingText, ScrollingSpeed, Showing, TextScroll, loader::Config};
use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::tailwind::{SLATE_900, YELLOW_300},
    time::{TimerMode, Timer},
    prelude::*,
};

#[derive(Component)]
pub struct Automated {
    timer: Timer,
}

#[derive(Component)]
pub struct JustAdded;

#[derive(Resource)]
pub struct AutoCueTimer {
    timer: Timer,
}

impl AutoCueTimer {
    pub fn new(time: f32) -> Self {
        Self {
            timer: Timer::from_seconds(time, TimerMode::Once),
        }
    }
}

pub fn spawn_text(
    cmds: &mut Commands,
    text: &str,
    duration: &f32,
    color: &str,
    text_font: TextFont,
    config: &Config,
    scrolling_speed: &mut ScrollingSpeed,
) -> Entity {
    let text_offset = crate::text::calc_text_offset(text, config.text_size, config.window_width);
    println!("Offset: {}, Duration: {}", text_offset, duration);
    
    // duration が 1000 以上の場合はループ再生
    if *duration >= 1000.0 {
        let text_width = text_offset * 2.0; // 概算のテキスト幅
        let loop_speed = 500.0; // 500px/s
        
        cmds.spawn((
            Text2d::new(text),
            text_font,
            TextColor(Color::Srgba(Srgba::hex(color).unwrap_or(YELLOW_300))),
            TextBackgroundColor(Color::Srgba(SLATE_900)),
            Transform::from_translation(Vec3::new(config.window_width / 2.0 + text_width / 2.0 + 50.0, 0.0, 0.0)),
            TextLayout::default(),
            LoopingText {
                original_x: text_offset,
                text_width,
                loop_speed,
            },
            Showing,
            JustAdded,
        ))
            .insert(NoFrustumCulling)
            .id()
    } else {
        scrolling_speed.speed = crate::text::calc_speed_from_text(text, config.text_size, duration, config.window_width);
        cmds.spawn((
            Text2d::new(text),
            text_font,
            TextColor(Color::Srgba(Srgba::hex(color).unwrap_or(YELLOW_300))),
            TextBackgroundColor(Color::Srgba(SLATE_900)),
            Transform::from_translation(Vec3::new(text_offset, 0.0, 0.0)),
            TextLayout::default(),
            TextScroll,
            Showing,
            JustAdded,
        ))
            .insert(NoFrustumCulling)
            .id()
    }
}

pub fn spawn_static_text(
    cmds: &mut Commands,
    text: &str,
    color: &str,
    text_font: TextFont,
) -> Entity {
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(Srgba::hex(color).unwrap_or(YELLOW_300))),
        TextBackgroundColor(Color::Srgba(SLATE_900)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        TextLayout::default(),
        Showing,
        JustAdded,
    ))
        .insert(NoFrustumCulling)
        .id()
}

pub fn spawn_queued_text(
    mut cmds: &mut Commands,
    text_queue: &mut crate::TextQueue,
    text_query: &Query<Entity, With<Showing>>,
    auto_cue_timer: &mut AutoCueTimer,
    font: TextFont,
    config: Config,
    mut scrolling_speed: &mut ScrollingSpeed,
)  {
    for entity in text_query.iter() {
        cmds.entity(entity).despawn();
    }

    if let Some(text_source) = text_queue.texts.get(text_queue.current_index) {
        let mut added_entity: Entity;
        
        match text_source.duration {
            0.0 => {
                added_entity = spawn_static_text(&mut cmds, &text_source.content, &text_source.color, font);
            },
            _ => {
                added_entity = spawn_text(&mut cmds, &text_source.content, &text_source.duration, &text_source.color, font, &config, &mut scrolling_speed);
            }
        }
        if text_source.interval.unwrap_or(0.0) > 0.0 {
            cmds.entity(added_entity).insert(Automated {
                timer: Timer::from_seconds(text_source.interval.unwrap(), TimerMode::Once)
            });
        }
    }
}
