use bevy::prelude::*;
use numerals::roman::Roman;

use crate::map::meta::Levels;

use super::Alive;
use super::camera::CameraMode;
use super::ender::trigger_restart;

pub struct EventPlugin;

impl Plugin for EventPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_event::<Restart>()
			.add_event::<LevelStart>()
			.add_event::<LevelExit>()
			.add_event::<DoLevelGen>()
			.add_event::<SpawnLevelText>()
			.add_system(handle_restart.system())
			.add_system(handle_start.system())
			.add_system(handle_exit.system())
			.add_system(debug_sender.system())
			.add_system(handle_dolevelgen.system());
	}
}

pub struct Restart;

fn handle_restart(
	evts: Res<Events<Restart>>,
	mut mode: ResMut<CameraMode>,
) {
	for _evt in evts.get_reader().iter(&evts) {
		mode.zoom_out_restart();
	}
}

pub struct LevelExit;

fn handle_exit(
	evts: Res<Events<LevelExit>>,
	mut mode: ResMut<CameraMode>,
) {
	for _evt in evts.get_reader().iter(&evts) {
		mode.zoom_out_next();
	}
}

pub struct LevelStart;

fn handle_start(
	evts: Res<Events<LevelStart>>,
	mut mode: ResMut<CameraMode>,
) {
	for _evt in evts.get_reader().iter(&evts) {
		mode.zoom_in();
	}
}

pub struct DoLevelGen;

fn handle_dolevelgen(
	levels: Res<Levels>,
	evts: Res<Events<DoLevelGen>>,
	mut textevts: ResMut<Events<SpawnLevelText>>,
	mut ents_query: Query<&mut Alive>,
) {
	for _evt in evts.get_reader().iter(&evts) {
		trigger_restart(&mut ents_query);

		if let Some(level) = levels.data.get(levels.start_at) {
			let romanify: Roman = (levels.start_at as i16 + 1).into();
			textevts.send(SpawnLevelText(format!("{:X}: {}", romanify, level.name)));
		}
	}
}

pub struct SpawnLevelText(pub String);

fn debug_sender(
	mut sta: ResMut<Events<LevelStart>>,
	mut res: ResMut<Events<Restart>>,
	mut exits: ResMut<Events<LevelExit>>,
	input: Res<Input<KeyCode>>,
) {
	if input.just_pressed(KeyCode::Q) {
		sta.send(LevelStart);
	}

	if input.just_pressed(KeyCode::W) {
		res.send(Restart);
	}

	if input.just_pressed(KeyCode::E) {
		exits.send(LevelExit);
	}
}
