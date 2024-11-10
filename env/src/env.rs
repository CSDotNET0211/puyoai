use std::arch::x86_64::{__m128i, _mm_load_si128, _mm_set_epi64x, _mm_setzero_si128, _mm_store_si128};
use std::collections::VecDeque;
use std::sync::LazyLock;

use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;
use rand::thread_rng;

use crate::board::{Board, WIDTH_WITH_BORDER};
use crate::board_bit::BoardBit;
use crate::event_type::EventType;
use crate::ojama_status::OjamaStatus;
use crate::puyo_kind::PuyoKind;
use crate::puyo_status::PuyoStatus;
use crate::rotation::Rotation;
use crate::split_board::SplitBoard;
use crate::vector2::Vector2;

pub const PUYO_COUNT: usize = 4;
pub const HEIGHT: usize = 14;
pub const WIDTH: usize = 6;
pub const SPAWN_POS: (i8, i8) = (3, 12);
pub const ROTATE_KICKS: [[i8; 2]; 4] = [
	[-1, 0],
	[0, 1],
	[1, 0],
	[0, -1],
];

pub enum FrameNeeded {}

impl FrameNeeded {
	pub const LAND_PUYO_ANIMATION: usize = 15;
	pub const MOVE: usize = 2;
	pub const SPAWN_NEW_PUYO: usize = 2;
	pub const TEAR_PUYO_DROP_PER_1_BLOCK: usize = 2;
	pub const VANISH_PUYO_ANIMATION: usize = 48;
}

pub const ALL_CLEAR_BONUS: usize = 2100;
pub const OJAMA_POS: [u8; 6] = [1, 2, 3, 4, 5, 6];
pub const MAX_OJAMA_RECEIVE_COUNT: usize = 30;

pub const TEAR_FRAME: [u8; 14] = [0, 19, 24, 28, 31, 34, 37, 40, 42, 44, 46, 48, 48, 48];
pub const ROTATE_DIFF: [[i8; 2]; 4] = [
	[1, 0],
	[0, -1],
	[-1, 0],
	[0, 1],
];

#[derive(Debug)]
pub struct Event {
	pub kind: EventType,
	pub frame: usize,
	//pub until: usize,
	pub value: usize,
	pub value2: usize,
}

//pub type Board = [PuyoKind; WIDTH * HEIGHT];
//pub type BoardBool = [bool; WIDTH * HEIGHT];

pub static DEAD_POSITION: LazyLock<Vector2> = LazyLock::new(|| {
	let dead_pos = Vector2::new(3, 12);
	dead_pos
});

pub struct DebugStatus {
	pub current_chain_count: usize,
	pub current_chain_attack: usize,
}

impl DebugStatus {
	pub fn new() -> Self {
		Self {
			current_chain_attack: 0,
			current_chain_count: 0,
		}
	}
}

pub struct Env {
	pub board: Board,
	pub center_puyo: PuyoKind,
	pub movable_puyo: PuyoKind,
	pub puyo_status: PuyoStatus,
	pub next: [[PuyoKind; 2]; 2],
	pub current_frame: usize,
	pub current_score: usize,
	pub events: VecDeque<Event>,
	pub ojama: OjamaStatus,
	pub all_cleared: bool,
	pub dead: bool,
	rng: ThreadRng,
	bag: VecDeque<PuyoKind>,
	rand: u32,
	pub debug_status: DebugStatus,
	pub ojama_rate: usize,
}


impl Env {
	pub unsafe fn new(seed: &u32) -> Env {
		Env {
			board: Board::default(),
			center_puyo: PuyoKind::Empty,
			movable_puyo: PuyoKind::Empty,
			puyo_status: PuyoStatus::new(Vector2::new(0, 0), Rotation::new(0)),
			next: [[PuyoKind::Empty, PuyoKind::Empty], [PuyoKind::Empty, PuyoKind::Empty]],
			current_frame: 0,
			current_score: 0,
			events: VecDeque::new(),
			ojama: OjamaStatus(0),
			all_cleared: false,
			//queue_rng: StdRng::seed_from_u64(*seed),
			rng: thread_rng(),
			dead: false,
			bag: VecDeque::with_capacity(256),
			rand: *seed,
			debug_status: DebugStatus::new(),
			ojama_rate: 70,
		}
	}

	pub unsafe fn init(&mut self) {
		self.init_bag();

		self.pop_next();
		self.pop_next();
		self.create_new_puyo();
	}


	fn lcg_rand(&mut self) -> u32 {
		// 定数の定義
		let a: u32 = 0x5D588B65;
		let c: u32 = 0x269EC3;
		let m: u32 = 0xFFFFFFFF;

		// LCG の計算: rand = (rand * a + c) & m
		self.rand = (self.rand.wrapping_mul(a).wrapping_add(c)) & m;

		// 次の乱数値を返す
		self.rand
	}

	fn shuffle_bag(&mut self) {
		let sfl: [[i32; 3]; 3] = [
			[15, 8, 28],
			[7, 16, 27],
			[3, 32, 26]
		];

		for k in 0..3 {
			for i in 0..sfl[k][0] {
				for _ in 0..sfl[k][1] {
					let n1 = self.lcg_rand().wrapping_shr(sfl[k][2] as u32).wrapping_add((i * 0x10) as u32);
					let n2 = self.lcg_rand().wrapping_shr(sfl[k][2] as u32).wrapping_add(((i + 1) * 0x10) as u32);

					let temp = self.bag[n1 as usize];
					self.bag[n1 as usize] = self.bag[n2 as usize];
					self.bag[n2 as usize] = temp;
				}
			}
		}
	}

	fn init_bag(&mut self) {
		for i in 0..256 {
			let kind = PuyoKind::from_bits((i % PUYO_COUNT + 4) as u8);
			self.bag.push_back(kind);
		}
		self.shuffle_bag();
	}

	fn pop_bag(&mut self) -> PuyoKind {
		if self.bag.len() == 0 {
			self.init_bag();
		}

		self.bag.pop_front().unwrap()
	}

	fn pop_next(&mut self) -> [PuyoKind; 2] {
		let next_for_pop = self.next[0];
		self.next[0] = self.next[1];


		self.next[1][0] = self.pop_bag();
		self.next[1][1] = self.pop_bag();
		next_for_pop
	}

	pub unsafe fn create_new_puyo(&mut self) {
		self.debug_status.current_chain_count = 0;

		if self.ojama.get_receivable_ojama_size() != 0 {
			self.board.try_put_ojama(&mut self.ojama, &mut self.rng);
		}

		if !self.board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
			self.dead = true;
			return;
		}

		self.events.push_back(Event {
			frame: self.current_frame,
			kind: EventType::Wait,
			value: FrameNeeded::SPAWN_NEW_PUYO,
			value2: Default::default(),
		});

		//let poped_next = [PuyoKind::Red, PuyoKind::Red];
		let poped_next = self.pop_next();
		self.movable_puyo = poped_next[0];
		self.center_puyo = poped_next[1];
		self.puyo_status = PuyoStatus::new(Vector2::new(SPAWN_POS.0, SPAWN_POS.1), Rotation::new(3));
	}

	/*#[inline]
	pub fn get_puyo(board: &Board, x: i32, y: i32) -> Option<PuyoKind> {
		if y < 0 ||
			x < 0 ||
			x >= WIDTH as i32 ||
			y >= HEIGHT as i32
		{
			return None;
		}


		return Some(board.get_cell(x as i16, y as i16));
	}*/

	pub unsafe fn move_right(&mut self) {
		if Self::move_puyo(&self.board, &mut self.puyo_status, 1, 0) {
			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::MOVE,
				value2: Default::default(),
			});
		}
	}

	pub unsafe fn move_left(&mut self) {
		if Self::move_puyo(&self.board, &mut self.puyo_status, -1, 0) {
			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::MOVE,
				value2: Default::default(),
			});
		}
	}

	#[inline]
	pub unsafe fn is_valid_rotation(puyo_status: &PuyoStatus, board: &Board, cw: bool, kick: &mut Vector2) -> bool {
		let mut rotation = puyo_status.rotation;
		if cw {
			rotation.rotate_cw();
		} else {
			rotation.rotate_ccw();
		}

		let mut status = puyo_status.clone();

		let d_combi = ROTATE_DIFF[rotation.0 as usize];
		status.position_diff.x = d_combi[0];
		status.position_diff.y = d_combi[1];

		if Self::is_valid_position(board, &status, 0, 0) {
			*kick = Vector2::new(0, 0);
			return true;
		} else {
			let diff = ROTATE_KICKS[rotation.0 as usize];
			//	if Self::is_valid_position(board, &mut status, diff[0], diff[1]) {
			if Self::is_valid_position(board, &mut status, diff[0], diff[1]) {
				*kick = Vector2::new(diff[0], diff[1]);
				return true;
			}
		}

		return false;
	}

	#[inline]
	pub unsafe fn is_valid_position(board: &Board, puyo_status: &PuyoStatus, x_diff: i8, y_diff: i8) -> bool {
		let heights = board.get_heights();

		if Self::is_valid_block(puyo_status.position.x + x_diff, puyo_status.position.y + y_diff, &heights) &&
			Self::is_valid_block(puyo_status.position.x + puyo_status.position_diff.x + x_diff, puyo_status.position.y + puyo_status.position_diff.y + y_diff, &heights) &&
			puyo_status.position.y + y_diff <= 13 {
			return true;
		}

		return false;
	}

	#[inline]
	///引数のboardのフラグ基準
	pub fn is_valid_block(x: i8, y: i8, heights: &[u16; 8]) -> bool {
		if y < 0 ||
			x < 0 ||
			x >= WIDTH_WITH_BORDER as i8 ||
			y >= HEIGHT as i8 ||
			heights[x as usize] > y as u16 {
			return false;
		}

		return true;
	}

	#[inline]
	pub unsafe fn rotate_cw(&mut self) {
		let mut kick = Vector2::new(0, 0);
		if Self::is_valid_rotation(&self.puyo_status, &self.board, false, &mut kick) {
			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::MOVE,
				value2: Default::default(),
			});
			Self::rotate_puyo(&mut self.puyo_status, 1);
			Self::move_puyo(&self.board, &mut self.puyo_status, kick.x, kick.y);
		}
	}

	#[inline]
	pub unsafe fn rotate_ccw(&mut self) {
		let mut kick = Vector2::new(0, 0);
		if Self::is_valid_rotation(&self.puyo_status, &self.board, true, &mut kick) {
			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::MOVE,
				value2: Default::default(),
			});
			Self::rotate_puyo(&mut self.puyo_status, 0);
			Self::move_puyo(&self.board, &mut self.puyo_status, kick.x, kick.y);
		}
	}

	#[inline]
	pub unsafe fn rotate_180(&mut self) {
		Self::rotate_puyo(&mut self.puyo_status, 2);
		self.events.push_back(Event {
			frame: self.current_frame,
			kind: EventType::Wait,
			value: FrameNeeded::MOVE,
			value2: Default::default(),
		});
		if self.puyo_status.rotation.0 == 3 {
			self.puyo_status.position.y -= 1;
		} else if self.puyo_status.rotation.0 == 1 {
			self.puyo_status.position.y += 1;
		}
	}

	#[inline]
	pub unsafe fn update(&mut self) {
		self.current_frame += 1;

		let ojama_rate = match self.current_frame / 60 {
			v if v <= 95 => { 70 }
			v if v <= 111 => { 52 }
			v if v <= 127 => { 34 }
			v if v <= 143 => { 25 }
			v if v <= 159 => { 16 }
			v if v <= 175 => { 12 }
			v if v <= 191 => { 8 }
			v if v <= 207 => { 6 }
			v if v <= 223 => { 4 }
			v if v <= 239 => { 3 }
			v if v <= 255 => { 2 }
			v if v >= 256 => { 1 }
			_ => { panic!() }
		};

		if self.ojama_rate != ojama_rate {
			self.ojama_rate = ojama_rate;
		}

		/*	if self.events.len() != 0 {
				let event = self.events.pop_front().unwrap();
				//if event.0 <= self.current_frame as u32 {}
			}*/

		self.ojama.update_one_frame();
	}

	#[inline]
	pub unsafe fn quick_drop(&mut self, opponent: Option<&mut Env>) {
		let drop_count = self.board.put_puyo(&self.puyo_status, &self.center_puyo, &self.movable_puyo);

		self.center_puyo = PuyoKind::Empty;
		self.movable_puyo = PuyoKind::Empty;

		if drop_count > 0 {
			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK * drop_count as usize,
				value2: Default::default(),
			});

			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::LAND_PUYO_ANIMATION,
				value2: Default::default(),
			});
		}


		let mut chain: u8 = 0;
		let mut board_mask = BoardBit::default();

		let mut chain_score: usize = 0;
		let mut elapsed_frame = 0usize;
		loop {
			let score = self.board.erase_if_needed(&chain, &mut board_mask, &mut 0);
			if score == 0 {
				break;
			}

			if self.all_cleared {
				chain_score += ALL_CLEAR_BONUS;
				self.all_cleared = false;
			}

			self.events.push_back(Event {
				frame: self.current_frame,
				kind: EventType::Wait,
				value: FrameNeeded::VANISH_PUYO_ANIMATION,
				value2: Default::default(),
			});
			elapsed_frame += FrameNeeded::VANISH_PUYO_ANIMATION;

			self.current_score += score as usize;
			chain_score += score as usize;

			let drop_count = self.board.drop_after_erased(&board_mask);


			if drop_count > 0 {
				self.events.push_back(Event {
					frame: self.current_frame,
					kind: EventType::Wait,
					value: FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK * drop_count as usize,
					value2: Default::default(),
				});
				elapsed_frame += FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK * drop_count as usize;

				self.events.push_back(Event {
					frame: self.current_frame,
					kind: EventType::Wait,
					value: FrameNeeded::VANISH_PUYO_ANIMATION,
					value2: Default::default(),
				});
				elapsed_frame += FrameNeeded::LAND_PUYO_ANIMATION;
			}

			chain += 1;
		}

		self.debug_status.current_chain_count = chain as usize;

		if self.board.is_same(&_mm_setzero_si128(),
							  &_mm_set_epi64x(0b1111111111111111000000000000000100000000000000010000000000000001u64 as i64,
											  0b0000000000000001000000000000000100000000000000011111111111111111u64 as i64),
							  &_mm_setzero_si128()) {
			self.all_cleared = true;
		}


		let mut attack: usize = chain_score / self.ojama_rate;
		attack = self.ojama.offset(attack);

		if attack != 0 {
			if let Some(opponent) = opponent {
				opponent.ojama.push(attack, (self.current_frame + elapsed_frame).saturating_sub(opponent.current_frame));
			}
		}
	}


	//pub fn get_score();
	#[inline]
	pub unsafe fn move_puyo(board: &Board, puyo_status: &mut PuyoStatus, x_diff: i8, y_diff: i8) -> bool {
		if Self::is_valid_position(board, puyo_status, x_diff, y_diff) {
			puyo_status.position.x += x_diff;
			puyo_status.position.y += y_diff;

			return true;
		}
		return false;
	}

	///0:cw 1:ccw 2:180
	#[inline]
	pub fn rotate_puyo(puyo_status: &mut PuyoStatus, r_type: u8) {
		//let before = puyo_status.rotation;
		if r_type == 0 {
			puyo_status.rotation.rotate_cw();
		} else if r_type == 1 {
			puyo_status.rotation.rotate_ccw();
		} else if r_type == 2 {
			puyo_status.rotation.rotate_180();
		}
		//	let after = puyo_status.rotation;

		//後の位置でkickset判断

		let d_combi = ROTATE_DIFF[puyo_status.rotation.0 as usize];
		puyo_status.position_diff.x = d_combi[0];
		puyo_status.position_diff.y = d_combi[1];
	}

	#[inline]
	pub fn is_in_board(x: i32, y: i32) -> bool {
		if y < 0 ||
			x < 0 ||
			x >= WIDTH as i32 ||
			y >= HEIGHT as i32
		{
			return false;
		}

		return true;
	}
}