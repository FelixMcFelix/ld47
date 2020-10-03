use crate::mechanics::{
	constants::*,
	GridPosition,
	Ordinate,
};
use bevy::{
	asset::HandleId,
	prelude::*,
};
use enum_primitive::*;
use lazy_static::lazy_static;
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
}

#[derive(Clone, Properties, Debug, Default)]
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
	pub created: bool
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
		}
	}

	/// Assumes that positions were chosen by neighbourhood.
	pub fn move_allowed_by_terrain(&self, former_pos: &GridPosition, next_pos: &GridPosition) -> bool {
		let dest = next_pos.unroll(self.width) as usize;
		match self.heights.get(dest).map(|v| (*v).into()) {
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

				let handle = Handle::from_id(*TILE_MESH_HANDLES.get(&tile_type).expect("Nani"));
				let _ = meshes.get_or_insert_with(
					handle,
					|| Mesh::from(tile_type.mesh()),
				);

				let pos = GridPosition::roll(i as Ordinate, self.width);

				let height = TileHeight::from(self.heights[i]).to_raw_height() + (pos.y as usize / 2);

				// println!("TIle has {:?} height {:?}", pos, height);

				world.spawn(PbrComponents {
					mesh: handle,
					material: materials.add(Color::rgb(0.5 * (pos.x as f32 / 5.0), 0.4 * (pos.y as f32 / 5.0), 0.3).into()),
					transform: Transform::from_translation(
						Vec3::new(pos.x as f32, height as f32, pos.y as f32)
					),
					..Default::default()
				}).with(WorldGeometry);
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
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut query: Query<&mut Map>,
) {
	for mut map in &mut query.iter() {
		if !map.created {
			println!("I am creating this map");
			map.create_geometry(&mut commands, &mut meshes, &mut materials);
			map.created = true;
		}
	}
}
