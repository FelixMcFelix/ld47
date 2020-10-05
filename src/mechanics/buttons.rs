use bevy::prelude::*;
use std::collections::HashMap;

use crate::map::Map;

use super::CollideGridPosition;
use super::DisplayGridPosition;
use super::GridPosition;
use super::OccupationMap;
use super::audio::SoundClass;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_resource(SignalCounter::default())
			.add_system(fire_signal_on_collide.system())
			.add_system(block_until_signal.system())
			.add_system(register_signal_sources.system());
	}
}

#[derive(Default)]
pub struct SignalCount {
	seen: usize,
	needed: usize,
}

#[derive(Default)]
pub struct SignalCounter(pub HashMap<usize, SignalCount>);

impl SignalCounter {
	fn ensure_exists(&mut self, signal: usize) -> &mut SignalCount {
		self.0.entry(signal)
			.or_default()
	}

	pub fn register_signal_source(&mut self, signal: usize) {
		let mut count = self.ensure_exists(signal);
		count.needed += 1;
		println!("Requiring signal {} -> {} needed", signal, count.needed);
	}

	pub fn increment_signal(&mut self, signal: usize) {
		let mut count = self.ensure_exists(signal);
		count.seen += 1;
		println!("incrementing signal {} -> {} seen", signal, count.seen);
	}

	pub fn decrement_signal(&mut self, signal: usize) {
		let mut count = self.ensure_exists(signal);
		count.seen -= 1;
		println!("decrementing signal {} -> {} seen", signal, count.seen);
	}

	pub fn signal_met(&self, signal: usize) -> bool {
		let out = self.0.get(&signal)
			.map(|counter| counter.seen >= counter.needed)
			.unwrap_or_default();

		// println!("query signal {} -> {}", signal, out);

		out
	}

	pub fn reinit(&mut self) {
		self.0 = Default::default();
	}
}

#[derive(Debug, Default)]
pub struct RegisterSignal(pub usize);

#[derive(Debug, Default)]
pub struct FireSignalOnCollide {
	signal: usize,
	pos: GridPosition,
	last_collide: bool,
}

impl FireSignalOnCollide {
	pub fn new(signal: usize, pos: GridPosition) -> Self {
		Self {
			signal,
			pos,
			..Default::default()
		}
	}
}

#[derive(Debug, Default)]
pub struct OccupySpaceUntilSignal{
	signal: usize,
	pos: GridPosition,
	last_collide: bool,
}

impl OccupySpaceUntilSignal {
	pub fn new(signal: usize, pos: GridPosition) -> Self {
		Self {
			signal,
			pos,
			last_collide: true,
		}
	}
}

fn fire_signal_on_collide(
	mut signals: ResMut<SignalCounter>,
	collisions: Res<OccupationMap>,
	mut maps: Query<&Map>,
	mut query: Query<&mut FireSignalOnCollide>,
) {
	for map in &mut maps.iter() {
		for mut signaller in &mut query.iter() {
			let pos = signaller.pos.unroll(map.width) as usize;
			let occupied = collisions.0[pos];

			if signaller.last_collide != occupied {
				if occupied {
					signals.increment_signal(signaller.signal);
				} else {
					signals.decrement_signal(signaller.signal);
				}
				signaller.last_collide = occupied;
			}
		}
	}
}

fn block_until_signal(
	mut commands: Commands,
	signals: ResMut<SignalCounter>,
	mut collisions: ResMut<OccupationMap>,
	mut evts: ResMut<Events<SoundClass>>,
	mut maps: Query<&Map>,
	mut query: Query<(Entity, &mut OccupySpaceUntilSignal, &mut Transform)>,
) {
	for map in &mut maps.iter() {
		for (ent, mut collide_data, mut tx) in &mut query.iter() {
			let pos = collide_data.pos.unroll(map.width) as usize;
			let cond_met = signals.signal_met(collide_data.signal);
			if cond_met != collide_data.last_collide {
				if cond_met {
					commands.remove_one::<CollideGridPosition>(ent);
					commands.remove_one::<DisplayGridPosition>(ent);

					tx.set_translation(Vec3::new(-22.0, -22.0, -22.0));
					evts.send(SoundClass::Button);
				} else {
					commands.insert_one(ent, CollideGridPosition(collide_data.pos));
					commands.insert_one(ent, DisplayGridPosition(collide_data.pos));
				}

				collide_data.last_collide = cond_met;
			}

			collisions.0[pos] = cond_met;
		}
	}
}

fn register_signal_sources(
	mut commands: Commands,
	mut signals: ResMut<SignalCounter>,
	mut query: Query<(Entity, &RegisterSignal)>,
) {
	for (ent, signal) in &mut query.iter() {
		signals.register_signal_source(signal.0);
		commands.remove_one::<RegisterSignal>(ent);
	}
}
