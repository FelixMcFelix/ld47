use crate::mechanics::Ordinate;

use super::Map;

use ron::{
	de::from_reader,
	ser::{to_string_pretty, PrettyConfig},
};
use serde::{Deserialize, Serialize};
use std::{
	fs::File,
	io::Write,
};

const LEVEL_MANIFEST_LOCATION: &str = "assets/levels.ron";
const EXAMPLE_MANIFEST_LOCATION: &str = "assets/levels.ron-ex";

const EXAMPLE_LEVEL_LOCATION: &str = "assets/levels/test-level.ron";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Levels {
	pub data: Vec<LevelInfo>,
	pub start_at: usize,
}

impl Levels {
	pub fn get_self() -> Self {
		let f = File::open(LEVEL_MANIFEST_LOCATION)
			.expect("Level load failed.");

		from_reader(f)
			.expect("Apparently misread.")
	}

	pub fn generate_example() {
		let mut f = File::create(EXAMPLE_MANIFEST_LOCATION)
			.expect("Level load failed.");

		let pretty = PrettyConfig::new();

		let out = to_string_pretty(&Levels {
			data: vec![LevelInfo::default()],
			..Default::default()
		}, pretty).expect("Must ser");

		f.write_all(out.as_bytes())
			.expect("Write failed");
	}

	pub fn load_next(&mut self) -> Map {
		self.start_at += 1;
		self.load_current()
	}

	pub fn load_current(&self) -> Map {
		self.data[self.start_at].get_map()
	}
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LevelInfo {
	name: String,
	path: String,
}

impl LevelInfo {
	pub fn get_map(&self) -> Map {
		let f = File::open(&self.path).expect("Level load failed.");

		from_reader(f)
			.expect("Apparently misread.")
	}
}

impl Map {
	pub fn generate_example_of_size(w: Ordinate, h: Ordinate) {
		let mut f = File::create(EXAMPLE_LEVEL_LOCATION)
			.expect("Level load failed.");

		let pretty = PrettyConfig::new();

		let out = to_string_pretty(&Map::empty_of_size(w, h), pretty).expect("Must ser");

		f.write_all(out.as_bytes())
			.expect("Write failed");
	}
}
