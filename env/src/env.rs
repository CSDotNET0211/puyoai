use std::arch::x86_64::{__m128i, _mm_load_si128, _mm_set_epi64x, _mm_setzero_si128, _mm_store_si128};
use std::collections::VecDeque;
use std::sync::LazyLock;
use rand::{Rng, thread_rng};
use rand::prelude::SliceRandom;
use rand::rngs::ThreadRng;

use crate::board::{Board, WIDTH_WITH_BORDER};
use crate::board_bit::BoardBit;
use crate::event_type::EventType;
use crate::event_type::EventType::Attack;
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

//pub const OJAMA_RATE: [u8; 12] = [70, 52, 34, 25, 16, 12, 8, 6, 4, 3, 2, 1];

pub const OJAMA_POS: [u8; 6] = [1, 2, 3, 4, 5, 6];


pub const TEAR_FRAME: [u8; 14] = [0, 19, 24, 28, 31, 34, 37, 40, 42, 44, 46, 48, 48, 48];
pub const ROTATE_DIFF: [[i8; 2]; 4] = [
	[1, 0],
	[0, -1],
	[-1, 0],
	[0, 1],
];

//pub type Board = [PuyoKind; WIDTH * HEIGHT];
//pub type BoardBool = [bool; WIDTH * HEIGHT];

pub static DEAD_POSITION: LazyLock<Vector2> = LazyLock::new(|| {
	let dead_pos = Vector2::new(3, 12);
	dead_pos
});


pub struct Env {
	pub board: Board,
	pub center_puyo: PuyoKind,
	pub movable_puyo: PuyoKind,
	pub puyo_status: PuyoStatus,
	pub next: [[PuyoKind; 2]; 2],
	pub current_frame: usize,
	pub current_score: usize,
	pub events: VecDeque<(u32, EventType, u32)>,
	pub ojama: OjamaStatus,
	pub all_clear: bool,
	pub dead: bool,
	rng: ThreadRng,
	bag: VecDeque<PuyoKind>,
	rand: u32,
}


impl Env {
	pub unsafe fn new(seed: &u32) -> Env {
		Env {
			board: Board::new(),
			center_puyo: PuyoKind::Empty,
			movable_puyo: PuyoKind::Empty,
			puyo_status: PuyoStatus::new(Vector2::new(0, 0), Rotation::new(0)),
			next: [[PuyoKind::Empty, PuyoKind::Empty], [PuyoKind::Empty, PuyoKind::Empty]],
			current_frame: 0,
			current_score: 0,
			events: VecDeque::new(),
			ojama: OjamaStatus(0),
			all_clear: false,
			//queue_rng: StdRng::seed_from_u64(*seed),
			rng: thread_rng(),
			dead: false,
			bag: VecDeque::with_capacity(256),
			rand: *seed,
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

	unsafe fn create_new_puyo(&mut self) {
		//if self.board[DEAD_POSITION.x as usize + DEAD_POSITION.y as usize * WIDTH] != PuyoKind::Empty {
		if !self.board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
			self.dead = true;
			return;
		}

		self.current_frame += 2;

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
			self.current_frame += 2;
		}
	}

	pub unsafe fn move_left(&mut self) {
		if Self::move_puyo(&self.board, &mut self.puyo_status, -1, 0) {
			self.current_frame += 2;
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

		let d_combi = ROTATE_DIFF[rotation.value() as usize];
		status.position_diff.x = d_combi[0];
		status.position_diff.y = d_combi[1];

		if Self::is_valid_position(board, &status, 0, 0) {
			*kick = Vector2::new(0, 0);
			return true;
		} else {
			let diff = ROTATE_KICKS[rotation.value() as usize];
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
	pub unsafe fn try_put_ojama(&mut self) {
		//一度に30個まで
		//TODO: 全体的にマジックナンバー何とかしろ、あとお邪魔ってマスクとかいるよね？14
		//offsetを使って使ったお邪魔を減らす
		let mut ojama_to_receive = self.ojama.get_receivable_ojama_size();


		if ojama_to_receive > 30 {
			ojama_to_receive = 30;
		}
		self.ojama.use_ojama(ojama_to_receive);

		let heights = self.board.get_heights();

		let row = ojama_to_receive / 6;

		//お邪魔用のbitを作成し、状態を上書きする。
		let ojama_mask_column_size: u16 = (1 << row) - 1;

		let mut v1: SplitBoard = SplitBoard([0; 8]);
		let mut v2: SplitBoard = SplitBoard([0; 8]);
		let mut v3: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(v1.0.as_mut_ptr() as *mut __m128i, self.board.0[0]);
		_mm_store_si128(v2.0.as_mut_ptr() as *mut __m128i, self.board.0[1]);
		_mm_store_si128(v3.0.as_mut_ptr() as *mut __m128i, self.board.0[2]);


		for x in 1..=6 {
			let ojama_mask_column = ojama_mask_column_size << heights[x];
			BoardBit::set_bit_true_column(&mut v1.0[x], &ojama_mask_column);
			BoardBit::set_bit_false_column(&mut v2.0[x], &ojama_mask_column);
			BoardBit::set_bit_false_column(&mut v3.0[x], &ojama_mask_column);
		}

		let ojama_pos_slice = &mut OJAMA_POS;  // Borrow the slice here to extend its lifetime
		let selected_columns = ojama_pos_slice.choose_multiple(&mut self.rng, (ojama_to_receive % 30) as usize);


		//	let ojama_pos_slice = OJAMA_POS.as_mut_slice();
		//	let selected_columns = ojama_pos_slice.as_mut().choose_multiple(&mut self.rng, (ojama_to_receive % 30) as usize);

		for &pos in selected_columns {
			let ojama_mask_column = 1 << heights[pos as usize];

			BoardBit::set_bit_true_column(&mut v1.0[pos as usize], &ojama_mask_column);
			BoardBit::set_bit_false_column(&mut v2.0[pos as usize], &ojama_mask_column);
			BoardBit::set_bit_false_column(&mut v3.0[pos as usize], &ojama_mask_column);
		}


		self.board.0[0] = _mm_load_si128(v1.0.as_ptr() as *const __m128i);
		self.board.0[1] = _mm_load_si128(v2.0.as_ptr() as *const __m128i);
		self.board.0[2] = _mm_load_si128(v3.0.as_ptr() as *const __m128i);
	}

	#[inline]
	pub unsafe fn rotate_cw(&mut self) {
		let mut kick = Vector2::new(0, 0);
		if Self::is_valid_rotation(&self.puyo_status, &self.board, false, &mut kick) {
			self.current_frame += 2;
			Self::rotate_puyo(&mut self.puyo_status, 1);
			Self::move_puyo(&self.board, &mut self.puyo_status, kick.x, kick.y);
		}
	}

	#[inline]
	pub unsafe fn rotate_ccw(&mut self) {
		let mut kick = Vector2::new(0, 0);
		if Self::is_valid_rotation(&self.puyo_status, &self.board, true, &mut kick) {
			self.current_frame += 2;
			Self::rotate_puyo(&mut self.puyo_status, 0);
			Self::move_puyo(&self.board, &mut self.puyo_status, kick.x, kick.y);
		}
	}

	#[inline]
	pub unsafe fn rotate_180(&mut self) {
		Self::rotate_puyo(&mut self.puyo_status, 2);
		self.current_frame += 2;
		if self.puyo_status.rotation.0 == 3 {
			self.puyo_status.position.y -= 1;
		} else if self.puyo_status.rotation.0 == 1 {
			self.puyo_status.position.y += 1;
		}
	}

	#[inline]
	pub unsafe fn update(&mut self) {
		self.current_frame += 1;

	/*	if self.events.len() != 0 {
			let event = self.events.pop_front().unwrap();
			//if event.0 <= self.current_frame as u32 {}
		}*/

		self.ojama.update_one_frame();
	}

	#[inline]
	pub unsafe fn quick_drop(&mut self, opponent: Option<&mut Env>) {
		let drop_count = self.board.put_puyo(&self.puyo_status, &self.center_puyo, &self.movable_puyo);
		self.current_frame += 2 * drop_count as usize;
		if drop_count > 0 {
			self.current_frame += 10 as usize;
		}

		let mut chain: u8 = 0;
		let mut board_mask = BoardBit::default();

		let mut chain_score: u32 = 0;
		loop {
			let score = self.board.erase_if_needed(chain as i32, &mut board_mask);
			if score == 0 {
				break;
			}

			if self.all_clear {
				chain_score += 2100;
				self.all_clear = false;
			}

			self.current_frame += 48 as usize;
			self.current_score += score as usize;
			chain_score += score;

			let drop_count = self.board.drop_after_erased(&board_mask);
			self.current_frame += 2 * drop_count as usize;
			chain += 1;
		}

		if self.board.is_same(&_mm_setzero_si128(),
							  &_mm_set_epi64x(0b1111111111111111000000000000000100000000000000010000000000000001u64 as i64,
											  0b0000000000000001000000000000000100000000000000011111111111111111u64 as i64),
							  &_mm_setzero_si128()) {
			self.all_clear = true;
		}

		//相殺
		let left = self.ojama.offset((chain_score / 70) as usize);

		if left != 0 {
			self.try_put_ojama();
		}
//TODO:
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

		//到着予定フレームはわかってるから、相手のフレームとの差分で
		if let Some(opponent) = opponent {
			opponent.ojama.push((chain_score / ojama_rate) as usize, (self.current_frame + 5).saturating_sub(opponent.current_frame));
			//			opponent.events.push_back(( as u32, Attack, chain_score / ojama_rate))
		}

		self.create_new_puyo();
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

		let d_combi = ROTATE_DIFF[puyo_status.rotation.value() as usize];
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