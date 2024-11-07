use ppc::{GameState, PpcPuyoKind};
use env::board::Board;
use env::puyo_kind::PuyoKind;
use env::puyo_status::PuyoStatus;

pub struct Field {
	pub board: Board,
	pub next: [(PuyoKind, PuyoKind); 2],
	pub current: Option<PuyoStatus>,
	pub is_movable: bool,
	pub current_chain: u8,
	pub movable_puyo: PuyoKind,
	pub center_puyo: PuyoKind,
}

impl Field {
	pub unsafe fn default() -> Self {
		Self {
			board: Board::default(),
			next: [(PuyoKind::Empty, PuyoKind::Empty), (PuyoKind::Empty, PuyoKind::Empty)],
			current: None,
			is_movable: false,
			current_chain: 0,
			//	current_game_state: GameState::Idle,
			//curernt_frame: 0,
			movable_puyo: PuyoKind::Empty,
			center_puyo: PuyoKind::Empty,
		}
	}
}