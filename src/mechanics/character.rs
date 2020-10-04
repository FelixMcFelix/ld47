use bevy::prelude::*;
use super::{
	ActiveTurn,
	CameraFacer,
	CollideGridPosition,
	Direction,
	DisplayGridPosition,
	GridPosition,
	OccupationMap,
	Ordinate,
	TurnLimit,
};
use crate::map::{EntShape, Map};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCommand {
	Move(Direction),
	Wait,
}

#[derive(Clone, Debug)]
pub struct Character {
	pub start: GridPosition,
	pub current: GridPosition,
	pub command_list: Vec<CharacterCommand>,
	pub cmd_list_pos: usize,
	pub my_turn: usize,
}

impl Character {
	pub fn new(pos: GridPosition) -> Self {
		Self {
			start: pos,
			current: pos,
			command_list: vec![],
			cmd_list_pos: 0,
			my_turn: 0,
		}
	}

	pub fn new_split(x: Ordinate, y: Ordinate) -> Self {
		Self::new(GridPosition{ x, y })
	}

	pub fn do_action(&mut self, action: CharacterCommand, map: &Map, colliders: &mut OccupationMap) {
		let supposed_dest = self.current.destination(action);

		println!("{:?} -> {:?}", action, supposed_dest);
		let normalised = supposed_dest.clamp(map.width, map.height).unroll(map.width) as usize;

		if map.move_allowed_by_terrain(&self.current, &supposed_dest) && !colliders.0[normalised] {
			let form_norm = self.current.clamp(map.width, map.height).unroll(map.width) as usize;
			colliders.move_collider(form_norm, normalised);
			self.current = supposed_dest;
			println!("moved to {:?}", supposed_dest);
		} else {
			println!("move blocked")
		}
	}

	pub fn do_queued_action(&mut self, map: &Map, colliders: &mut OccupationMap) {
		println!("Queue!");
		let action = self.command_list[self.cmd_list_pos];
		self.do_action(action, map, colliders);
		self.cmd_list_pos += 1;
	}

	pub fn reset(&mut self) {
		self.current = self.start;
		self.cmd_list_pos = 0;
	}

	pub fn new_me(&self) -> Self {
		let mut out = self.clone();
		out.reset();
		out.command_list.clear();
		out.my_turn += 1;
		out
	}

	pub fn spawn(
		self,
		comms: &mut Commands,
		mut meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		asset_server: &Res<AssetServer>,
		mut textures: &mut ResMut<Assets<Texture>>,
	) {
		let texture_handle = asset_server
	        .load_sync(&mut textures, "assets/placeholder/ramza.png")
	        .unwrap();

	    let material = materials.add(StandardMaterial {
	    	albedo_texture: Some(texture_handle),
	    	shaded: false,
	    	..Default::default()
	    });

		let pos = self.current;

		let mesh = EntShape::Billboard.existing_mesh(&mut meshes);

		comms.spawn((
				self,
				ActiveCharacter,
				DisplayGridPosition(pos),
				CollideGridPosition(pos),
				CameraFacer,
			))
			.with_bundle(PbrComponents {
				mesh,
				material,
				..Default::default()
			});
	}
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_system(char_control.system())
			.add_system(char_act.system())
			.add_system(char_display.system())
			.add_system(char_reset.system())
			.add_system(char_set_collision.system());
	}
}

pub struct ActiveCharacter;

pub struct InactiveCharacter;

fn char_control(
	limit: Res<TurnLimit>,
	mut occupation: ResMut<OccupationMap>,
	mut turn: ResMut<ActiveTurn>,
	key_input: Res<Input<KeyCode>>,
	mut map_query: Query<&Map>,
	mut query: Query<(&mut Character, &ActiveCharacter)>,
) {
	for map in &mut map_query.iter() {
		for (mut character, _active) in &mut query.iter() {
			let mut chosen_dir = None;

			for key in key_input.get_just_pressed() {
				use CharacterCommand::*;
				match key {
					KeyCode::Up => {
						chosen_dir = Some(Move(Direction::North));
					},
					KeyCode::Right => {
						chosen_dir = Some(Move(Direction::East));
					},
					KeyCode::Left => {
						chosen_dir = Some(Move(Direction::West));
					},
					KeyCode::Down => {
						chosen_dir = Some(Move(Direction::South));
					},
					KeyCode::Space => {
						chosen_dir = Some(Wait);
					},
					_ => {},
				}
			}

			if let Some(action) = chosen_dir {
				if turn.allow_turn(limit.0, character.my_turn) {
					// ALWAYS push action regardless of whether or not it is doable.
					character.command_list.push(action);

					character.do_action(action, map, &mut occupation);

					turn.march_turn();
				}
			}
		}	
	}
}

fn char_act(
	limit: Res<TurnLimit>,
	mut turn: ResMut<ActiveTurn>,
	mut occupation: ResMut<OccupationMap>,
	mut map_query: Query<&Map>,
	mut query: Query<(&mut Character, &InactiveCharacter)>,
) {
	for map in &mut map_query.iter() {
		for (mut character, _inactive) in &mut query.iter() {
			if turn.allow_turn(limit.0, character.my_turn) {
				// ALWAYS push action regardless of whether or not it is doable.
				character.do_queued_action(map, &mut occupation);

				turn.march_turn();
			}
		}	
	}
}

fn char_display(
	mut query: Query<(&Character, &mut DisplayGridPosition)>,
) {
	for (character, mut pos) in &mut query.iter() {
		pos.0 = character.current;
	}	
}

fn char_set_collision(
	mut query: Query<(&Character, &mut CollideGridPosition)>,
) {
	for (character, mut pos) in &mut query.iter() {
		pos.0 = character.current;
	}	
}

fn char_reset(
	mut commands: Commands,
	limit: Res<TurnLimit>,
	mut turn: ResMut<ActiveTurn>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut textures: ResMut<Assets<Texture>>,
	mut actives_query: Query<(Entity, &mut Character, &ActiveCharacter)>,
	mut inactives_query: Query<(&mut Character, &InactiveCharacter)>,
) {
	if turn.should_reset(limit.0) {
		for (ent, mut character, _active_char) in &mut actives_query.iter() {
			commands.remove_one::<ActiveCharacter>(ent);
			commands.insert_one(ent, InactiveCharacter);

			let new = character.new_me();

			new.spawn(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures);

			character.reset();
		}

		for (mut character, mut _inactive) in &mut inactives_query.iter() {
			character.reset();
		}

		turn.reset_and_add_ent();
	}
}
