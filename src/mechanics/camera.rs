use super::{
	character::ActiveCharacter,
	CameraFaced, DisplayGridPosition, GridPosition
};
use crate::map::Map;

use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(CameraDest::default())
			.add_system(slide_camera_to_dest.system())
			.add_system(active_char_is_camera_dest.system());
	}
}

#[derive(Debug, Default)]
pub struct CameraDest(pub Option<GridPosition>);

fn slide_camera_to_dest(
	dest: Res<CameraDest>,
	input: Res<Input<KeyCode>>,
	mut maps: Query<&Map>,
	mut cameras: Query<(&CameraFaced, &mut Transform)>,
) {
	let mut offset = Vec3::new(-2.0, 2.0, -2.0);

	if input.pressed(KeyCode::C) {
		offset[1] += 3.0;
	}

	if let Some(dest) = dest.0 {
		for map in &mut maps.iter() {
			for (_tag, mut tx) in &mut cameras.iter() {
				let z_target = map.heights[dest.unroll(map.width) as usize];
				let target = Vec3::new(-dest.y as f32, z_target as f32, dest.x as f32);

				let start = tx.value();

				// tx.translate(0.05 * (target - start));


				let target_cam = Mat4::face_toward(
					target + offset,
					target,
					Vec3::new(0.0, 1.0, 0.0),
				);

				*tx = Transform::new(*start + 0.03 * (target_cam - *start)).with_scale(1.0/75.0);
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
