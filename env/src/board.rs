use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_andnot_si128, _mm_cmpeq_epi32, _mm_cmpeq_epi64, _mm_load_si128, _mm_or_si128, _mm_set_epi64x, _mm_setzero_si128, _mm_store_si128, _mm_test_all_ones, _pext_u32, _popcnt32};
use std::mem;

use crate::board_bit::BoardBit;
use crate::puyo_kind::PuyoKind;
use crate::puyo_status::PuyoStatus;
use crate::split_board::SplitBoard;

pub const WIDTH: u8 = 6;
pub const WIDTH_WITH_BORDER: u8 = 8;

pub const HEIGHT: u8 = 14;
pub const HEIGHT_WITH_BORDER: u8 = 16;
pub const COLOR_PUYOS: [PuyoKind; 4] = [PuyoKind::Red, PuyoKind::Yellow, PuyoKind::Blue, PuyoKind::Green];

pub struct Board(pub(crate) [__m128i; 3]);


impl Board {
	#[inline]
	pub unsafe fn new() -> Board {
		Board {
			0: [_mm_setzero_si128(), _mm_set_epi64x(0b1111111111111111000000000000000100000000000000010000000000000001u64 as i64, 0b0000000000000001000000000000000100000000000000011111111111111111u64 as i64), _mm_setzero_si128()],

		}
	}
	#[inline]
	pub fn clone(&self) -> Self {
		unsafe {
			let mut new_data = [mem::zeroed(); 3];
			for i in 0..3 {
				new_data[i] = _mm_load_si128(&self.0[i] as *const __m128i);
			}
			Board(new_data)
		}
	}
	///落ちた量
	#[inline]
	pub unsafe fn put_puyo(&mut self, puyo_status: &PuyoStatus, center: &PuyoKind, movable: &PuyoKind) -> u8 {
		let mut drop_count: u8;

		//TODO: get_heights関数使おう
		let board = _mm_or_si128(self.0[0], _mm_or_si128(self.0[1], self.0[2]));
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, board);


		let puyo_center_x = puyo_status.position.x;
		let puyo_center_y = puyo_status.position.y;
		let puyo_movable_x = puyo_status.position.x + puyo_status.position_diff.x;
		let puyo_movable_y = puyo_status.position.y + puyo_status.position_diff.y;

		//yが1の時board_filled_countが0で対応
		if puyo_center_y > puyo_movable_y {
			let board_filled_count = _popcnt32(board_split_aligned.0[puyo_movable_x as usize] as i32);
			self.set_flag(puyo_movable_x, board_filled_count as i8, movable);
			board_split_aligned.0[puyo_movable_x as usize] |= 1 << (board_filled_count);

			let move_drop_count = puyo_movable_y - board_filled_count as i8;
			drop_count = move_drop_count as u8;
			//---//

			let board_filled_count = _popcnt32(board_split_aligned.0[puyo_center_x as usize] as i32);
			self.set_flag(puyo_center_x, board_filled_count as i8, center);

			let center_drop_count = puyo_center_y - board_filled_count as i8;
			if drop_count < center_drop_count as u8 {
				drop_count = center_drop_count as u8;
			}

			drop_count
		} else {
			let board_filled_count = _popcnt32(board_split_aligned.0[puyo_center_x as usize] as i32);
			self.set_flag(puyo_center_x, board_filled_count as i8, center);
			board_split_aligned.0[puyo_center_x as usize] |= 1 << (board_filled_count);

			let move_drop_count = puyo_center_y - board_filled_count as i8;
			drop_count = move_drop_count as u8;
			//---//

			let board_filled_count = _popcnt32(board_split_aligned.0[puyo_movable_x as usize] as i32);
			self.set_flag(puyo_movable_x, board_filled_count as i8, movable);

			let center_drop_count = puyo_movable_y - board_filled_count as i8;
			if drop_count < center_drop_count as u8 {
				drop_count = center_drop_count as u8;
			}

			drop_count
		}
	}

	//TODO: あふれ出ちゃう
	#[inline]
	pub unsafe fn put_puyo_1(&mut self, x: u8, puyo: &PuyoKind) {
		let board = _mm_or_si128(self.0[0], _mm_or_si128(self.0[1], self.0[2]));
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, board);

		let board_filled_count = _popcnt32(board_split_aligned.0[x as usize] as i32);
		self.set_flag(x as i8, board_filled_count as i8, puyo);
	}

	#[inline]
	pub unsafe fn get_heights(&self) -> [u16; 8] {
		let mut heights: [u16; 8] = [0; 8];
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, self.get_not_empty_board().0);

		for i in 0..8 {
			heights[i] = _popcnt32(board_split_aligned.0[i] as i32) as u16;
		}

		heights
	}

	/*/// heightsはそっちで用意して
	#[inline]
	pub unsafe fn get_heights2(&self, heights: &[u16; 8]) {
		let mut heights: [u16; 8] = [0; 8];
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, self.get_not_empty_board().0);

		for i in 0..8 {
			heights[i] = _popcnt32(board_split_aligned.0[i] as i32) as u16;
		}

		heights
	}*/

	#[inline]
	pub unsafe fn set_flag(&mut self, x: i8, y: i8, puyo_kind: &PuyoKind) {
		match puyo_kind {
			PuyoKind::Yellow => {
				BoardBit::set_bit_true(&mut self.0[0], x, y);
				BoardBit::set_bit_true(&mut self.0[1], x, y);
				BoardBit::set_bit_true(&mut self.0[2], x, y);
			}
			PuyoKind::Green => {
				BoardBit::set_bit_true(&mut self.0[0], x, y);
				BoardBit::set_bit_false(&mut self.0[1], x, y);
				BoardBit::set_bit_true(&mut self.0[2], x, y);
			}
			PuyoKind::Red => {
				BoardBit::set_bit_false(&mut self.0[0], x, y);
				BoardBit::set_bit_false(&mut self.0[1], x, y);
				BoardBit::set_bit_true(&mut self.0[2], x, y);
			}
			PuyoKind::Blue => {
				BoardBit::set_bit_false(&mut self.0[0], x, y);
				BoardBit::set_bit_true(&mut self.0[1], x, y);
				BoardBit::set_bit_true(&mut self.0[2], x, y);
			}
			PuyoKind::Ojama => {
				BoardBit::set_bit_true(&mut self.0[0], x, y);
				BoardBit::set_bit_false(&mut self.0[1], x, y);
				BoardBit::set_bit_false(&mut self.0[2], x, y);
			}
			PuyoKind::Wall => {
				BoardBit::set_bit_false(&mut self.0[0], x, y);
				BoardBit::set_bit_true(&mut self.0[1], x, y);
				BoardBit::set_bit_false(&mut self.0[2], x, y);
			}
			PuyoKind::Empty => {
				BoardBit::set_bit_false(&mut self.0[0], x, y);
				BoardBit::set_bit_false(&mut self.0[1], x, y);
				BoardBit::set_bit_false(&mut self.0[2], x, y);
			}
			_ => panic!()
		}
	}
	#[inline]
	pub unsafe fn from_str(str: &str) -> Board {
		let mut board = Board {
			0: [_mm_setzero_si128(); 3]
		};

		let mut counter = 0;
		//let mut chars = str.chars();
		for x in 0..8 {
			for y in 0..16 {
				match str.chars().nth(y * 8 + x).unwrap() {
					'Y' => {
						board.0[0] = BoardBit::set_bit(board.0[0], counter);
						board.0[1] = BoardBit::set_bit(board.0[1], counter);
						board.0[2] = BoardBit::set_bit(board.0[2], counter);
					}
					'G' => {
						board.0[0] = BoardBit::set_bit(board.0[0], counter);
						//	BoardBit::set_bit(board.0[1], counter);
						board.0[2] = BoardBit::set_bit(board.0[2], counter);
					}
					'R' => {
						//			BoardBit::set_bit(board.0[0], counter);
						//			BoardBit::set_bit(board.0[1], counter);
						board.0[2] = BoardBit::set_bit(board.0[2], counter);
					}
					'B' => {
						//	BoardBit::set_bit(board.0[0], counter);
						board.0[1] = BoardBit::set_bit(board.0[1], counter);
						board.0[2] = BoardBit::set_bit(board.0[2], counter);
					}
					'O' => {
						board.0[0] = BoardBit::set_bit(board.0[0], counter);
						//	BoardBit::set_bit(board.0[1], counter);
						//	BoardBit::set_bit(board.0[2], counter);
					}
					'W' => {
						//	BoardBit::set_bit(board.0[0], counter);
						board.0[1] = BoardBit::set_bit(board.0[1], counter);
						//	BoardBit::set_bit(board.0[2], counter);
					}
					'E' => {}
					_ => panic!()
				}
				counter += 1;
			}
		}

		board
	}

	#[inline]
	pub unsafe fn is_same(&self, v1: &__m128i, v2: &__m128i, v3: &__m128i) -> bool {
		//TODO: ガチで負荷削減するなら配列使う?
		if _mm_test_all_ones(_mm_cmpeq_epi32(self.0[0], *v1)) == 1 &&
			_mm_test_all_ones(_mm_cmpeq_epi32(self.0[1], *v2)) == 1 &&
			_mm_test_all_ones(_mm_cmpeq_epi32(self.0[2], *v3)) == 1 {
			return true;
		};
		false
	}

	// 指定したbitの値を取得する関数
	#[inline]
	fn get_bit_from_m128i(value: __m128i, bit_position: usize) -> u8 {
		// __m128iは128ビットなので、bit_positionは0〜127である必要があります
		assert!(bit_position < 128);

		// 128ビット全体をu128として取り出す
		let bytes: [u8; 16] = unsafe { std::mem::transmute(value) };
		let u128_value = u128::from_le_bytes(bytes);

		// 指定されたbitの位置の値を取得
		((u128_value >> bit_position) & 1) as u8
	}
	#[inline]
	pub unsafe fn to_str(&self) -> String {
		let mut board = String::new();

		for y in 0..16 {
			for x in 0..8 {
				let v1 = Self::get_bit_from_m128i(self.0[0], x * 16 + y);
				let v2 = Self::get_bit_from_m128i(self.0[1], x * 16 + y);
				let v3 = Self::get_bit_from_m128i(self.0[2], x * 16 + y);
				let value = (v3 << 2) | (v2 << 1) | v1;

				board += PuyoKind::from_bits(value).to_string();
			}
			board += "\r\n";
		}

		board
	}

	///ojama,red,green,blue,yellow,(preserved)
	#[inline]
	pub unsafe fn get_not_empty_board(&self) -> BoardBit {
		let v0 = BoardBit(self.0[0]);
		let v1 = BoardBit(self.0[1]);
		let v2 = BoardBit(self.0[2]);

		v0 | v1 | v2
	}

	
	#[inline]
	pub unsafe fn is_empty_cell(&self, x: i16, y: i16) -> bool {
		self.get_bits(PuyoKind::Empty).get_1_flag((x * HEIGHT_WITH_BORDER as i16 + y) as i8)
	}
	#[inline]
	pub unsafe fn erase_if_needed(&self, count_chain: i32, erased_flag: &mut BoardBit) -> u32 {
		erased_flag.0 = _mm_setzero_si128();

		let mut color_count = 0;
		let mut erased_puyo_count = 0;
		let mut link_bonus = 0;

		for color_puyo in COLOR_PUYOS {
			let mask = self.get_bits(color_puyo).mask_board_12();

			let mut erasing_bit = BoardBit::default();

			if !mask.find_erasing_flag(&mut erasing_bit) {
				continue;
			}

			color_count += 1;
			erased_flag.set_all(&erasing_bit);

			let pop_count = erasing_bit.popcnt128();
			erased_puyo_count += pop_count;

			if pop_count <= 7 {
				link_bonus += Self::get_link_bonus(&pop_count);
				continue;
			}

			erasing_bit.iterate_bit_with_masking(|x: BoardBit| -> BoardBit{
				let expanded = x.expand(mask);
				let count = expanded.popcnt128();
				link_bonus += Self::get_link_bonus(&count);
				return expanded;
			});
		}

		if color_count == 0 {
			return 0;
		}

		let color_bonus = Self::get_color_bonus(&color_count);
		let chain_bonus = Self::get_chain_bonus(
			&(count_chain as u8 + if color_count == 0 { 0 } else { 1 }));

		let ojama_erased = erased_flag.expand_edge().mask(&self.get_bits(PuyoKind::Ojama)/*.mask_board_12()*/);
		erased_flag.set_all(&ojama_erased);

		let mut bonus = color_bonus as i32 + chain_bonus as i32 + link_bonus;
		if bonus == 0 {
			bonus = 1;
		}

		return (10 * erased_puyo_count * bonus) as u32;
	}
	#[inline]
	unsafe fn pop(board: &u16, mask: &u16) -> u32 {
		_pext_u32((*board) as u32, (*mask) as u32)
	}
	///落ちる量
	#[inline]
	pub unsafe fn drop_after_erased(&mut self, erased: &BoardBit) -> u8 {
		let mut drop_count: u8 = 0;
		let mut mask_split_aligned: SplitBoard = SplitBoard([0; 8]);
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);

		_mm_store_si128(mask_split_aligned.0.as_mut_ptr() as *mut __m128i, erased.0);

		let dont_drop_mask = 0b1100000000000000;


		for i in 0..3 {
			_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, self.0[i]);

			//TODO:チェック
			for split_index in 0..mask_split_aligned.0.len() {
				if drop_count < _popcnt32(mask_split_aligned.0[split_index] as i32) as u8 {
					drop_count = _popcnt32(mask_split_aligned.0[split_index] as i32) as u8;
				}

				let dont_drop = board_split_aligned.0[split_index] & dont_drop_mask;
				let test_column = board_split_aligned.0[split_index] & !dont_drop_mask;
				board_split_aligned.0[split_index] = Self::pop(&test_column, &!(mask_split_aligned.0[split_index])) as u16;
				board_split_aligned.0[split_index] |= dont_drop;
			}

			self.0[i] = _mm_load_si128(board_split_aligned.0.as_ptr() as *const __m128i);
		}

		drop_count
	}
	#[inline]
	fn get_color_bonus(color_count: &u32) -> u32 {
		match color_count {
			1 => 0,
			2 => 3,
			3 => 6,
			4 => 12,
			_ => panic!("unsupported")
		}
	}
	#[inline]
	fn get_link_bonus(link_count: &i32) -> i32 {
		match link_count {
			0..=4 => 0,
			5 => 2,
			6 => 3,
			7 => 4,
			8 => 5,
			9 => 6,
			10 => 7,
			_ => 10
		}
	}
	#[inline]
	fn get_chain_bonus(chain_count: &u8) -> u32 {
		return match chain_count {
			0 => panic!("what"),
			1 => 0,
			2 => 8,
			3 => 16,
			4 => 32,
			5 => 64,
			6 => 96,
			7 => 128,
			_ => {
				//8以降
				128 + (*chain_count as u32 - 7) * 32
			}
		};
	}


	///指定したぷよのbitboardを作成
	#[inline]
	pub unsafe fn get_bits(&self, puyo_color: PuyoKind) -> BoardBit {
		let zero = _mm_setzero_si128();
		//let a = BoardBit(_mm_cmpeq_epi8(zero, zero));

		let v0 = BoardBit(self.0[0]);
		let v1 = BoardBit(self.0[1]);
		let v2 = BoardBit(self.0[2]);

		return match puyo_color {
			PuyoKind::Empty => {
				let oror = v0 | v1 | v2;
				let zeze = BoardBit(_mm_cmpeq_epi64(zero, zero));
				oror ^ zeze
			}
			PuyoKind::Ojama => BoardBit(_mm_andnot_si128(self.0[2], _mm_andnot_si128(self.0[1], self.0[0]))),
			PuyoKind::Wall => BoardBit(_mm_andnot_si128(self.0[2], _mm_andnot_si128(self.0[0], self.0[1]))),
			PuyoKind::Preserved => panic!(),
			PuyoKind::Red => BoardBit(_mm_andnot_si128(self.0[0], _mm_andnot_si128(self.0[1], self.0[2]))),
			PuyoKind::Green => BoardBit(_mm_and_si128(self.0[0], _mm_andnot_si128(self.0[1], self.0[2]))),
			PuyoKind::Blue => BoardBit(_mm_andnot_si128(self.0[0], _mm_and_si128(self.0[1], self.0[2]))),
			PuyoKind::Yellow => BoardBit(_mm_and_si128(self.0[0], _mm_and_si128(self.0[1], self.0[2])))
		};
	}
}