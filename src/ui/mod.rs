use bevy::{
	diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
	prelude::*,
};
use crate::mechanics::GhostLimit;
use crate::{
	mechanics::{ActiveTurn, TurnLimit},
};

#[derive(Debug, Default)]
pub struct FpsCounter {
	pub enabled: bool
}

fn fps_update_system(
	diagnostics: Res<Diagnostics>,
	mut query: Query<(&FpsCounter, &mut Text)>,
) {
	for (ctl, mut text) in &mut query.iter() {
		if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
			if ctl.enabled {
				if let Some(average) = fps.average() {
					text.value = format!("FPS: {:.2}", average);
				}
			} else {
				if !text.value.is_empty() {
					text.value = String::new();
				}
			}
		}
	}
}

fn fps_control_system(
	key_input: Res<Input<KeyCode>>,
	mut query: Query<&mut FpsCounter>,
) {

	if key_input.just_pressed(KeyCode::F1) {
		for mut counter in &mut query.iter() {
			counter.enabled = !counter.enabled;
		}
	}
}

#[derive(Debug, Default)]
pub struct TurnCounter;

fn turn_system(
	limit: Res<TurnLimit>,
	turn: Res<ActiveTurn>,
	mut query: Query<(&TurnCounter, &mut Text)>,
) {
	let target_turn = turn.turn + 1;
	for (_ctl, mut text) in &mut query.iter() {
		text.value = format!("{}/{}", target_turn.min(limit.0), limit.0);
	}
}


#[derive(Debug, Default)]
pub struct GhostCounter;

#[derive(Debug, Default)]
pub struct TopLevel;

fn reruns_system(
	limit: Res<GhostLimit>,
	mut query: Query<(&GhostCounter, &mut Text)>,
) {
	for (_ctl, mut text) in &mut query.iter() {
		text.value = format!("{}", limit.0);
	}	
}

fn reruns_recolour_system(
	limit: Res<GhostLimit>,
	mut query: Query<(&GhostCounter, &TopLevel, &mut Text)>,
) {
	for (_ctl, _tag, mut text) in &mut query.iter() {
		text.style.color = match limit.0 {
			0 => Color::RED,
			1 => Color::rgb(1.0, 0.5, 0.0),
			_ => Color::WHITE,
		};
	}	
}

pub fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let font_handle = asset_server.load("assets/fonts/as/AlegreyaSans-Bold.ttf").unwrap();

	commands
		.spawn(UiCameraComponents::default())
		// FPS counter.
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				..Default::default()
			},
			text: Text {
				value: "".to_string(),
				font: font_handle,
				style: TextStyle {
					font_size:60.0,
					color: Color::WHITE,
				}
			},
			..Default::default()
		})
		.with(FpsCounter { enabled: false })
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexStart,
				position_type: PositionType::Absolute,
				position: Rect {
					left: Val::Px(5.0),
					bottom: Val::Px(5.0),
					..Default::default()
				},
				..Default::default()
			},
			text: Text {
				value: "".to_string(),
				font: font_handle,
				style: TextStyle {
					font_size:60.0,
					color: Color::BLACK,
				}
			},
			..Default::default()
		})
		.with(TurnCounter)
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexStart,
				position_type: PositionType::Absolute,
				position: Rect {
					left: Val::Px(2.0),
					bottom: Val::Px(2.0),
					..Default::default()
				},
				..Default::default()
			},
			text: Text {
				value: "".to_string(),
				font: font_handle,
				style: TextStyle {
					font_size: 60.0,
					color: Color::WHITE,
				}
			},
			..Default::default()
		})
		.with(TurnCounter)
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				position_type: PositionType::Absolute,
				position: Rect {
					right: Val::Px(2.0),
					bottom: Val::Px(2.0),
					..Default::default()
				},
				..Default::default()
			},
			text: Text {
				value: "".to_string(),
				font: font_handle,
				style: TextStyle {
					font_size:60.0,
					color: Color::BLACK,
				}
			},
			..Default::default()
		})
		.with(GhostCounter)
		.spawn(TextComponents {
			style: Style {
				align_self: AlignSelf::FlexEnd,
				position_type: PositionType::Absolute,
				position: Rect {
					right: Val::Px(4.0),
					bottom: Val::Px(4.0),
					..Default::default()
				},
				..Default::default()
			},
			text: Text {
				value: "".to_string(),
				font: font_handle,
				style: TextStyle {
					font_size:60.0,
					color: Color::WHITE,
				}
			},
			..Default::default()
		})
		.with(GhostCounter)
		.with(TopLevel);
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(FrameTimeDiagnosticsPlugin::default())
			.add_startup_system(setup.system())
			.add_system(fps_control_system.system())
			.add_system(fps_update_system.system())
			.add_system(turn_system.system())
			.add_system(reruns_system.system())
			.add_system(reruns_recolour_system.system());
	}
}
