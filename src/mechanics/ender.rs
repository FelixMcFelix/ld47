use bevy::prelude::*;

use crate::map::meta::Levels;

use super::Alive;
use super::DisplayGridPosition;
use super::CollideGridPosition;
use super::character::Character;

#[derive(Debug, Default,)]
pub struct Ender{ fired: bool }

pub struct EnderPlugin;

impl Plugin for EnderPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_system(ender_progresses_level.system());
	}
}

fn ender_progresses_level(
	mut level_info: ResMut<Levels>,
	mut query: Query<(&mut Ender, &DisplayGridPosition)>,
	mut chars_query: Query<(&Character, &CollideGridPosition)>,
	mut ents_query: Query<&mut Alive>,
) {
	let mut do_end = None;
	for (mut ender, pos) in &mut query.iter() {
		if ender.fired {
			continue;
		}

		for (_char, char_pos) in &mut chars_query.iter() {
			if char_pos.0 == pos.0 {
				do_end = Some(pos.0.clone());
				ender.fired = true;
			}
		}
	}

	if let Some(_end_pos) = do_end {
		println!("Done!");
		//despawn all
		for mut alive in &mut ents_query.iter() {
			alive.0 = false;
		}

		// increment map.
		level_info.load_next();
	}
}
