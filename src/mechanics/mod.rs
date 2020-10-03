pub mod character;
pub mod constants;

use bevy::prelude::*;

use character::CharacterCommand;
use crate::map::Map;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
	North,
	East,
	South,
	West,
}

pub type Ordinate = isize;

// height is a derived property
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct GridPosition {
	pub x: Ordinate,
	pub y: Ordinate,
}

pub struct DisplayGridPosition(pub GridPosition);

impl GridPosition {
	pub fn roll(ordinate: Ordinate, map_w: Ordinate) -> Self {
		Self {
			x: ordinate / map_w,
			y: ordinate % map_w,
		}
	}

	pub fn unroll(self, map_w: Ordinate) -> Ordinate {
		self.x + (self.y * map_w)
	}

	pub fn destination(self, action: CharacterCommand) -> Self {
		match action {
			CharacterCommand::Move(d) => self.neighbour(d),
			_ => self,
		}
	}

	pub fn neighbour(mut self, direction: Direction) -> Self {
		match direction {
			Direction::North => {self.x += 1;},
			Direction::South => {self.x -= 1;},
			Direction::East => {self.y += 1;},
			Direction::West => {self.y -= 1;},
		}

		self
	}
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_system(display_pos_to_world.system());
	}
}

pub struct MechanicsPlugin;

impl Plugin for MechanicsPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_plugin(RenderPlugin)
			.add_plugin(character::CharacterPlugin)
			.add_resource(TurnLimit(7))
			.add_resource(ActiveTurn::default());
	}
}

#[derive(Clone, Copy, Debug, Default, Properties)]
pub struct TurnLimit(pub usize);

#[derive(Clone, Copy, Debug, Default, Properties)]
pub struct ActiveTurn {
	active_ent_refresh: usize,
	active_ent: usize,
	pub turn: usize,
}

impl ActiveTurn {
	pub fn should_reset(&self, limit: usize) -> bool {
		self.turn == limit
	}

	pub fn allow_turn(&self, limit: usize, my_subturn: usize) -> bool {
		self.turn != limit && my_subturn == self.active_ent
	}

	pub fn march_turn(&mut self) {
		if self.active_ent == 0 {
			self.active_ent = self.active_ent_refresh;
			self.turn += 1;
		} else {
			self.active_ent -= 1;
		}
	}

	pub fn reset_and_add_ent(&mut self) {
		self.active_ent_refresh += 1;
		self.active_ent = self.active_ent_refresh;
		self.turn = 0;
	}
}

fn display_pos_to_world(
	mut map_query: Query<&Map>,
	mut query: Query<(&DisplayGridPosition, &mut Transform)>,
) {
	for map in &mut map_query.iter() {
		for (pos, mut transform) in &mut query.iter() {
			let pos = pos.0;
			let height = map.heights[pos.unroll(map.width) as usize];
			transform.set_translation(Vec3::new(
				-pos.y as f32,
				height as f32 + 0.5,
				pos.x as f32,
			));
		}
	}
}
