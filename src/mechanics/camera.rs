use super::events::DoLevelGen;
use super::{
	character::ActiveCharacter,
	CameraFaced, DisplayGridPosition, GridPosition
};
use crate::map::Map;
use crate::map::WORLD_HEIGHT_SCALE;

use std::time::Duration;

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(CameraDest::default())
			.add_resource(CameraMode(CameraState::Normal))
			.add_system(slide_camera_to_dest.system())
			.add_system(active_char_is_camera_dest.system())
			.add_system(tick_camera_mode.system());
	}
}

const ZOOM_IN_TIME: Duration = Duration::from_secs(2);
const ZOOM_IN_ROT_COUNT: f32 = 1.0;
const ZOOM_OUT_TIME: Duration = Duration::from_secs(1);
const ZOOM_OUT_ROT_COUNT: f32 = 1.0;
const ZOOM_HOLD_TIME: Duration = Duration::from_secs(3);

const ZOOM_EXTRA_HEIGHT: f32 = 15.0;

enum CameraState {
	Normal,
	Zoomin(Timer),
	Zoomout(Timer, bool),
	Hold(Timer),
}

impl CameraState {
	fn normal() -> Self {
		CameraState::Normal
	}

	fn zoom_in() -> Self {
		CameraState::Zoomin(Timer::new(ZOOM_IN_TIME, false))
	}

	fn zoom_out() -> Self {
		CameraState::Zoomout(Timer::new(ZOOM_OUT_TIME, false), false)
	}

	fn zoom_out_hold() -> Self {
		CameraState::Zoomout(Timer::new(ZOOM_OUT_TIME, false), true)
	}

	fn hold() -> Self {
		CameraState::Hold(Timer::new(ZOOM_HOLD_TIME, false))
	}

	fn tick(&mut self, time: &Time) -> bool {
		match self {
			CameraState::Zoomin(a) => {
				a.tick(time.delta_seconds);
				a.just_finished
			},
			CameraState::Zoomout(a, _) => {
				a.tick(time.delta_seconds);
				a.just_finished
			},
			CameraState::Hold(a) => {
				a.tick(time.delta_seconds);
				a.just_finished
			},
			_ => false,
		}
	}

	fn next(&mut self, time: &Time) -> Option<Self> {
		use CameraState::*;
		if self.tick(time) {
			Some(match self {
				Zoomin(_) => CameraState::normal(),
				Hold(_) | Zoomout(_, false) => CameraState::zoom_in(),
				Zoomout(_, true) => CameraState::hold(),
				_ => unreachable!(),
			})
		} else {
			None
		}
	}

	fn allow_pan(&self) -> bool {
		match self {
			CameraState::Normal => true,
			_ => false,
		}
	}

	// Returns new lookat_offset, eye.
	fn camera_mods(&self, usual_eye_offset: Vec3) -> (Vec3, Vec3, bool) {
		let mut height_to_add = 0.0;
		let mut y_rotation_amount = 0.0;
		let mut hard_set = true;

		use CameraState::*;
		match self {
			Normal => { hard_set = false; },
			Hold(_) => {
				height_to_add = ZOOM_EXTRA_HEIGHT;
			},
			Zoomout(a, _) => {
				if a.finished {
					height_to_add = ZOOM_EXTRA_HEIGHT;
				} else {
					let frac_elapsed = a.elapsed / a.duration;

					height_to_add = frac_elapsed * ZOOM_EXTRA_HEIGHT;
					y_rotation_amount = -(frac_elapsed * 2.0 * std::f32::consts::PI * ZOOM_OUT_ROT_COUNT);
				}
			},
			Zoomin(a) => {
				if a.finished {
					
				} else {
					let frac_elapsed = a.elapsed / a.duration;
					let rec = 1.0 - frac_elapsed;

					height_to_add = rec * ZOOM_EXTRA_HEIGHT;
					y_rotation_amount = rec * 2.0 * std::f32::consts::PI * ZOOM_IN_ROT_COUNT;
				}
			},
		}

		let mut out = Mat3::from_rotation_y(y_rotation_amount).mul_vec3(usual_eye_offset);
		out[1] += height_to_add;

		(
			// centre offset,
			Vec3::new(0.0, height_to_add, 0.0),
			// eye offset
			out,
			hard_set
		)
	}
}

impl CameraMode {
	pub fn zoom_in(&mut self) {
		self.0 = CameraState::zoom_in();
	}

	pub fn zoom_out_next(&mut self) {
		self.0 = CameraState::zoom_out_hold();
	}

	pub fn zoom_out_restart(&mut self) {
		self.0 = CameraState::zoom_out();
	}

	pub fn try_move(&mut self, time: &Time, exits: &mut ResMut<Events<DoLevelGen>>) {
		if let Some(new_state) = self.0.next(time) {
			match &new_state {
				CameraState::Zoomin(_) => {
					exits.send(DoLevelGen);
				},
				_ => {},
			}

			self.0 = new_state;
		}
	}

	// Returns new lookat, eye.
	pub fn camera_mods(&self, usual_eye_offset: Vec3) -> (Vec3, Vec3, bool) {
		self.0.camera_mods(usual_eye_offset)
	}

	pub fn allow_pan(&self) -> bool {
		self.0.allow_pan()
	}
}

pub struct CameraMode(CameraState);

#[derive(Debug, Default)]
pub struct CameraDest(pub Option<GridPosition>);

fn tick_camera_mode(
	time: Res<Time>,
	mut mode: ResMut<CameraMode>,
	mut exits: ResMut<Events<DoLevelGen>>,
) {
	mode.try_move(&time, &mut exits);
}

fn slide_camera_to_dest(
	dest: Res<CameraDest>,
	mode: Res<CameraMode>,
	input: Res<Input<KeyCode>>,
	mut maps: Query<&Map>,
	mut cameras: Query<(&CameraFaced, &mut Transform)>,
) {
	let mut offset = Vec3::new(-2.0, 2.0, -2.0);

	if mode.allow_pan() {
		if input.pressed(KeyCode::Z) {
			offset = Mat3::from_rotation_y(-std::f32::consts::FRAC_PI_6).mul_vec3(offset);
		} else if input.pressed(KeyCode::C) {
			offset = Mat3::from_rotation_y(std::f32::consts::FRAC_PI_6).mul_vec3(offset);
		}

		if input.pressed(KeyCode::X) {
			offset[1] += 3.0;
		}
	}

	let (centre_offset, eye_offset, hard_set) = mode.camera_mods(offset);

	if let Some(dest) = dest.0 {
		for map in &mut maps.iter() {
			for (_tag, mut tx) in &mut cameras.iter() {
				let z_target = map.heights[dest.unroll(map.width) as usize];
				let target = Vec3::new(-dest.y as f32, (z_target as f32) * WORLD_HEIGHT_SCALE, dest.x as f32);

				let start = tx.value();

				let target_cam = Mat4::face_toward(
					target + eye_offset,
					target + centre_offset,
					Vec3::new(0.0, 1.0, 0.0),
				);

				let scale = if hard_set {
					1.0
				} else {
					0.01
				};

				*tx = Transform::new(*start + scale * (target_cam - *start)).with_scale(1.0/75.0);
			}
		}
	}
}

fn active_char_is_camera_dest(
	mut dest: ResMut<CameraDest>,
	mut chars: Query<(&ActiveCharacter, &DisplayGridPosition)>,
) {
	for (_tag, pos) in &mut chars.iter() {
		dest.0 = Some(pos.0);
	}
}
