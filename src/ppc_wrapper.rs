use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use ppc::{GameState, PpcInput, PpcPuyoKind};
use ppc::GameState::Idle;
use ppc::ppc::PPC;
use ppc::scp::Controller;
use vigem_client::XButtons;

use ai::key_type::KeyType;
use ai::opponent_status::OpponentStatus;
use env::board::Board;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::PuyoKind;
use env::puyo_status::PuyoStatus;
use env::rotation::Rotation;
use env::vector2::Vector2;

use crate::COLOR_PUYOS;
use crate::field::Field;

pub struct PpcWrapper {
	ppc: PPC,
	pub field: Arc<Mutex<Field>>,
	pub opponent_field: Arc<Mutex<Field>>,
	player_index: usize,
	left_puyos: Vec<PuyoKind>,
	puyo_mapping: HashMap<PpcPuyoKind, PuyoKind>,
	//pub on_complete_action: Option<Box<dyn Fn(&mut PpcWrapper) + Send>>,
	//pub on_gamestate_changed: Option<Box<dyn Fn(GameState, &mut PpcWrapper) + Send>>,
	pub inputs: Vec<KeyType>,
	raw_board: [PpcPuyoKind; 14 * 6],
	raw_board_otf: [PpcPuyoKind; 14 * 6],
	current_frame: u32,
	current_state: GameState,
	pub origin_pos: Option<PuyoStatus>,
	opponent_index: usize,
	pub opponent_status: OpponentStatus,
	pub opponent_board: Board,
	raw_opponent_board: [PpcPuyoKind; 14 * 6],
	pub ojama_status: OjamaStatus,
	controller: Option<Controller>,
	pressing: bool,
}


impl PpcWrapper {
	pub unsafe fn new(player_index: usize, opponent_index: usize, scp: Option<Controller>) -> Self {
		let mut ppc_wrapper = Self {
			ppc: PPC::new(player_index),
			raw_board: [PpcPuyoKind::Null; 84],
			raw_board_otf: [PpcPuyoKind::Null; 84],
			puyo_mapping: Default::default(),
			inputs: Default::default(),
			left_puyos: Default::default(),
			field: Arc::new(Mutex::new(Field::default())),
			opponent_field: Arc::new(Mutex::new(Field::default())),
			player_index,
			opponent_status: OpponentStatus::default(),
			ojama_status: OjamaStatus(0),
			current_frame: 0,
			current_state: Idle,
			origin_pos: None,
			opponent_index,
			opponent_board: Board::default(),
			raw_opponent_board: [PpcPuyoKind::Null; 14 * 6],
			controller: scp,
			pressing: false,
		};
		ppc_wrapper.reset_puyo_info();

		ppc_wrapper
	}

	pub fn reset_puyo_info(&mut self) {
		self.left_puyos = COLOR_PUYOS.to_vec();
		self.puyo_mapping.clear();
		self.inputs.clear();
		self.origin_pos = None;
		if self.controller.is_some() {
			self.controller.as_mut().unwrap().release_all();
		}
	}

	pub fn connect(&mut self) {
		self.ppc.connect();
	}

	//fn operate() {}


	///各種フィールド情報を更新
	pub unsafe fn update(&mut self) {
		self.update_game_state();

		if self.current_state != GameState::Start {
			return;
		}

		let new_frame = self.ppc.get_frame();
		if self.current_frame != new_frame {
			self.current_frame = new_frame;
		} else {
			return;
		}

		if !self.ppc.is_active_window() {
			return;
		}


		self.update_board();

		self.ppc.get_board(&mut self.raw_opponent_board);
		for y in 1..=13u8 {
			for x in 0..6u8 {
				let raw_puyo = self.raw_opponent_board[(x + (y) * 6) as usize];
				let puyo = self.check_and_register_using_puyos(&raw_puyo);
				self.opponent_board.set_flag(&(x + 1), &(14 - (y + 1) + 1), &puyo);
			}
		}
		for x in 0..6u8 {
			let raw_puyo = self.raw_opponent_board[(x + 0 * 6) as usize];
			let puyo = self.check_and_register_using_puyos(&raw_puyo);
			self.opponent_board.set_flag(&(x + 1), &14, &puyo);
		}


		self.update_next();


		//	println!("board end");
		//	}
//		self.update_opponent_board(&mut self.raw_opponent_board, &mut self.opponent_board);
		self.update_current();
		match self.ppc.get_is_movable() {
			Ok(value) => {
				self.field.lock().unwrap().is_movable = value;
			}
			Err(_) => {
				self.field.lock().unwrap().is_movable = false;
			}
		}

//		let think = Instant::now();

		let new_current_chain = self.ppc.get_current_chain().unwrap();
		if self.field.lock().unwrap().current_chain != new_current_chain
			&& new_current_chain == 1 {
			//連鎖開始
			let board_otf = self.ppc.get_board_otf(&mut self.raw_board_otf);
		}
		self.field.lock().unwrap().current_chain = new_current_chain;


		if self.current_frame % 60 == 0 {
			self.opponent_status = OpponentStatus::new(&self.opponent_board);
		}


		self.try_control();
	}

	unsafe fn update_board(&mut self/*, raw_board: &mut [PpcPuyoKind; 84], board: &mut Board*/) {
		self.ppc.get_board(&mut self.raw_board);
		for y in 1..=13u8 {
			for x in 0..6u8 {
				let raw_puyo = self.raw_board[(x + (y) * 6) as usize];
				let puyo = self.check_and_register_using_puyos(&raw_puyo);
				self.field.lock().unwrap().board.set_flag(&(x + 1), &(14 - (y + 1) + 1), &puyo);
			}
		}

		for x in 0..6u8 {
			let raw_puyo = self.raw_board[(x + 0 * 6) as usize];
			let puyo = self.check_and_register_using_puyos(&raw_puyo);
			self.field.lock().unwrap().board.set_flag(&(x + 1), &14, &puyo);
		}
	}


	fn update_current(&mut self) {
		let pos = self.ppc.get_current_pos();
		let rotation = self.ppc.get_current_rotation();
		let center = self.ppc.get_current_center_puyo();
		let movable = self.ppc.get_current_movable_puyo();


		match (pos, rotation, center, movable) {
			(Ok(pos), Ok(rotation), Ok(center), Ok(movable)) => {
				if pos.0 == 0 || pos.1 == 0 || center == PpcPuyoKind::Null || movable == PpcPuyoKind::Null {} else {
					let convert_rotation = match rotation {
						0 => Rotation(3),
						1 => Rotation(2),
						2 => Rotation(1),
						3 => Rotation(0),
						_ => { panic!() }
					};

					self.field.lock().unwrap().current = Option::from(PuyoStatus::new(Vector2::new(pos.0, 16 - pos.1 - 1), convert_rotation));
					self.field.lock().unwrap().center_puyo = self.check_and_register_using_puyos(&center);
					self.field.lock().unwrap().movable_puyo = self.check_and_register_using_puyos(&movable);

					/*Option::from(PpcPuyoStatus {
						center_puyo: PpcPuyoKind::from(center),
						movable_puyo: PpcPuyoKind::from(movable),
						rotation: rotation,
						position: pos,
					})*/
				}
			}
			(Err(e), _, _, _) | (_, Err(e), _, _) | (_, _, Err(e), _) | (_, _, _, Err(e)) => {}
		};
	}

	fn update_next(&mut self) {
		for next_index in 0..=1usize {
			match self.ppc.get_queue(next_index as u8) {
				Ok((value1, value2)) => {
					self.field.lock().unwrap().next[next_index] = (self.check_and_register_using_puyos(&value1), self.check_and_register_using_puyos(&value2));
				}
				Err(_) => {
					self.field.lock().unwrap().next[next_index] = (PuyoKind::Empty, PuyoKind::Empty);
				}
			}
		}
	}

	fn update_game_state(&mut self) {
		let new_state = self.ppc.get_state();
		if new_state != self.current_state {
			dbg!(new_state);
			match new_state {
				GameState::Idle => {}
				GameState::Start => {
					self.reset_puyo_info();
				}
				GameState::Run => {}
				GameState::End => {}
			}
			self.current_state = new_state;
		}


		//	self.field.lock().unwrap().current_game_state = new_state;
	}


	fn try_control(&mut self) {
		if self.inputs.len() == 0 /* || self.origin_pos.is_none()*/ {
			return;
		}

		let new_pos = self.field.lock().as_ref().unwrap().current.as_ref().unwrap().clone();
		if self.origin_pos.is_none() {
			if !self.field.lock().unwrap().is_movable {
				return;
			}

			self.origin_pos = Some(new_pos.clone());
			dbg!(&new_pos);
		}
		//inputs[0]のやつをxboxの入力に直す
		let xbox_button = match self.inputs[0] {
			KeyType::Right => { XButtons::RIGHT }
			KeyType::Left => { XButtons::LEFT }
			KeyType::Drop => { XButtons::DOWN }
			KeyType::RotateRight => { XButtons::A }
			KeyType::RotateLeft => { XButtons::B }
			KeyType::Rotate180 => { XButtons::X }
			_ => { panic!() }
		};


		match xbox_button {
			XButtons::RIGHT | XButtons::LEFT | XButtons::A | XButtons::B => {
				if self.pressing {
					//	println!("離した！");
					self.controller.as_mut().unwrap().release_all();
				} else {
					//	println!("押した！");
					self.controller.as_mut().unwrap().press(XButtons::from(xbox_button));
				}
				self.pressing = !self.pressing;
			}
			XButtons::X => {
				if self.pressing {
					self.controller.as_mut().unwrap().release_all();
				} else {
					self.controller.as_mut().unwrap().press(XButtons::from(XButtons::A));
				}
				self.pressing = !self.pressing;
			}
			XButtons::DOWN => {
				//	println!("長押し");
				self.controller.as_mut().unwrap().press(XButtons::from(xbox_button));
			}

			_ => {}
		}


		let result = match xbox_button {
			XButtons::RIGHT => {
				self.origin_pos.as_ref().unwrap().position.x + 1 == new_pos.position.x
			}
			XButtons::LEFT => {
				self.origin_pos.as_ref().unwrap().position.x - 1 == new_pos.position.x
			}
			XButtons::B => {
				let mut r = self.origin_pos.as_ref().unwrap().rotation.0 as i8;
				r -= 1;
				if r == -1 {
					r = 3;
				}


				r == new_pos.rotation.0 as i8
			}
			XButtons::A => {
				let mut r = self.origin_pos.as_ref().unwrap().rotation.0 as i8;
				r += 1;
				if r == 4 {
					r = 0;
				}
				r == new_pos.rotation.0 as i8
			}
			XButtons::X => {//180回転
				let mut r = self.origin_pos.as_ref().unwrap().rotation.0 as i8;
				r += 2;
				r %= 4;
				r == new_pos.rotation.0 as i8
			}
			XButtons::DOWN => {
				//println!("{}", current_pos.1);
				//	let new_put_count = self.ppc.get_puyo_put_count();
				let state = self.field.lock().unwrap().is_movable;
				//let state = self.ppc.get_movable_state();
				if state {
					false
				} else {
					true
				}
			}
			_ => { panic!() }
		};

		if result {
			self.controller.as_mut().unwrap().release_all();
			self.pressing = false;
			self.origin_pos = None;
			//dbg!(self.inputs[0]);
			//	println!("reset");
			self.inputs.remove(0);
		}
	}

	fn check_and_register_using_puyos(&mut self, raw_puyo: &PpcPuyoKind) -> PuyoKind {
		match raw_puyo {
			PpcPuyoKind::Null => {
				return Self::convert_puyo_kind(raw_puyo);
			}
			PpcPuyoKind::Garbage => {
				return Self::convert_puyo_kind(raw_puyo);
			}
			_ => {}
		}


//mappingになくて変換失敗したら

//let 
		if !self.puyo_mapping.contains_key(raw_puyo) {
//	if !*using_puyos.contains(raw_puyo) {
			let temp = Self::convert_puyo_kind(raw_puyo);
			let mut result = self.left_puyos.iter().position(|x| *x == temp);
			if result == None {
				result = Option::from(0usize);
			}

			let index = result.unwrap();
			let selected_puyo = self.left_puyos[index];

			self.puyo_mapping.insert(*raw_puyo, selected_puyo);
			println!("added {:?} to {:?}", raw_puyo, selected_puyo);
			self.left_puyos.remove(index);
			selected_puyo
		} else {
			self.puyo_mapping[raw_puyo]
		}
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
