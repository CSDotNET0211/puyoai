use env::board::Board;

pub struct Potential {
	pub chain: u8,
	pub added_count: u8,
	///置いた結果のboard
	pub diff_board: Board,
}

impl Potential {
	pub unsafe fn default() -> Self {
		Potential {
			added_count: 0,
			chain: 0,
			diff_board:Board::new()
		}
	}
}