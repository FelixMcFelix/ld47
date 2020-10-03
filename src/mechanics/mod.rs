pub mod constants;

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
	North,
	East,
	South,
	West,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCommand {
	Move(Direction),
	Wait,
}

pub type Ordinate = isize;

// height is a derived property
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GridPosition {
	pub x: Ordinate,
	pub y: Ordinate,
}

impl GridPosition {
	pub fn roll(ordinate: Ordinate, map_w: Ordinate) -> Self {
		Self {
			x: ordinate / map_w,
			y: ordinate % map_w,
		}
	}

	pub fn unroll(self, map_w: Ordinate) -> Ordinate {
		self.x + (self.y * map_w)
	}

	pub fn destination(self, action: CharacterCommand) -> Self {
		match action {
			CharacterCommand::Move(d) => self.neighbour(d),
			_ => self,
		}
	}

	pub fn neighbour(mut self, direction: Direction) -> Self {
		match direction {
			Direction::North => {self.x += 1;},
			Direction::South => {self.x -= 1;},
			Direction::East => {self.y += 1;},
			Direction::West => {self.y -= 1;},
		}

		self
	}
}

pub trait Movable {

}