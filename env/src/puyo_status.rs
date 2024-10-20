use std::ops::Add;
use crate::env::ROTATE_DIFF;

use crate::rotation::Rotation;
use crate::vector2::Vector2;

#[derive(Debug)]
pub struct PuyoStatus {
	pub position: Vector2,
	pub rotation: Rotation,
	pub position_diff: Vector2,
}

impl Add<i8> for Rotation {
	type Output = Rotation;

	fn add(self, rhs: i8) -> Self::Output {
		let result = (self.0 as i8 + rhs) & 0b11;
		Rotation::new(result as u8)
	}
}

impl PuyoStatus {
	pub fn new(position: Vector2, rotation: Rotation) -> PuyoStatus {
		PuyoStatus {
			position,
			rotation,
			position_diff: Vector2::new(ROTATE_DIFF[rotation.0 as usize][0], ROTATE_DIFF[rotation.0 as usize][1]),
		}
	}

	pub fn clone(&self) -> PuyoStatus {
		PuyoStatus {
			position: self.position.clone(),
			rotation: self.rotation,
			position_diff: self.position_diff.clone(),
		}
	}

	pub fn create_hash(&self, x_diff: i8, r_diff: i8) -> u16 {
		//中心ぷよの位置と回転情報の方がシンプル
		//rrrxyy
		//TODO: yどうしようか
		let r = self.rotation + r_diff;


		r.value() as u16 * 1000 +
			(self.position.x + x_diff) as u16 * 100 +
			self.position.y as u16 * 1
	}
}