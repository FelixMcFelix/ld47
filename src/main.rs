mod map;
mod mechanics;
mod ui;

use bevy::{
	prelude::*,
	render::pass::ClearColor,
};
use map::{EntShape, MapPlugin};
use mechanics::{
	character::{ActiveCharacter, Character},
	MechanicsPlugin,
	TurnLimit,
};
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
		.add_resource(WindowDescriptor {
			title: "LD47: Stuck in a Loop".to_string(),
			..Default::default()
		})
		.add_resource(ClearColor(Color::hex("374b6d").expect("Ha")))
		.add_default_plugins()
		// .add_plugin(BillboardPlugin)
		// .add_resource(Msaa { samples: 4 })
		.add_plugin(HelloPlugin)
		.add_plugin(UiPlugin)
		.add_plugin(MapPlugin)
		.add_plugin(MechanicsPlugin)
		.add_system(hello_world.system())
		.add_startup_system(setup.system())
		.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut textures: ResMut<Assets<Texture>>,
	// mut billboards: ResMut<Assets<BillboardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	commands
		.spawn(LightComponents {
			transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
			..Default::default()
		})
		.spawn(Camera2dComponents {
			transform: Transform::new(Mat4::face_toward(
				Vec3::new(-2.0, 2.0, -2.0),
				Vec3::new(0.0, 0.0, 0.0),
				Vec3::new(0.0, 1.0, 0.0),
			)).with_scale(1.0/75.0),
			..Default::default()
		})
		.with(mechanics::CameraFaced)
		.spawn((map::Map::empty_of_size(10, 5),))
		.spawn((TurnLimit(4),));

	Character::new_split(1, 1).spawn(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures)
}
