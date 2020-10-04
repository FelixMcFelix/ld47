pub mod meta;

use crate::mechanics::ActiveTurn;
use crate::mechanics::Alive;
use crate::mechanics::{
	constants::*,
	ender::Ender,
	spawner::Spawner,
	DisplayGridPosition,
	GhostLimit,
	GridPosition,
	OccupationMap,
	Ordinate,
	TurnLimit,
};
use bevy::render::mesh::{VertexAttribute, VertexAttributeValues};
use bevy::{
	asset::HandleId,
	prelude::*,
};
use enum_primitive::*;
use lazy_static::lazy_static;
use self::meta::Levels;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TileHeight {
	// Tile will appear at this height, possibly allowing pathing.
	Passable(usize),
	// Tile will appear at this height, but prevents pathing.
	Impassable(usize),
}

impl From<isize> for TileHeight {
	fn from(input: isize) -> Self {
		let actual_height = input.abs() as usize;
		(if input < 0 {
			TileHeight::Impassable
		} else {
			TileHeight::Passable
		})(actual_height)
	}
}

impl Default for TileHeight {
	fn default() -> Self {
		TileHeight::Passable(0)
	}
}

impl TileHeight {
	pub fn to_raw_height(self) -> usize {
		match self {
			TileHeight::Impassable(a) => a,
			TileHeight::Passable(a) => a,
		}
	}
}

lazy_static! {
	static ref TILE_MESH_HANDLES: HashMap<TileShape, HandleId> = {
		let mut m = HashMap::new();
		for i in 0..u8::MAX {
			if let Some(t) = TileShape::from_u8(i) {
				m.insert(t, HandleId::new());
			}
		}

		m
	};
}

enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EntShape {
	Billboard = 0,
	BoostSquare,
}
}

impl EntShape {
	pub fn mesh(self) -> Mesh {
		use EntShape::*;
		match self {
			Billboard => Mesh::from(shape::Quad { size: (32.0/38.0, 1.0).into(), flip: false }),
			BoostSquare => {
				let mut m = Mesh::from(shape::Plane { size: 1.0 });
				let boost_height = -0.49;

				for attr_block in m.attributes.iter_mut() {
					if attr_block.name == VertexAttribute::POSITION {
						use VertexAttributeValues::*;
						match &mut attr_block.values {
							Float3(ref mut v) => {
								for val_set in v.iter_mut() {
									val_set[1] += boost_height;
								}
							},
							Float4(ref mut v) => {
								for val_set in v.iter_mut() {
									val_set[1] += boost_height;
								}
							},
							_ => {}
						}
					}
				}

				m
			},
		}
	}

	pub fn existing_mesh(
		self,
		meshes: &mut ResMut<Assets<Mesh>>,
	) -> Handle<Mesh> {
		let handle = Handle::from_id(*ENT_MESH_HANDLES.get(&self)
				.expect("Nani"));
		let _ = meshes.get_or_insert_with(
			handle,
			|| Mesh::from(self.mesh()),
		);

		handle
	}
}

lazy_static! {
	static ref ENT_MESH_HANDLES: HashMap<EntShape, HandleId> = {
		let mut m = HashMap::new();
		for i in 0..u8::MAX {
			if let Some(t) = EntShape::from_u8(i) {
				m.insert(t, HandleId::new());
			}
		}

		m
	};
}

enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EntClass {
	Start = 0,
	End,
}
}

impl EntClass {
	pub fn create(
		self,
		pos: GridPosition,
		comms: &mut Commands,
		mut meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		asset_server: &Res<AssetServer>,
		mut textures: &mut ResMut<Assets<Texture>>,
	) {
		match self {
			EntClass::Start => {
				let texture_handle = asset_server
					.load_sync(&mut textures, "assets/placeholder/start.png")
					.unwrap();

				let material = materials.add(StandardMaterial {
					albedo_texture: Some(texture_handle),
					shaded: false,
					..Default::default()
				});

				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						Spawner::default(),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						..Default::default()
					});
			},
			EntClass::End => {
				let texture_handle = asset_server
					.load_sync(&mut textures, "assets/placeholder/end.png")
					.unwrap();

				let material = materials.add(StandardMaterial {
					albedo_texture: Some(texture_handle),
					shaded: false,
					..Default::default()
				});

				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						Ender::default(),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						..Default::default()
					});
			},
		}

		comms.with(Alive::default());
	}
}


enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TileShape {
	Plane = 0,
}
}

impl TileShape {
	pub fn mesh(self) -> Mesh {
		match self {
			TileShape::Plane => Mesh::from(shape::Plane { size: 1.0 }),
		}
	}

	pub fn existing_mesh(
		self,
		meshes: &mut ResMut<Assets<Mesh>>,
	) -> Handle<Mesh> {
		let handle = Handle::from_id(*TILE_MESH_HANDLES.get(&self)
				.expect("Nani"));
		let _ = meshes.get_or_insert_with(
			handle,
			|| Mesh::from(self.mesh()),
		);

		handle
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EntBlueprint {
	class: u8,
	pos: GridPosition,
}

#[derive(Clone, Properties, Debug, Default, Deserialize, Serialize)]
pub struct Map {
	/// Map width.
	pub width: Ordinate,
	/// Map Height.
	pub height: Ordinate,
	/// Tile texture/type data.
	pub tiles: Vec<u8>,
	/// Shape to apply to tile.
	pub tile_shapes: Vec<u8>,
	/// Height (and passability) of each tile.
	pub heights: Vec<isize>,

	#[property(ignore)]
	pub created: bool,

	pub ents: Option<Vec<EntBlueprint>>,

	pub turn_limit: TurnLimit,

	pub ghost_limit: Option<GhostLimit>,
}

impl Map {
	pub fn len(&self) -> usize {
		(self.width * self.height) as usize
	}

	pub fn empty_of_size(width: Ordinate, height: Ordinate) -> Self {
		if width < 0 || height < 0 {
			panic!("That is NOT a valid map shape (negative dim(s)).");
		}
		let els = (width * height) as usize;
		Self {
			width,
			height,
			tiles: vec![Default::default(); els],
			tile_shapes: vec![Default::default(); els],
			heights: vec![Default::default(); els],
			created: false,
			ents: Some(vec![
				EntBlueprint{
					class: EntClass::Start as u8,
					pos: GridPosition{ x:0, y:0 }
				},
			]),
			turn_limit: TurnLimit(7),
			ghost_limit: Some(GhostLimit(1)),
		}
	}

	/// Assumes that positions were chosen by neighbourhood.
	pub fn move_allowed_by_terrain(&self, former_pos: &GridPosition, next_pos: &GridPosition) -> bool {
		let dest = next_pos.unroll(self.width) as usize;
		next_pos.x < self.width
			&& next_pos.x >= 0
			&& next_pos.y < self.height
			&& next_pos.y >= 0
			&& match self.heights.get(dest).map(|v| (*v).into()) {
			Some(TileHeight::Passable(h)) => {
				let src = former_pos.unroll(self.width) as usize;
				match self.heights.get(src).map(|v| (*v).into()) {
					Some(TileHeight::Passable(s)) => {
						let max = h.max(s);
						let min = h.min(s);
						max - min < HEIGHT_JUMP_LIMIT
					},
					_ => false,
				}
			},
			_ => false,
		}
	}

	fn create_geometry(
		&self,
		world: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
	) {
		for i in 0..self.len() {
			if let Some(tile_type) = TileShape::from_u8(self.tile_shapes[i]) {

				let handle = tile_type.existing_mesh(meshes);

				let pos = GridPosition::roll(i as Ordinate, self.width);

				let height = TileHeight::from(self.heights[i]).to_raw_height();

				world.spawn(PbrComponents {
					mesh: handle,
					material: materials.add(Color::rgb(0.5 * (pos.x as f32 / 5.0), 0.4 * (pos.y as f32 / 5.0), 0.3).into()),
					transform: Transform::from_translation(
						Vec3::new(-pos.x as f32, height as f32, pos.y as f32)
					),
					..Default::default()
				}).with(WorldGeometry)
				.with(Alive::default());
			}
		}
	}

	fn create_limits(
		&self,
		comms: &mut Commands,
	) {
		comms.insert_resource(self.turn_limit);

		if let Some(ghosts) = self.ghost_limit {
			comms.insert_resource(ghosts);
		}
	}

	fn create_entities(
		&self,
		world: &mut Commands,
		meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		asset_server: &Res<AssetServer>,
		textures: &mut ResMut<Assets<Texture>>,
	) {
		if let Some(ents) = &self.ents {
			for blueprint in ents {
				if let Some(ent) = EntClass::from_u8(blueprint.class) {
					ent.create(blueprint.pos, world, meshes, materials, asset_server, textures)
				}
			}
		}
	}
}

pub struct WorldGeometry;

pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app
			.add_system(map_creator.system());
	}
}

fn map_creator(
	mut commands: Commands,
	level_info: ResMut<Levels>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut textures: ResMut<Assets<Texture>>,
	mut occupation: ResMut<OccupationMap>,
	mut turn: ResMut<ActiveTurn>,
	mut query: Query<&mut Map>,
) {
	let mut was_empty = true;
	for mut map in &mut query.iter() {
		if !map.created {
			println!("I am creating this map");
			map.create_geometry(&mut commands, &mut meshes, &mut materials);
			map.create_limits(&mut commands);
			map.create_entities(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures);
			map.created = true;

			occupation.0 = vec![false; map.len()];
		}
		was_empty = false;
	}

	if was_empty {
		turn.reinit();
		commands.spawn((level_info.load_current(),Alive::default()));
	}
}
