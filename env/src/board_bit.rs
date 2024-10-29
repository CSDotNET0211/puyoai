use std::arch::x86_64::*;
use std::ops::{BitAnd, BitOr, BitXor};
use std::sync::LazyLock;
use crate::split_board::SplitBoard;


//TODO:値間違ってる
static MASK_12: LazyLock<BoardBit> = LazyLock::new(|| unsafe {
	BoardBit(_mm_set_epi64x(-246294362775553, 2305596714850975743))
});

static MASK_13: LazyLock<BoardBit> = LazyLock::new(|| unsafe {
	BoardBit(_mm_set_epi64x(-211109453807617, 4611474908973629439))
});

#[derive(Debug, Clone, Copy)]
pub struct BoardBit(pub __m128i);

impl BoardBit {
	#[inline]
	pub unsafe fn default() -> BoardBit {
		BoardBit {
			0: _mm_setzero_si128()
		}
	}
	#[inline]
	pub unsafe fn l_shift(&self) -> BoardBit {
		BoardBit(_mm_slli_epi16::<1>(self.0))
	}
	#[inline]
	pub unsafe fn r_shift(&self) -> BoardBit {
		BoardBit(_mm_srli_epi16::<1>(self.0))
	}
	#[inline]
	pub unsafe fn d_shift(&self) -> BoardBit {
		BoardBit(_mm_slli_si128::<2>(self.0))
	}
	#[inline]
	pub unsafe fn u_shift(&self) -> BoardBit {
		BoardBit(_mm_srli_si128::<2>(self.0))
	}
	#[inline]
	///同色を表すBoardBitのフラグから、消せるぷよ（＝4つ以上つながっているぷよ群）のフラグを作る
	pub unsafe fn find_erasing_flag(&self, erasable: &mut BoardBit) -> bool {


		//上下左右のbit取得して、
		let right = _mm_and_si128(_mm_srli_si128::<2>(self.0), self.0);
		let left = _mm_and_si128(_mm_slli_si128::<2>(self.0), self.0);
		let up = _mm_and_si128(_mm_srli_epi16::<1>(self.0), self.0);
		let down = _mm_and_si128(_mm_slli_epi16::<1>(self.0), self.0);

		let up_down_and = _mm_and_si128(up, down);
		let left_right_and = _mm_and_si128(left, right);
		let up_down_or = _mm_or_si128(up, down);
		let left_right_or = _mm_or_si128(left, right);

		let threes = _mm_or_si128(_mm_and_si128(up_down_and, left_right_or), _mm_and_si128(left_right_and, up_down_or));
		let twos = _mm_or_si128(_mm_or_si128(up_down_and, left_right_and), _mm_and_si128(up_down_or, left_right_or));

		let two_down = _mm_and_si128(_mm_slli_epi16::<1>(twos), twos);
		let two_left = _mm_and_si128(_mm_slli_si128::<2>(twos), twos);

		erasable.0 = _mm_or_si128(_mm_or_si128(threes, two_down), two_left);

		if erasable.is_empty() {
			return false;
		}

		let two_up = _mm_and_si128(_mm_srli_epi16::<1>(twos), twos);
		let two_right = _mm_and_si128(_mm_srli_si128::<2>(twos), twos);

		*erasable = BoardBit(_mm_or_si128(_mm_or_si128(erasable.0, two_up), two_right)).expand_1(self);
		return true;
	}
	#[inline]
	pub unsafe fn is_empty(&self) -> bool {
		_mm_testz_si128(self.0, self.0) == 1
	}
	#[inline]
	pub fn mask(&self, mask: &BoardBit) -> BoardBit {
		*self & *mask
	}
	#[inline]
	pub unsafe fn expand(&self, mask: BoardBit) -> BoardBit {
		let mut seed = self.0;

		loop {
			let mut expanded = _mm_or_si128(_mm_slli_epi16::<1>(seed), seed);
			expanded = _mm_or_si128(_mm_srli_epi16::<1>(seed), expanded);
			expanded = _mm_or_si128(_mm_slli_si128::<2>(seed), expanded);
			expanded = _mm_or_si128(_mm_srli_si128::<2>(seed), expanded);
			expanded = _mm_and_si128(mask.0, expanded);

			if _mm_testc_si128(seed, expanded) == 1 {
				return BoardBit(expanded);
			}

			seed = expanded;
		}
	}

	#[inline]
	pub unsafe fn expand_1(&self, mask: &BoardBit) -> BoardBit {
		let v1 = self.l_shift();
		let v2 = self.r_shift();
		let v3 = self.u_shift();
		let v4 = self.d_shift();

		return ((*self | v1) | (v2 | v3) | v4) & *mask;
	}

	#[inline]
	pub unsafe fn expand_1_without_mask(&self) -> BoardBit {
		let v1 = self.l_shift();
		let v2 = self.r_shift();
		let v3 = self.u_shift();
		let v4 = self.d_shift();

		return (*self | v1) | (v2 | v3) | v4;
	}

	/*#[inline]
	pub unsafe fn expand_1_raw(&self, mask: &__m128i) -> BoardBit {
		let v1 = self.l_shift();
		let v2 = self.r_shift();
		let v3 = self.u_shift();
		let v4 = self.d_shift();

		return ((*self | v1) | (v2 | v3) | v4) & mask.0;
	}*/

	#[inline]
	pub unsafe fn expand_edge(&self) -> BoardBit {
		let m1 = self.l_shift();
		let m2 = self.r_shift();
		let m3 = self.u_shift();
		let m4 = self.d_shift();

		(m1 | m2) | (m3 | m4)
	}
	#[inline]
	pub unsafe fn mask_board_12(&self) -> BoardBit {
		*MASK_12 & *self
	}
	#[inline]
	pub unsafe fn mask_board_13(&self) -> BoardBit {
		*MASK_13 & *self
	}
	#[inline]
	pub unsafe fn horizontal_or16(&self) -> i32 {
		let mut x = _mm_or_si128(_mm_srli_si128::<8>(self.0), self.0);
		x = _mm_or_si128(_mm_srli_si128::<4>(x), x);
		x = _mm_or_si128(_mm_srli_si128::<2>(x), x);
		return _mm_cvtsi128_si32(x) & 0xFFFF;
	}
	#[inline]
	pub unsafe fn popcnt128(&self) -> i32 {
		let low = _mm_cvtsi128_si64(self.0); // 下位64ビットを抽出
		let shifted = _mm_srli_si128::<8>(self.0); // 128ビット右に8バイトシフト
		let high = _mm_cvtsi128_si64(shifted); // 上位64ビットを抽出


		_popcnt64(low) + _popcnt64(high)
	}
	#[inline]
	pub fn set_all(&mut self, fb: &BoardBit) {
		*self = *self | *fb
	}
	#[inline]
	pub unsafe fn iterate_bit_with_masking<F>(&self, mut f: F)
		where F: FnMut(BoardBit) -> BoardBit
	{
		let zero = _mm_setzero_si128();
		let down_ones = _mm_cvtsi64_si128(-1i64);
		let up_ones = _mm_slli_si128::<8>(down_ones);

		let mut current = *self;
		while !_mm_testz_si128(up_ones, current.0) == 1 {
			let y = _mm_and_si128(current.0, _mm_sub_epi64(zero, current.0));
			let z = _mm_and_si128(up_ones, y);
			let mask = f(BoardBit(z));
			current = BoardBit(_mm_andnot_si128(mask.0, current.0));
		}

		while !_mm_testz_si128(down_ones, current.0) == 1 {
			let y = _mm_and_si128(current.0, _mm_sub_epi64(zero, current.0));
			let z = _mm_and_si128(down_ones, y);
			let mask = f(BoardBit(z));
			current = BoardBit(_mm_andnot_si128(mask.0, current.0));
		}
	}
	#[inline]
	pub unsafe fn get_1_flag(&self, bit_index: i8) -> bool {
		let bit_value;
		if bit_index < 64 {
			// 下位64ビットの処理
			let lower_64 = unsafe { _mm_extract_epi64::<0>(self.0) };
			bit_value = (lower_64 >> bit_index) & 1;
		} else {
			// 上位64ビットの処理
			let upper_64 = unsafe { _mm_extract_epi64::<1>(self.0) };
			bit_value = (upper_64 >> (bit_index - 64)) & 1;
		}

		bit_value == 1
	}
	#[inline]
	pub fn set_bit(x: __m128i, bit_pos: u8) -> __m128i {
		// 指定された位置のビットを立てるマスクを作成
		let mut mask = [0u64; 2];
		if bit_pos < 64 {
			mask[0] = 1u64 << bit_pos;
		} else {
			mask[1] = 1u64 << (bit_pos - 64);
		}

		// マスクを__m128iに変換
		let mask_m128i = unsafe { _mm_set_epi64x(mask[1] as i64, mask[0] as i64) };

		// ビットを立てる
		unsafe { _mm_or_si128(x, mask_m128i) }
	}
	#[inline]
	pub unsafe fn set_bit_true(board: &mut __m128i, x: u8, y: u8) {
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, *board);
		board_split_aligned.0[x as usize] = board_split_aligned.0[x as usize] | 1 << y;
		*board = _mm_load_si128(board_split_aligned.0.as_ptr() as *const __m128i);
	}

	#[inline]
	pub unsafe fn set_bit_true_column(board_column: &mut u16, mask: &u16) {
		*board_column = *board_column | *mask;
	}

	#[inline]
	pub unsafe fn set_bit_false(board: &mut __m128i, x: u8, y: u8) {
		let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
		_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, *board);
		board_split_aligned.0[x as usize] = board_split_aligned.0[x as usize] & !(1 << y);
		*board = _mm_load_si128(board_split_aligned.0.as_ptr() as *const __m128i);
	}

	#[inline]
	pub unsafe fn set_bit_false_column(board_column: &mut u16, mask: &u16) {
		*board_column = *board_column & !*mask;
	}
}


impl BitAnd for BoardBit {
	type Output = BoardBit;

	fn bitand(self, rhs: BoardBit) -> BoardBit {
		unsafe {
			BoardBit(_mm_and_si128(self.0, rhs.0))
		}
	}
}


impl BitOr for BoardBit {
	type Output = BoardBit;

	fn bitor(self, rhs: Self) -> Self::Output {
		unsafe { BoardBit(_mm_or_si128(self.0, rhs.0)) }
	}
}

impl BitXor for BoardBit {
	type Output = BoardBit;

	fn bitxor(self, rhs: BoardBit) -> Self::Output {
		unsafe { BoardBit(_mm_xor_si128(self.0, rhs.0)) }
	}
}
/*
impl BitXor<__m128i> for BoardBit {
	type Output = __m128i;

	fn bitxor(self, rhs: __m128i) -> Self::Output {
		unsafe { _mm_xor_si128(self.0, rhs) }
	}
}

impl BitOr<__m128i> for BoardBit {
	type Output = __m128i;

	fn bitor(self, rhs: __m128i) -> Self::Output {
		unsafe { _mm_or_si128(self.0, rhs) }
	}
}

impl BitAnd<__m128i> for BoardBit {
	type Output = __m128i;

	fn bitand(self, rhs: __m128i) -> __m128i {
		unsafe {
			_mm_and_si128(self.0, rhs)
		}
	}
}
*/