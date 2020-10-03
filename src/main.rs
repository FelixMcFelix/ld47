mod mechanics;
mod ui;

use bevy::prelude::*;
use ui::UiPlugin;

fn hello_world(time: Res<Time>, mut timer: ResMut<TestTtime>) {
	timer.0.tick(time.delta_seconds);

	if timer.0.finished {
		println!("hello workd");
	}
}

struct TestTtime(Timer);

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(TestTtime(Timer::from_seconds(2.0, true)))
			.add_system(hello_world.system());
	}
}

fn main() {
	App::build()
		.add_default_plugins()
		// .add_resource(Msaa { samples: 4 })
		.add_plugin(HelloPlugin)
		.add_plugin(UiPlugin)
		.add_system(hello_world.system())
		.add_startup_system(setup.system())
		.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands
		.spawn(PbrComponents {
			mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
			material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
			..Default::default()
		})
		.spawn(LightComponents {
			transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
			..Default::default()
		})
		.spawn(Camera3dComponents {
			transform: Transform::new(Mat4::face_toward(
				Vec3::new(-3.0, 3.0, 5.0),
				Vec3::new(0.0, 0.0, 0.0),
				Vec3::new(0.0, 1.0, 0.0),
			)),
			..Default::default()
		})
		;
}