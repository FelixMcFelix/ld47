pub mod buttons;
pub mod camera;
pub mod character;
pub mod constants;
pub mod ender;
pub mod events;
pub mod spawner;

use bevy::prelude::*;

use camera::CameraPlugin;
use character::CharacterCommand;
use crate::map::Map;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
	North,
	East,
	South,
	West,
}

pub type Ordinate = isize;

// height is a derived property
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct GridPosition {
	pub x: Ordinate,
	pub y: Ordinate,
}

pub struct DisplayGridPosition(pub GridPosition);
pub struct CollideGridPosition(pub GridPosition);

pub struct Alive(pub bool);

impl Default for Alive {
	fn default() -> Self {
	    Self(true)
	}
}

fn despawn_if_not_alive(
	mut commands: Commands,
	mut living: Query<(Entity, &Alive)>,
) {
	for (ent, alive) in &mut living.iter() {
		if !alive.0 {
			commands.despawn_recursive(ent);
		}
	}
}

impl GridPosition {
	pub fn clamp(self, w: Ordinate, h: Ordinate) -> Self {
		Self {
			x: self.x.max(0).min(w-1),
			y: self.y.max(0).min(h-1),
		}
	}

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

pub struct CameraFaced;
pub struct CameraFacer;

fn camera_facer(
	mut cameras: Query<(&CameraFaced, &Transform)>,
	mut query: Query<(&CameraFacer, &mut Transform)>,
) {
	for (_camera, camera_tx) in &mut cameras.iter() {
		let face_axis = camera_tx.value().z_axis();
		let rot_angle = face_axis.z().atan2(face_axis.x());
		for (_tag, mut ent_tx) in &mut query.iter() {
			ent_tx.set_rotation(Quat::from_rotation_y(rot_angle));
		}
	}
}

#[derive(Debug, Default)]
pub struct OccupationMap(pub Vec<bool>);

impl OccupationMap {
	pub fn move_collider(&mut self, from: usize, to: usize) {
		self.0[from] = false;
		self.0[to] = true;
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
			.add_plugin(events::EventPlugin)
			.add_plugin(CameraPlugin)
			.add_resource(OccupationMap::default())
			.add_system(collision_populater.system())
			.add_system(despawn_if_not_alive.system())
			.add_plugin(character::CharacterPlugin)
			.add_plugin(spawner::SpawnerPlugin)
			.add_plugin(ender::EnderPlugin)
			.add_plugin(buttons::ButtonPlugin)
			.add_system(camera_facer.system())
			.add_resource(TurnLimit(1))
			.add_resource(GhostLimit(1))
			.add_resource(ActiveTurn::default())
			.add_system(turn_lockout.system());
	}
}

#[derive(Clone, Copy, Debug, Default, Properties, Deserialize, Serialize)]
pub struct TurnLimit(pub usize);

#[derive(Clone, Copy, Debug, Default, Properties, Deserialize, Serialize)]
pub struct GhostLimit(pub usize);

#[derive(Clone, Copy, Debug, Default, Properties)]
pub struct ActiveTurn {
	active_ent_refresh: usize,
	active_ent: usize,
	pub turn: usize,
	block_turn: bool,
}

impl ActiveTurn {
	pub fn should_reset(&self, limit: usize) -> bool {
		self.turn == limit
	}

	pub fn allow_turn(&self, limit: usize, my_subturn: usize) -> bool {
		!self.block_turn && self.turn != limit && my_subturn == self.active_ent
	}

	pub fn march_turn(&mut self) {
		if self.active_ent == 0 {
			self.active_ent = self.active_ent_refresh;
			self.turn += 1;
		} else {
			self.active_ent -= 1;
		}

		self.block_turn = false;
	}

	pub fn reset_and_add_ent(&mut self) {
		self.active_ent_refresh += 1;
		self.active_ent = self.active_ent_refresh;
		self.turn = 0;
	}

	pub fn reinit(&mut self) {
		*self = Default::default();
	}
}

fn turn_lockout(
	mut turn: ResMut<ActiveTurn>,
) {
	turn.block_turn = false;
}

fn collision_populater(
	mut occupation: ResMut<OccupationMap>,
	mut map_query: Query<&Map>,
	mut query: Query<&CollideGridPosition>,
) {
	for map in &mut map_query.iter() {
		let o_len = occupation.0.len();
		occupation.0.resize(map.len(), false);
		occupation.0[..o_len].iter_mut()
			.map(|x| *x = false)
			.count();

		for pos in &mut query.iter() {
			occupation.0[pos.0.unroll(map.width) as usize] = true;
		}
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
