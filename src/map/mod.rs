pub mod materials;
pub mod meta;

use crate::mechanics::audio::SoundClass;
use crate::mechanics::buttons::{
	FireSignalOnCollide,
	OccupySpaceUntilSignal,
	RegisterSignal,
};
use crate::mechanics::{
	buttons::SignalCounter,
	constants::*,
	ender::Ender,
	spawner::Spawner,
	ActiveTurn,
	Alive,
	Direction,
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
use materials::AnimatedMaterial;
use self::meta::Levels;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const WORLD_HEIGHT_SCALE: f32 = 0.5;

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

fn mesh_flip_uv(in_mesh: &mut Mesh) {
	for attr_block in in_mesh.attributes.iter_mut() {
		if attr_block.name == VertexAttribute::UV {
			use VertexAttributeValues::*;
			match &mut attr_block.values {
				Float2(fs) => {
					fs[..].reverse();
				},
				_ => {},
			}
		}
	}
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

				mesh_flip_uv(&mut m);

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ActionChannel(pub usize);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EntData {
	Start,
	End,
	Button(ActionChannel),
	Door(ActionChannel),
}

impl EntData {
	pub fn create(
		&self,
		pos: GridPosition,
		rot: Direction,
		comms: &mut Commands,
		mut meshes: &mut ResMut<Assets<Mesh>>,
		materials: &mut ResMut<Assets<StandardMaterial>>,
		asset_server: &Res<AssetServer>,
		textures: &mut ResMut<Assets<Texture>>,
	) {
		let (material, anim) = match self.anim().handles(asset_server, textures, materials) {
			TexVariety::Unanim(mat) => (mat, None),
			TexVariety::Anim(mat) => (mat.first().unwrap(), Some(mat)),
		};

		// let transform = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI));
		let transform = Transform::from_rotation(
			Quat::from_rotation_y(rot.angle())
			);

		match self {
			EntData::Start => {
				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						Spawner::default(),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						transform,
						draw: Draw {
							is_transparent: true,
							..Default::default()
						},
						..Default::default()
					});
			},
			EntData::End => {
				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						Ender::default(),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						transform,
						draw: Draw {
							is_transparent: true,
							..Default::default()
						},
						..Default::default()
					});
			},
			EntData::Button(data) => {
				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						FireSignalOnCollide::new(data.0, pos),
						RegisterSignal(data.0),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						transform,
						draw: Draw {
							is_transparent: true,
							..Default::default()
						},
						..Default::default()
					});
			},
			EntData::Door(data) => {
				let mesh = EntShape::BoostSquare.existing_mesh(&mut meshes);

				comms.spawn((
						OccupySpaceUntilSignal::new(data.0, pos),
						DisplayGridPosition(pos),
					))
					.with_bundle(PbrComponents {
						mesh,
						material,
						transform,
						draw: Draw {
							is_transparent: true,
							..Default::default()
						},
						..Default::default()
					});
			},
		}

		comms.with(Alive::default());

		if let Some(anim) = anim {
			comms.with(anim);
		}
	}

	pub fn anim(&self) -> EntAnim {
		match self {
			EntData::Start => EntAnim::Start,
			EntData::End => EntAnim::End,
			EntData::Button(_) => EntAnim::Button,
			EntData::Door(_) => EntAnim::Door,
		}
	}
}

impl Default for EntData {
	fn default() -> Self {
		EntData::Start
	}
}

enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TileShape {
	Plane = 0,
	Slope
}
}

impl TileShape {
	pub fn mesh(self) -> Mesh {
		let mut m = match self {
			TileShape::Plane => Mesh::from(shape::Plane { size: 1.0 }),
			TileShape::Slope => {
				let mut m = Mesh::from(shape::Plane { size: 1.0 });

				for attr_block in m.attributes.iter_mut() {
					if attr_block.name == VertexAttribute::POSITION {
						use VertexAttributeValues::*;
						match &mut attr_block.values {
							Float3(ref mut v) => {
								for val_set in v.iter_mut().take(2) {
									val_set[1] += 1.0;
								}
							},
							Float4(ref mut v) => {
								for val_set in v.iter_mut().take(2) {
									val_set[1] += 1.0;
								}
							},
							_ => {}
						}
					}
				}

				m
			},
		};

		mesh_flip_uv(&mut m);

		m
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

pub enum TexVariety {
	Unanim(Handle<StandardMaterial>),
	Anim(AnimatedMaterial),
}

impl TexVariety {
	fn from_asset_list(
		fps: f32,
		strs: &[&str],
		asset_server: &AssetServer,
		mut textures: &mut Assets<Texture>,
		materials: &mut Assets<StandardMaterial>,
	) -> Self {
		match strs.len() {
			1 => {
				let texture_handle = asset_server
					.load_sync(&mut textures, strs[0])
					.unwrap();

				let material = materials.add(StandardMaterial {
					albedo_texture: Some(texture_handle),
					shaded: false,
					..Default::default()
				});

				TexVariety::Unanim(material)
			},
			a if a > 1 => {
				let mut handles = vec![];

				for i in 0..a {
					let texture_handle = asset_server
						.load_sync(&mut textures, strs[i])
						.unwrap();

					let material = materials.add(StandardMaterial {
						albedo_texture: Some(texture_handle),
						shaded: false,
						..Default::default()
					});

					handles.push(material);
				}

				TexVariety::Anim(AnimatedMaterial::new(fps, handles))
			},
			_ => unimplemented!(),
		}
	}
}

enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TileTexture {
	Good = 0,
	Bad,
	Bad2,
	SlopeDrop,
	Sand,

	Up,
	Left,
	Right,
	Down,
	Block,

	LeftBlock,
	RightBlock,
	GemWall,
	StdWall,
}
}

impl TileTexture {
	pub fn handles(
		self,
		asset_server: &AssetServer,
		textures: &mut Assets<Texture>,
		materials: &mut Assets<StandardMaterial>,
	) -> TexVariety {
		let (fps, res) = match self {
			TileTexture::Good => (0.0, &[
				"assets/tiles/ground_01.png",
			][..]),
			TileTexture::Bad => (0.0, &[
				"assets/tiles/ground_00.png",
			][..]),
			TileTexture::Bad2 => (3.0, &[
				"assets/placeholder/bad2.png",
				"assets/placeholder/bad22.png",
			][..]),
			TileTexture::SlopeDrop => (0.0, &[
				"assets/tiles/ground_02.png",
			][..]),
			TileTexture::Sand => (0.0, &[
				"assets/tiles/ground_03.png",
			][..]),
			TileTexture::Up => (0.0, &[
				"assets/tiles/ground_04.png",
			][..]),
			TileTexture::Left => (0.0, &[
				"assets/tiles/ground_05.png",
			][..]),
			TileTexture::Right => (0.0, &[
				"assets/tiles/ground_06.png",
			][..]),
			TileTexture::Down => (0.0, &[
				"assets/tiles/ground_07.png",
			][..]),
			TileTexture::Block => (0.0, &[
				"assets/tiles/ground_08.png",
			][..]),
			TileTexture::LeftBlock => (0.0, &[
				"assets/tiles/ground_09.png",
			][..]),
			TileTexture::RightBlock => (0.0, &[
				"assets/tiles/ground_10.png",
			][..]),
			TileTexture::GemWall => (0.0, &[
				"assets/tiles/ground_11.png",
			][..]),
			TileTexture::StdWall => (0.0, &[
				"assets/tiles/ground_12.png",
			][..]),
		};

		TexVariety::from_asset_list(fps, res, asset_server, textures, materials)
	}

	pub fn sound_class(self) -> SoundClass {
		use TileTexture::*;
		match self {
			Good | Sand => SoundClass::Sand,
			Up | Left | Right | Down | Block | LeftBlock | RightBlock => SoundClass::Stone,
			_ => SoundClass::Na,
		}
	}
}

enum_from_primitive!{
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EntAnim {
	Start,
	End,
	Button,
	ButtonGone,
	Door,
	Char,
}
}

impl EntAnim {
	pub fn handles(
		self,
		asset_server: &AssetServer,
		textures: &mut Assets<Texture>,
		materials: &mut Assets<StandardMaterial>,
	) -> TexVariety {
		let (fps, res) = match self {
			EntAnim::Start => (0.0, &[
				"assets/tiles/ground_14.png",
			][..]),
			EntAnim::End => (0.0, &[
				"assets/tiles/ground_13.png",
			][..]),
			EntAnim::Button => (0.0, &[
				"assets/tiles/ground_15.png",
			][..]),
			EntAnim::Door => (0.0, &[
				"assets/tiles/ground_11.png",
			][..]),
			EntAnim::ButtonGone => (0.0, &[
				"assets/tiles/ground_14.png",
			][..]),
			EntAnim::Char => (3.0, &[
				"assets/char/char1.png",
				"assets/char/char2.png",
			][..]),
		};

		TexVariety::from_asset_list(fps, res, asset_server, textures, materials)
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EntBlueprint {
	pos: GridPosition,
	data: EntData,
	rot: Option<Direction>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Wall {
	pos: GridPosition,
	h: f32,
	texture: u8,
	rot: Option<Direction>,
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
	/// Shape to apply to tile.
	pub tile_rots: Vec<u8>,
	/// Height (and passability) of each tile.
	pub heights: Vec<isize>,

	#[property(ignore)]
	pub created: bool,

	pub ents: Option<Vec<EntBlueprint>>,

	pub walls: Option<Vec<Wall>>,

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
			tile_rots: vec![Default::default(); els],
			created: false,
			ents: Some(vec![
				EntBlueprint{
					pos: GridPosition{ x:0, y:0 },
					data: EntData::Start,
					rot: None,
				},
			]),
			walls: None,
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
		asset_server: &AssetServer,
		textures: &mut Assets<Texture>,
	) {
		for i in 0..self.len() {
			let maybe_tex = TileTexture::from_u8(self.tiles[i]);
			let maybe_rot = Direction::from_u8(self.tile_rots[i]).map(|m| m.angle());
			if let Some(((tile_type, tex_type), angle)) = TileShape::from_u8(self.tile_shapes[i]).zip(maybe_tex).zip(maybe_rot) {

				let handle = tile_type.existing_mesh(meshes);

				let pos = GridPosition::roll(i as Ordinate, self.width);

				let height = TileHeight::from(self.heights[i]).to_raw_height();

				let (material, anim) = match tex_type.handles(asset_server, textures, materials) {
					TexVariety::Unanim(mat) => (mat, None),
					TexVariety::Anim(mat) => (mat.first().unwrap(), Some(mat)),
				};

				world.spawn(PbrComponents {
					mesh: handle,
					material,
					transform: Transform::from_translation(
						Vec3::new(-pos.x as f32, (height as f32) * WORLD_HEIGHT_SCALE, pos.y as f32)
					).with_non_uniform_scale(Vec3::new(1.0, WORLD_HEIGHT_SCALE, -1.0))
					.with_rotation(Quat::from_rotation_y(angle)),
					..Default::default()
				}).with(WorldGeometry)
				.with(Alive::default());

				if let Some(anim) = anim {
					world.with(anim);
				}
			}
		}

		if let Some(ref walls) = self.walls {
			for wall in walls.iter() {

				println!("{:?}", wall);

				// let i = wall.pos.unroll(self.width) as usize;
				let maybe_rot = wall.rot;
				if let Some((tex_type, dir)) = TileTexture::from_u8(wall.texture).zip(maybe_rot) {
					let angle = dir.angle();
					let mesh = meshes.add(Mesh::from(shape::Quad { size: (1.0, 1.0).into(), flip: true }));

					let pos = wall.pos;

					let height = wall.h;

					let (material, anim) = match tex_type.handles(asset_server, textures, materials) {
						TexVariety::Unanim(mat) => (mat, None),
						TexVariety::Anim(mat) => (mat.first().unwrap(), Some(mat)),
					};

					let (x_adj, y_adj) = match dir {
						Direction::North => (0.0, -0.5),
						Direction::South => (0.5, 0.0),
						_ => (0.0, 0.0),
					};

					world.spawn(PbrComponents {
						mesh,
						material,
						transform: Transform::from_translation(
							Vec3::new(-pos.y as f32 + x_adj, (height as f32) * WORLD_HEIGHT_SCALE, pos.x as f32 + y_adj)
						).with_non_uniform_scale(Vec3::new(1.0, WORLD_HEIGHT_SCALE, 1.0))
						.with_rotation(Quat::from_rotation_y(angle)),
						..Default::default()
					}).with(WorldGeometry)
					.with(Alive::default());

					if let Some(anim) = anim {
						world.with(anim);
					}
				}
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
				let rot = blueprint.rot.unwrap_or_default();
				blueprint.data.create(blueprint.pos, rot, world, meshes, materials, asset_server, textures)
			}
		}
	}
}

pub struct WorldGeometry;

pub struct MapPlugin;

impl Plugin for MapPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app
			.add_plugin(materials::MaterialPlugin)
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
	mut signals: ResMut<SignalCounter>,
	mut query: Query<&mut Map>,
) {
	let mut was_empty = true;
	for mut map in &mut query.iter() {
		if !map.created {
			println!("I am creating this map");
			map.create_geometry(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures);
			map.create_limits(&mut commands);
			map.create_entities(&mut commands, &mut meshes, &mut materials, &asset_server, &mut textures);
			map.created = true;

			occupation.0 = vec![false; map.len()];
		}
		was_empty = false;
	}

	if was_empty {
		turn.reinit();
		signals.reinit();

		if level_info.start_at >= level_info.data.len() {

		} else {
			commands.spawn((level_info.load_current(),Alive::default()));
		}
	}
}
