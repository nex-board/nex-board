use bevy::{
    camera::visibility::NoFrustumCulling,
    color::palettes::{css::BLACK, tailwind::YELLOW_300},
    prelude::*,
    text::{TextLayout, TextLayoutInfo},
};

struct TextSource {
    contents: String,
    duration: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TextQueue {
            texts: vec![
                TextSource {
                    contents: "大阪公立大学工業高等専門学校".to_string(),
                    duration: 5.0,
                },
                TextSource {
                    contents: "学友会執行委員会 情報通信課".to_string(),
                    duration: 8.0,
                },
                TextSource {
                    contents: "学友会執行委員会 総務課展示電気係".to_string(),
                    duration: 5.0,
                },
                TextSource {
                    contents: "学友会執行委員会 音響課".to_string(),
                    duration: 2.0,
                },
            ],
            current_index: 0,
        })
        .init_resource::<ScrollingState>()
        .init_resource::<ScrollingSpeed>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mouse_click)
        .add_systems(Update, text_scroll)
        .add_systems(Update, check_text_completion)
        .run();
}

#[derive(Component)]
struct TextScroll;

#[derive(Component)]
struct ScrollingActive;

#[derive(Resource)]
struct TextQueue {
    texts: Vec<TextSource>,
    current_index: usize,
}

#[derive(Resource, Default)]
struct ScrollingState {
    is_active: bool,
}

#[derive(Resource, Default)]
struct ScrollingSpeed {
    speed: f32,
}

fn setup(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    text_queue: Res<TextQueue>,
    mut scrolling_speed: ResMut<ScrollingSpeed>,
) {
    let font = asset_server.load("fonts/ipag.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 1080.0,
        ..default()
    };
    cmds.spawn(Camera2d);

    // 最初のテキストを表示（スクロールは無効状態で開始）
    spawn_text(
        &mut cmds,
        &text_queue.texts[0].contents,
        &text_queue.texts[0].duration,
        text_font,
        &mut *scrolling_speed,
    );
}

fn spawn_text(
    cmds: &mut Commands,
    text: &str,
    duration: &f32,
    text_font: TextFont,
    scrolling_speed: &mut ScrollingSpeed,
) {
    let text_offset = calc_text_offset(text);
    println!("Offset: {}", text_offset);
    cmds.spawn((
        Text2d::new(text),
        text_font,
        TextColor(Color::Srgba(YELLOW_300)),
        TextBackgroundColor(BLACK.into()),
        Transform::from_translation(Vec3::new(text_offset, 0.0, 0.0)),
        TextLayout::default(),
        TextScroll,
    ))
    .insert(NoFrustumCulling);
    scrolling_speed.speed = calc_speed(text_offset * 2.0, duration);
}

fn text_scroll(
    time: Res<Time>,
    scrolling_state: Res<ScrollingState>,
    scrolling_speed: Res<ScrollingSpeed>,
    mut query: Query<&mut Transform, (With<TextScroll>, With<ScrollingActive>)>,
) {
    if !scrolling_state.is_active {
        return;
    }

    for mut transform in &mut query {
        transform.translation.x -= scrolling_speed.speed * time.delta_secs()
    }
}

fn handle_mouse_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut scrolling_state: ResMut<ScrollingState>,
    mut text_queue: ResMut<TextQueue>,
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    waiting_text_query: Query<Entity, (With<TextScroll>, Without<ScrollingActive>)>,
    active_text_query: Query<Entity, (With<TextScroll>, With<ScrollingActive>)>,
    mut scrolling_speed: ResMut<ScrollingSpeed>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if !scrolling_state.is_active {
            // 待機中のテキストがある場合、スクロール開始
            scrolling_state.is_active = true;

            for entity in waiting_text_query.iter() {
                cmds.entity(entity).insert(ScrollingActive);
            }

            println!("スクロール開始！");
        } else {
            // スクロール中の場合、現在のテキストを削除して次のテキストを表示
            for entity in active_text_query.iter() {
                cmds.entity(entity).despawn();
            }

            // 次のテキストインデックスに進む
            text_queue.current_index = (text_queue.current_index + 1) % text_queue.texts.len();

            // スクロールを停止
            scrolling_state.is_active = false;

            // 次のテキストを表示
            let font = asset_server.load("fonts/ipag.ttf");
            let text_font = TextFont {
                font: font.clone(),
                font_size: 1080.0,
                ..default()
            };
            spawn_text(
                &mut cmds,
                &text_queue.texts[text_queue.current_index].contents.clone(),
                &text_queue.texts[text_queue.current_index].duration,
                text_font,
                &mut *scrolling_speed,
            );

            println!(
                "次のテキストにスキップ: {}",
                text_queue.texts[text_queue.current_index].contents
            );
        }
    }
}

fn check_text_completion(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    mut text_queue: ResMut<TextQueue>,
    mut scrolling_state: ResMut<ScrollingState>,
    query: Query<(Entity, &Transform, &TextLayoutInfo), (With<TextScroll>, With<ScrollingActive>)>,
    mut scrolling_speed: ResMut<ScrollingSpeed>,
) {
    let font = asset_server.load("fonts/ipag.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 1080.0,
        ..default()
    };

    for (entity, transform, info) in query.iter() {
        let text_width = info.size.x;
        let text_left_edge = transform.translation.x + text_width / 2.0 + 1300.0;

        // テキストが完全に画面左端を通り過ぎたかチェック（テキスト全体が画面外に出るまで待つ）
        if text_left_edge < 0.0 {
            // 現在のテキストエンティティを削除
            cmds.entity(entity).despawn();

            // 次のテキストインデックスに進む
            text_queue.current_index = (text_queue.current_index + 1) % text_queue.texts.len();

            // スクロールを停止
            scrolling_state.is_active = false;

            // 次のテキストを表示
            spawn_text(
                &mut cmds,
                &text_queue.texts[text_queue.current_index].contents.clone(),
                &text_queue.texts[text_queue.current_index].duration,
                text_font,
                &mut *scrolling_speed,
            );

            println!(
                "Next: {} ",
                text_queue.texts[text_queue.current_index].contents
            );
            break; // 一度に一つのテキストのみ処理
        }
    }
}

fn count_ascii(s: &str) -> (usize, usize) {
    s.chars().fold((0, 0), |mut acc, c| {
        if c.is_ascii() {
            acc.0 += 1;
        } else {
            acc.1 += 1;
        }
        acc
    })
}

fn calc_text_offset(s: &str) -> f32 {
    let (ascii, non_ascii) = count_ascii(s);
    println!("{}", non_ascii * 2 + ascii);
    return ((non_ascii * 2 + ascii) as f32 * 580.0 / 2.0) + (1980.0 / 2.0);
}

fn calc_speed(w: f32, d: &f32) -> f32 {
    return (w + 1080.0) / d;
}
