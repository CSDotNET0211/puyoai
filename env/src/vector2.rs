#[derive(Debug, Clone, Copy)]
pub struct Vector2 {
	pub x: i8,
	pub y: i8,
}

impl Vector2 {
	#[inline]
	pub fn new(x: i8, y: i8) -> Self {
		Self {
			x,
			y,
		}
	}
	#[inline]
	pub fn clone(&self) -> Self {
		Self {
			x: self.x,
			y: self.y,
		}
	}
	#[inline]
	pub fn default() -> Self {
		Self {
			x: 0,
			y: 0,
		}
	}
}