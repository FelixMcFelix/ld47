use bevy::{
	diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
	prelude::*,
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

pub fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
) {
	let font_handle = asset_server.load("assets/fonts/as/AlegreyaSans-Bold.ttf").unwrap();

	commands
		.spawn(UiCameraComponents::default())
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
		.with(FpsCounter { enabled: false });
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(FrameTimeDiagnosticsPlugin::default())
			.add_startup_system(setup.system())
			.add_system(fps_control_system.system())
			.add_system(fps_update_system.system());
	}
}
