use std::time::Duration;

use bevy::prelude::*;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_system(material_setter.system())
			.add_system(animated_material_tick.system());
	}
}

pub struct SetMaterial(pub Handle<StandardMaterial>);

fn material_setter(
	mut commands: Commands,
	mut to_change: Query<(Entity, &mut Handle<StandardMaterial>, &SetMaterial)>,
) {
	for (ent, mut handle, new_mat) in &mut to_change.iter() {
		*handle = new_mat.0;
		commands.remove_one::<SetMaterial>(ent);
	}
}

pub struct AnimatedMaterial {
	pub base_timer: Timer,
	pub pos: usize,
	pub materials: Vec<Handle<StandardMaterial>>,
}

impl AnimatedMaterial {
	pub fn new(fps: f32, materials: Vec<Handle<StandardMaterial>>) -> Self {
		Self {
			base_timer: Timer::new(Duration::from_secs_f32(1.0/fps), true),
			pos: 0,
			materials,
		}
	}

	fn tick(&mut self, time: &Time) -> Option<Handle<StandardMaterial>> {
		self.base_timer.tick(time.delta_seconds);

		if self.base_timer.just_finished {
			self.pos += 1;
			self.pos %= self.materials.len();

			self.materials.get(self.pos).cloned()
		} else {
			None
		}
	}

	pub fn current(&self) -> Option<Handle<StandardMaterial>> {
		self.materials.get(self.pos).cloned()
	}

	pub fn first(&self) -> Option<Handle<StandardMaterial>> {
		self.materials.get(0).cloned()
	}
}

fn animated_material_tick(
	mut commands: Commands,
	time: Res<Time>,
	mut to_change: Query<(Entity, &mut AnimatedMaterial)>,
) {
	for (ent, mut material_list) in &mut to_change.iter() {
		if let Some(mat) = material_list.tick(&time) {
			commands.insert_one(ent, SetMaterial(mat));
		}
	}
}
