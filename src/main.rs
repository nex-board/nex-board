use bevy::{
    camera::visibility::NoFrustumCulling, color::palettes::{css::BLACK, tailwind::YELLOW_300}, prelude::*
};

fn main() {
    App::new()
	.add_plugins(DefaultPlugins)
	.add_systems(Startup, setup)
	.add_systems(Update, animate_scroll)
	.run();
}

#[derive(Component)]
struct AnimateScroll;

fn setup(mut cmds: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/ipaexg.ttf");
    let text_font = TextFont {
	font: font.clone(),
	font_size: 1080.0,
	..default()
    };
    cmds.spawn(Camera2d);
    cmds.spawn((
	Text2d::new("学友会執行委員会 情報通信課"),
	text_font.clone(),
	TextColor(Color::Srgba(YELLOW_300)),
	TextBackgroundColor(BLACK.into()),
	Transform::from_xyz(10000.0, 0.0, 0.0),
	AnimateScroll,
    ))
    .insert(NoFrustumCulling);
}

fn animate_scroll(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<AnimateScroll>>,
) {
    for mut transform in &mut query {
	transform.translation.x -= 5000.0 * time.delta_secs()
    }
}
    
