#[derive(Debug)]
pub struct Vector2 {
	pub x: i8,
	pub y: i8,
}

impl Vector2 {
	pub fn new(x: i8, y: i8) -> Vector2 {
		Vector2 {
			x,
			y,
		}
	}

	pub fn clone(&self) -> Vector2 {
		Vector2 {
			x: self.x,
			y: self.y,
		}
	}
}