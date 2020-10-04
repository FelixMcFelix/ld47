use bevy::prelude::*;

use super::DisplayGridPosition;
use super::character::Character;

#[derive(Debug, Default,)]
pub struct Spawner {
	pub used: bool,
}

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_system(spawner_makes_player.system());
	}
}

fn spawner_makes_player(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut textures: ResMut<Assets<Texture>>,
	mut query: Query<(&mut Spawner, &DisplayGridPosition)>,
) {
	for (mut spawner, pos) in &mut query.iter() {
		if !spawner.used {
			spawner.used = true;
			Character::new(pos.0)
				.spawn(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures)
		}
	}
}
