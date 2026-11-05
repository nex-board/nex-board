use crate::{Fonts, LoopingText, ScrollingSpeed, ScrollingState, Showing, TextQueue, TextScroll, loader::Config, server::{BulletinResponse, WebSocketChannel, WsResponse}};
use axum::http::response;
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
    ))
        .insert(NoFrustumCulling)
        .id()
}

pub fn spawn_queued_text(
    mut cmds: &mut Commands,
    text_queue: &mut crate::TextQueue,
    text_query: &Query<Entity, With<Showing>>,
    font: TextFont,
    config: &Config,
    mut scrolling_speed: &mut ScrollingSpeed,
)  {
    for entity in text_query.iter() {
        cmds.entity(entity).despawn();
    }

    if let Some(text_source) = text_queue.texts.get(text_queue.current_index) {
        let added_entity: Entity;
        
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

pub fn auto_cue_system (
    mut cmds: Commands,
    mut text_queue: ResMut<TextQueue>,
    text_query: Query<Entity, With<Showing>>,
    mut auto_cue_query: Query<&mut Automated>,
    fonts: Res<Fonts>,
    config: Res<Config>,
    time: Res<Time>,
    mut scrolling_speed: ResMut<ScrollingSpeed>,
    mut scrolling_state: ResMut<ScrollingState>,
    ws_channel: ResMut<WebSocketChannel>,
) {
    for mut automated in auto_cue_query.iter_mut() {
        automated.timer.tick(time.delta());
        println!("{}", automated.timer.elapsed_secs());
        if automated.timer.just_finished() {
            println!("Finish");
            text_queue.current_index += 1;
            spawn_queued_text(&mut cmds, &mut text_queue, &text_query, fonts.text_font.clone(), &*config, &mut *scrolling_speed);
            if text_queue.texts[text_queue.current_index].duration > 0.0 {
                scrolling_state.is_active = true;
            }

            let prev_text = text_queue.texts.get(text_queue.current_index.saturating_sub(1) as usize).map(|t| t.content.clone()).unwrap_or_default();
            let now_text = text_queue.texts.get(text_queue.current_index).map(|t| t.content.clone()).unwrap_or_default();
            let next_text = text_queue.texts.get(text_queue.current_index + 1 as usize).map(|t| t.content.clone()).unwrap_or_default();

            let response = WsResponse::Bulletin(BulletinResponse {
                prev_text,
                now_text,
                next_text
            });

            let _ = ws_channel.response_sender.send(response);
        }
    }
}
