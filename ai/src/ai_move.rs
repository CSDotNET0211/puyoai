use crate::key_type::KeyType;

#[derive(PartialEq, Debug,Clone)]
pub struct AIMove {
	pub eval: f32,
	pub path: Vec<KeyType>,
}

impl AIMove {
	pub fn new(eval: f32, path: Vec<KeyType>) -> AIMove {
		AIMove {
			eval,
			path,
		}
	}
}