use std::sync::{Arc, Mutex, MutexGuard};
use ppc::{PpcInput, PpcPuyoKind};
use ai::key_type::KeyType;
use env::puyo_kind::PuyoKind;
use ppc::ppc::PPC;
use env::board::Board;
use env::puyo_status::PuyoStatus;
use crate::check_and_register_using_puyos;

pub struct PpcWrapper {
	ppc: PPC,
	board: Option<Board>,
	queue: Option<[(PpcPuyoKind, PpcPuyoKind); 2]>,
	current: Option<PuyoStatus>,
	is_movable: bool,
}


impl PpcWrapper {
	pub unsafe fn new() -> Self {
		let callback = Some(Box::new(|| {
			update()
		}));

		let mut s = Self {
			ppc: PPC::new(callback),
			board: None,
			queue: None,
			current: None,
			is_movable: false,
		};

		s.update();

		s
	}
	pub unsafe fn update(&mut self) {
		match *self.ppc.board.lock().unwrap() {
			Some(board) => {
				if self.board.is_none() {
					self.board = Option::from(Board::new());
				}

				for y in 1..=13 {
					for x in 0..6 {
						let raw_puyo = &board[(x + (y) * 6) as usize];
						self.board.set_flag(x + 1, 14 - (y + 1) + 1, &check_and_register_using_puyos(&raw_puyo, &mut left_puyos, &mut puyo_mapping));
					}
				}

				for x in 0..6 {
					let raw_puyo = &board[(x + 0 * 6) as usize];
					self.board.set_flag(x + 1, 14, &check_and_register_using_puyos(&raw_puyo, &mut left_puyos, &mut puyo_mapping));
				}
			}
			None => {
				self.board = None
			}
		}

		match *self.ppc.next.lock().unwrap() {
			Some(next) => {
				self.queue = next;
			}
			None => {
				self.queue = None;
			}
		}

		self.is_movable = self.ppc.is_movable.lock().unwrap();
	}
	pub fn queue(&self) {}
	pub fn current_queue(&self) {}
	pub fn is_movable(&self) -> bool {
		self.ppc.is_movable.lock().unwrap()
	}


	fn convert_puyo_kind(ppc_puyo_kind: &PpcPuyoKind) -> PuyoKind {
		match *ppc_puyo_kind {
			PpcPuyoKind::Null => { PuyoKind::Empty }
			PpcPuyoKind::Red => { PuyoKind::Red }
			PpcPuyoKind::Green => { PuyoKind::Green }
			PpcPuyoKind::Blue => { PuyoKind::Blue }
			PpcPuyoKind::Yellow => { PuyoKind::Yellow }
			PpcPuyoKind::Purple => { PuyoKind::Preserved }//なければなんでもいい
			PpcPuyoKind::Garbage => { PuyoKind::Ojama }
		}
	}

	fn convert_key_input(key_type: &KeyType) -> PpcInput {
		match *key_type {
			KeyType::Right => { PpcInput::Right }
			KeyType::Left => { PpcInput::Left }
			KeyType::Top => { panic!() }
			KeyType::Down => { panic!() }
			KeyType::Drop => { PpcInput::Down }
			KeyType::RotateRight => { PpcInput::RotateRight }
			KeyType::RotateLeft => { PpcInput::RotateLeft }
			KeyType::Rotate180 => { PpcInput::Rotate180 }
		}
	}
}