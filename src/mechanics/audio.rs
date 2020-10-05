use std::time::Duration;

use crate::map::{Map, TileTexture};

use bevy::prelude::*;
use enum_primitive::*;
use rand::{
	distributions::{Distribution,Uniform,},
	thread_rng,
};

use super::GridPosition;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_event::<StepEvent>()
			.add_event::<SoundClass>()
			.add_resource(MusicTimer::new())
			.add_system(handle_footstep.system())
			.add_system(handle_sound.system())
			.add_system(music.system());
	}
}

pub struct MusicTimer(Timer, bool);

impl MusicTimer {
	pub fn new() -> Self {
		// Song is exactly 16s long.
		Self(Timer::new(Duration::from_secs(16), true), false)
	}
}

fn music(
	time: Res<Time>,
	mut player: ResMut<MusicTimer>,
	asset_server: Res<AssetServer>,
	audio_out: Res<AudioOutput>,
) {
	player.0.tick(time.delta_seconds);

	if player.0.finished || !player.1 {
		let handle = asset_server.load("assets/music/NoName.mp3")
			.unwrap();

		if !player.1 {
			player.0.reset();
		}

		audio_out.play(handle);
		player.1 = true;
	}
}

fn handle_footstep(
	evts: Res<Events<StepEvent>>,
	asset_server: Res<AssetServer>,
	audio_out: Res<AudioOutput>,
	mut maps: Query<&Map>,
) {
	for evt in evts.get_reader().iter(&evts) {
		let StepEvent(pos) = evt;
		for map in &mut maps.iter() {
			if let Some(cls) = TileTexture::from_u8(map.tiles[(pos.unroll(map.width) as usize)]) {
				let sound = cls.sound_class();
				if let Some(path) = sound.get_random() {
					let audio = asset_server
						.load(path)
						.unwrap();

					audio_out.play(audio);
				}
			}
		}
	}
}

fn handle_sound(
	evts: Res<Events<SoundClass>>,
	asset_server: Res<AssetServer>,
	audio_out: Res<AudioOutput>,
) {
	for evt in evts.get_reader().iter(&evts) {
		if let Some(path) = evt.get_random() {
			let audio = asset_server
				.load(path)
				.unwrap();

			audio_out.play(audio);
		}
	}
}

#[inline]
fn random_element<'a, T>(arr: &'a[T]) -> &'a T {
	let mut rng = thread_rng();
	&arr[Uniform::new(0, arr.len()).sample(&mut rng)]
}

#[derive(Copy, Clone, Debug)]
pub enum SoundClass {
	Sand,
	Stone,
	Water,
	Blocked,
	Button,
	Na,
}

impl SoundClass {
	pub fn sound_paths(self) -> &'static [&'static str] {
		match self {
			SoundClass::Sand => &[
				"assets/sfx/sand0.mp3",
				"assets/sfx/sand1.mp3",
				"assets/sfx/sand2.mp3",
				"assets/sfx/sand3.mp3",
				"assets/sfx/sand4.mp3",
			][..],
			SoundClass::Blocked => &[
				"assets/sfx/block0.mp3",
				"assets/sfx/block1.mp3",
				"assets/sfx/block2.mp3",
			][..],
			SoundClass::Stone => &[
				"assets/sfx/stone0.mp3",
				"assets/sfx/stone1.mp3",
				"assets/sfx/stone2.mp3",
				"assets/sfx/stone3.mp3",
			][..],
			SoundClass::Button => &[
				"assets/sfx/button.mp3",
			][..],
			_ => &[][..],
		}
	}

	pub fn get_random(self) -> Option<&'static str> {
		let paths = self.sound_paths();

		if paths.is_empty() {
			None
		} else {
			Some(&random_element(paths))
		}
	}
}

pub struct StepEvent(pub GridPosition);
