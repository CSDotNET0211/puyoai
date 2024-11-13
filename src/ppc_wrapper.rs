use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ppc::{GameState, PpcInput, PpcPuyoKind};
use ppc::GameState::Idle;
use ppc::ppc::PPC;
use ppc::scp::Controller;
use vigem_client::XButtons;

use ai::key_type::KeyType;
use env::board::Board;
use env::board_bit::BoardBit;
use env::env::{Env, FrameNeeded};
use env::puyo_kind::PuyoKind;
use env::puyo_status::PuyoStatus;
use env::rotation::Rotation;
use env::vector2::Vector2;

use crate::COLOR_PUYOS;

pub struct PpcWrapper {
	ppc: Arc<Mutex<PPC>>,
	player_index: usize,
	left_puyos: Arc<Mutex<Vec<PuyoKind>>>,
	puyo_mapping: Arc<Mutex<HashMap<PpcPuyoKind, PuyoKind>>>,
	pub inputs: Arc<Mutex<Vec<KeyType>>>,
	raw_board: Arc<Mutex<[PpcPuyoKind; 14 * 6]>>,
	current_state: Arc<Mutex<GameState>>,
	pub origin_pos: Arc<Mutex<Option<PuyoStatus>>>,
	controller: Arc<Mutex<Option<Controller>>>,
	pressing: Arc<Mutex<bool>>,
	pub env: Arc<Mutex<Env>>,
	pub is_movable: Arc<Mutex<bool>>,
	pub current_chain: Arc<Mutex<u8>>,
}


impl PpcWrapper {
	pub unsafe fn new(player_index: usize, scp: Option<Controller>) -> Self {
		let mut ppc_wrapper = Self {
			ppc: Arc::new(Mutex::new(PPC::new(player_index))),
			raw_board: Arc::new(Mutex::new([PpcPuyoKind::Null; 84])),
			puyo_mapping: Arc::new(Mutex::new(Default::default())),
			inputs: Arc::new(Mutex::new(Default::default())),
			left_puyos: Default::default(),
			player_index,
			current_state: Arc::new(Mutex::new(Idle)),
			origin_pos: Arc::new(Mutex::new(None)),
			controller: Arc::new(Mutex::new(scp)),
			pressing: Arc::new(Mutex::new(false)),
			env: Arc::new(Mutex::new(Env::new(&0))),
			is_movable: Arc::new(Mutex::new(false)),
			current_chain: Arc::new(Mutex::new(0)),
		};
		ppc_wrapper.reset_puyo_info();

		ppc_wrapper
	}

	pub fn reset_puyo_info(&mut self) {
		*self.left_puyos.lock().unwrap() = COLOR_PUYOS.to_vec();
		self.puyo_mapping.lock().unwrap().clear();
		self.inputs.lock().unwrap().clear();
		*self.origin_pos.lock().unwrap() = None;
		if self.controller.lock().unwrap().is_some() {
			(self.controller.lock().unwrap().as_mut().unwrap()).release_all();
		}
	}

	pub fn connect(&mut self) {
		self.ppc.lock().unwrap().connect();
	}

	//fn operate() {}


	///各種フィールド情報を更新
	pub unsafe fn update(&mut self, opponent: Arc<Mutex<PpcWrapper>>) {
		self.update_game_state();

		if *self.current_state.lock().unwrap() != GameState::Start {
			return;
		}


		let new_frame = self.ppc.lock().unwrap().get_frame();
		if self.env.lock().unwrap().current_frame != new_frame as usize {
			self.env.lock().unwrap().current_frame = new_frame as usize;
		} else {
			return;
		}
		if !self.ppc.lock().unwrap().is_active_window() {
			return;
		}
//println!("frame: {}", new_frame);

		let env = self.env.lock().as_mut().unwrap();
		let ppc = self.ppc.lock().as_mut().unwrap();
		let mut raw_board = *self.raw_board.lock().unwrap();

		self.env.update();

		self.ppc.get_board(&mut raw_board);
		Self::update_board(&mut raw_board, &mut env.board, &mut self.left_puyos, &mut self.puyo_mapping);
		Self::update_next(ppc, &mut env.next, &mut self.left_puyos, &mut self.puyo_mapping);

		Self::update_current(ppc, &mut env.puyo_status, &mut env.center_puyo, &mut env.movable_puyo, &mut left_puyos, &mut puyo_mapping);
		match self.ppc.get_is_movable() {
			Ok(value) => {
				self.is_movable = value;
			}
			Err(_) => {
				self.is_movable = false;
			}
		}

//		let think = Instant::now();

		//連鎖検知
		let new_current_chain = self.ppc.get_current_chain().unwrap();
		//	println!("{new_current_chain}");
		if self.current_chain != new_current_chain
			&& new_current_chain == 1 {
			//panic!();
			println!("連鎖 detect:{}", self.player_index);
			//連鎖開始
			self.ppc.get_board_otf(&mut self.raw_board);
			Self::update_board(&mut self.raw_board, &mut self.env.board, &mut self.left_puyos, &mut self.puyo_mapping);
			//OjamaStatus

			//TODO: 全消し判定もここで
			let mut chain: u8 = 0;
			let mut board_mask = BoardBit::default();
			let mut chain_score: usize = 0;
			let mut elapsed_frame = 0usize;

			println!("連鎖開始準備");
			let mut opponent = opponent.lock().unwrap();
			println!("連鎖開始しちゃうよん");
			loop {
				let score = self.env.board.erase_if_needed(&chain, &mut board_mask, &mut 0);
				if score == 0 {
					break;
				}

				elapsed_frame += FrameNeeded::VANISH_PUYO_ANIMATION;
				let drop_count = self.env.board.drop_after_erased(&board_mask);
				if drop_count > 0 {
					elapsed_frame += FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK * drop_count as usize;
					elapsed_frame += FrameNeeded::LAND_PUYO_ANIMATION;
				}

				chain_score += score as usize;
				chain += 1;
			}

			let ojama_rate = opponent.env.ojama_rate;
			dbg!((chain_score / ojama_rate));
			if chain_score == 40 {
				chain_score = 70;
			}
			dbg!(elapsed_frame);
			opponent.env.ojama.push(chain_score / ojama_rate, elapsed_frame);
		}

		self.current_chain = new_current_chain;

		if self.controller.is_some() {
			self.try_control();
		}
	}


	unsafe fn update_board(raw_board: &mut [PpcPuyoKind; 84], board: &mut Board, left_puyos: &mut Vec<PuyoKind>, puyo_mapping: &mut HashMap<PpcPuyoKind, PuyoKind>) {
		for y in 1..=13u8 {
			for x in 0..6u8 {
				let raw_puyo = raw_board[(x + (y) * 6) as usize];
				let puyo = Self::check_and_register_using_puyos(&raw_puyo, left_puyos, puyo_mapping);
				board.set_flag(&(x + 1), &(14 - (y + 1) + 1), &puyo);
			}
		}

		for x in 0..6u8 {
			let raw_puyo = raw_board[(x + 0 * 6) as usize];
			let puyo = Self::check_and_register_using_puyos(&raw_puyo, left_puyos, puyo_mapping);
			board.set_flag(&(x + 1), &14, &puyo);
		}
	}


	fn update_current(ppc: &mut PPC, puyo_status: &mut PuyoStatus, center_puyo: &mut PuyoKind, movable_puyo: &mut PuyoKind, left_puyos: &mut Vec<PuyoKind>, puyo_mapping: &mut HashMap<PpcPuyoKind, PuyoKind>) {
		let pos = ppc.get_current_pos();
		let rotation = ppc.get_current_rotation();
		let center = ppc.get_current_center_puyo();
		let movable = ppc.get_current_movable_puyo();


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

					*puyo_status = PuyoStatus::new(Vector2::new(pos.0, 16 - pos.1 - 1), convert_rotation);
					*center_puyo = Self::check_and_register_using_puyos(&center, left_puyos, puyo_mapping);
					*movable_puyo = Self::check_and_register_using_puyos(&movable, left_puyos, puyo_mapping);

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

	fn update_next(ppc: &mut PPC, next: &mut [[PuyoKind; 2]; 2], left_puyos: &mut Vec<PuyoKind>, puyo_mapping: &mut HashMap<PpcPuyoKind, PuyoKind>) {
		for next_index in 0..=1usize {
			match ppc.get_queue(next_index as u8) {
				Ok((value1, value2)) => {
					next[next_index] = [Self::check_and_register_using_puyos(&value1, left_puyos, puyo_mapping),
						Self::check_and_register_using_puyos(&value2, left_puyos, puyo_mapping)];
				}
				Err(_) => {
					next[next_index] = [PuyoKind::Empty, PuyoKind::Empty];
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

		let new_pos = self.env.puyo_status.clone();
		if self.origin_pos.is_none() {
			if !self.is_movable {
				return;
			}

			self.origin_pos = Some(new_pos.clone());
			//	dbg!(&new_pos);
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
				let state = self.is_movable;
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

	fn check_and_register_using_puyos(raw_puyo: &PpcPuyoKind, left_puyos: &mut Vec<PuyoKind>, puyo_mapping: &mut HashMap<PpcPuyoKind, PuyoKind>) -> PuyoKind {
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
		if !puyo_mapping.contains_key(raw_puyo) {
//	if !*using_puyos.contains(raw_puyo) {
			let temp = Self::convert_puyo_kind(raw_puyo);
			let mut result = left_puyos.iter().position(|x| *x == temp);
			if result == None {
				result = Option::from(0usize);
			}

			let index = result.unwrap();
			let selected_puyo = left_puyos[index];

			puyo_mapping.insert(*raw_puyo, selected_puyo);
			println!("added {:?} to {:?}", raw_puyo, selected_puyo);
			left_puyos.remove(index);
			selected_puyo
		} else {
			puyo_mapping[raw_puyo]
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
