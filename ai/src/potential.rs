use env::board::Board;
use env::vector2::Vector2;

#[derive(Debug, Clone)]
pub struct Potential {
	pub chain: u8,
	pub added_count: u8,
	///置いた結果のboard
	//pub diff_board: Board,
	pub near_empty_count: u8,
	pub ignite_pos: Vector2,
}

impl Potential {
	pub fn new(chain: u8, added_count: u8, diff_board: Board, empty_around_count: u8, added_pos: Vector2) -> Self {
		Potential {
			chain,
			added_count,
		//	diff_board,
			near_empty_count: empty_around_count,
			ignite_pos: added_pos,
		}
	}
	pub unsafe fn default() -> Self {
		Potential {
			added_count: 0,
			chain: 0,
		//	diff_board: Board::default(),
			near_empty_count: 0,
			ignite_pos: Vector2::default(),
		}
	}
}