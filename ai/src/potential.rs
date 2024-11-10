use env::board::Board;
use env::vector2::Vector2;

pub struct Potential {
	pub chain: u8,
	pub added_count: u8,
	///置いた結果のboard
	pub diff_board: Board,
	pub empty_around_count: u8,
	pub added_pos: Vector2,
}

impl Potential {
	pub unsafe fn default() -> Self {
		Potential {
			added_count: 0,
			chain: 0,
			diff_board: Board::default(),
			empty_around_count: 0,
			added_pos: Vector2::default(),
		}
	}
}