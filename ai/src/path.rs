use crate::key_type::KeyType;

pub struct Path {
	pub key_type: KeyType,
	pub move_count: u8,
	pub before_x: i8,
	pub before_y: i8,
	pub before_x_diff: i8,
	pub before_y_diff: i8,
}

impl Path {
	pub fn new(key_type: KeyType, move_count: u8, x: i8, y: i8, x_diff: i8, y_diff: i8) -> Path {
		Path {
			key_type,
			move_count,
			before_x: x,
			before_y: y,
			before_x_diff: x_diff,
			before_y_diff: y_diff,
		}
	}
}