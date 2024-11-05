use std::sync::{Arc, Mutex, MutexGuard};
use ppc::{PpcInput, PpcPuyoKind};
use ppc::field::Field;
use ai::key_type::KeyType;
use env::puyo_kind::PuyoKind;
use ppc::ppc::PPC;
use ppc::PpcPuyoKind::Null;
use ppc::scp::Controller;
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
	pub unsafe fn new(player_index: usize, scp: Controller) -> Self {
		let mut ppc_wrapper = Self {
			ppc: PPC::new(player_index),
			board: None,
			queue: None,
			current: None,
			is_movable: false,
		};

		ppc_wrapper.ppc.register_update(Some(Box::new(|| {
			ppc_wrapper.update();
		})));

		ppc_wrapper.update();

		ppc_wrapper
	}


	///各種フィールド情報を更新
	fn update(field: Arc<Mutex<Field>>) {


		//取得できなかった奴はnone
		if field.lock().unwrap().board.is_none() {
			field.lock().unwrap().board = Default::default();
		}
		self.get_field(self.player_index as u8, &mut self.field.lock().unwrap().board.unwrap());

		/*if self.next.lock().unwrap().is_none() {
			*self.next.lock().unwrap() = Default::default();
		}*/
		match self.get_queue(0) {
			Ok((value1, value2)) => {
				self.field.lock().unwrap().next.unwrap()[0] = (value1, value2);
			}
			Err(_) => { self.field.lock().unwrap().next.unwrap()[0] = (PpcPuyoKind::Null, PpcPuyoKind::Null) }
		}


		match self.get_queue(1) {
			Ok((value1, value2)) => {
				self.field.lock().unwrap().next.unwrap()[1] = (value1, value2);
			}
			Err(_) => { self.field.lock().unwrap().next.unwrap()[1] = (PpcPuyoKind::Null, PpcPuyoKind::Null) }
		}


		match self.get_is_movable() {
			Ok(value) => {
				self.field.lock().unwrap().is_movable = value;
			}
			Err(_) => {
				self.field.lock().unwrap().is_movable = false;
			}
		}


		let pos = self.get_current_pos();
		let rotation = self.get_current_rotation();
		let center = self.get_current_center_puyo(0);
		let movable = self.get_current_movable_puyo(0);


		self.field.lock().unwrap().current = match (pos, rotation, center, movable) {
			(Ok(pos), Ok(rotation), Ok(center), Ok(movable)) => {
				if pos.0 == 0 || pos.1 == 0 || center == PpcPuyoKind::Null || movable == Null {
					None
				} else {
					Option::from(PpcPuyoStatus {
						center_puyo: PpcPuyoKind::from(center),
						movable_puyo: PpcPuyoKind::from(movable),
						rotation: rotation,
						position: pos,
					})
				}
			}
			(Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => {
				None
			}
		};


		self.on_update.unwrap();
	}

	fn update_game_state(&mut self) {
		let new_state = self.get_state();
		if new_state != self.field.lock().unwrap().current_game_state {
			if self.on_gamestate_changed.is_some() {
				self.on_gamestate_changed.unwrap()(self.field.lock().unwrap().current_game_state);
			}
		}

		self.field.lock().unwrap().current_game_state = new_state;
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
