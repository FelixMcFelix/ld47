use std::time::Duration;

use bevy::{
	diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
	prelude::*,
};
use crate::mechanics::GhostLimit;
use crate::mechanics::events::SpawnLevelText;
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
		.with(TopLevel)
		.spawn(TextComponents {
			style: Style {
				// align_self: AlignSelf::Center,
				position_type: PositionType::Absolute,
				// align_content: AlignContent::Center,
				// justify_content: JustifyContent::Center,
				// size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
				position: Rect {
					top: Val::Percent(20.5),
					left: Val::Percent(25.0),
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
		.with(FadeInOut::level_text())
		.with(LevelText)
		.spawn(TextComponents {
			style: Style {
				// align_self: AlignSelf::Center,
				position_type: PositionType::Absolute,
				// align_content: AlignContent::Center,
				// justify_content: JustifyContent::Center,
				// size: Size::new(Val::Percent(100.0), Val::Percent(99.0)),
				position: Rect {
					top: Val::Percent(20.0),
					left: Val::Percent(25.0),
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
		.with(FadeInOut::level_text())
		.with(LevelText);
}

pub struct LevelText;

const LEVEL_TEXT_FADE_IN_TIME: Duration = Duration::from_millis(500);
const LEVEL_TEXT_FADE_HOLD_TIME: Duration = Duration::from_secs(3);
const LEVEL_TEXT_FADE_OUT_TIME: Duration = Duration::from_millis(1500);

fn display_level_name(
	evts: Res<Events<SpawnLevelText>>,
	mut query: Query<(&LevelText, &mut Text, &mut FadeInOut)>,
) {
	for evt in evts.get_reader().iter(&evts) {
		let text_to_show = &evt.0;
		for (_tag, mut el, mut fade_state) in &mut query.iter() {
			el.value = text_to_show.clone();
			fade_state.start();
		}
	}
}

#[derive(Clone, Debug)]
struct FadeInOut {
	inn: Duration,
	hold: Duration,
	out: Duration,
	active: Option<(FadeComponent, Timer)>,
}

#[derive(Copy, Clone, Debug)]
enum FadeComponent {
	In, Hold, Out,
}

impl FadeInOut {
	fn new(inn: Duration, hold: Duration, out: Duration) -> Self {
		Self {
			inn,
			hold,
			out,
			active: None,
		}
	}

	fn level_text() -> Self {
		Self::new(
			LEVEL_TEXT_FADE_IN_TIME,
			LEVEL_TEXT_FADE_HOLD_TIME,
			LEVEL_TEXT_FADE_OUT_TIME
		)
	}

	fn start(&mut self) {
		self.active = Some((FadeComponent::In, Timer::new(self.inn, false)));
	}

	fn tick(&mut self, time: &Time) {
		let mut erase = false;
		if let Some((ref mut state, ref mut timer)) = &mut self.active {
			timer.tick(time.delta_seconds);

			if timer.just_finished {
				match state {
					FadeComponent::In => {
						*timer = Timer::new(self.hold, false);
						*state = FadeComponent::Hold;
					},
					FadeComponent::Hold => {
						*timer = Timer::new(self.out, false);
						*state = FadeComponent::Out;
					},
					FadeComponent::Out => {
						erase = true;
					},
				}
			}
		}

		if erase {
			self.active = None;
		}
	}

	fn opacity(&self) -> f32 {
		match &self.active {
			None => 0.0,
			Some((FadeComponent::Hold, _timer)) => 1.0,
			Some((FadeComponent::In, timer)) => timer.elapsed / timer.duration,
			Some((FadeComponent::Out, timer)) => 1.0 - (timer.elapsed / timer.duration),
		}
	}
}

fn ui_fade_in_out_tick_system(
	time: Res<Time>,
	mut fader: Query<&mut FadeInOut>,
) {
	for mut el in &mut fader.iter() {
		el.tick(&time);
	}
}

fn ui_fade_in_out_system(
	mut fader: Query<(&mut Text, &FadeInOut)>,
) {
	for (mut el, fade_state) in &mut fader.iter() {
		el.style.color.a = fade_state.opacity();
	}
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
			.add_system(reruns_recolour_system.system())
			.add_system(display_level_name.system())
			.add_system(ui_fade_in_out_tick_system.system())
			.add_system(ui_fade_in_out_system.system());
	}
}
