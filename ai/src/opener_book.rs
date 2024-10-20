use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_andnot_si128, _mm_cmpeq_epi32, _mm_set_epi64x, _mm_setzero_si128, _mm_test_all_ones, _mm_testz_si128};
use crate::debug::Debug;
use env::board::{Board, COLOR_PUYOS};
use env::board_bit::BoardBit;

#[derive(Debug, Clone)]
pub struct Template(pub Box<[__m128i]>);

impl Template {
	pub unsafe fn evaluate(&self, board: &Board) -> u8 {
		let mut match_score: u8 = 0;

		let mut color_count: [u8; 4] = [0; 4];
		let template = &self.0;
		//let not_empty_board = board.get_not_empty_board();

		for color_puyo in COLOR_PUYOS {
			let mut used_mask = _mm_setzero_si128();

			for (index, test) in template.iter().enumerate() {
				let test_expand = BoardBit(*test).expand_1_without_mask().0;


				//色ぷよ抽出
				let mask = board.get_bits(color_puyo);
				//テンプレの適応度、testでくり抜く
				let extract = BoardBit(_mm_and_si128(mask.0, *test));
				let extract_expand_1 = BoardBit(_mm_and_si128(mask.0, test_expand));

				let conformity_score = extract.popcnt128() as u8;
				let conformity_score_expand_1 = extract_expand_1.popcnt128() as u8;

				//拡張した部分に含まれてる＝左右の連結に同じ色がある＝隣り合ってる
				//その色が空の場合違うことになるね、それ自身が0マッチ＝そもそも存在しないから触れてない場合は無視で
				if conformity_score != 0 && conformity_score != conformity_score_expand_1 {
					return 0;
				}

				match_score += conformity_score;
				if conformity_score != 0 {
					color_count[index] += 1;
					//	color_count += 1;
				}

				//順序変えちゃったけどこれどうやろうか
				if color_count[index] > 1 {
					//もはや作成不可能
					return 0;
				}

				/*	let sub_from_not_empty_to_extract = _mm_andnot_si128(extract.0, not_empty_board.0);
					let should_be_zero = _mm_and_si128(sub_from_not_empty_to_extract, *test);
					let a = should_be_zero.clone();
					//	if _mm_test_all_ones(_mm_cmpeq_epi32(should_be_zero, _mm_set_epi64x(0b1111111111111111000000000000000100000000000000010000000000000001u64 as i64,
					//																		0b0000000000000001000000000000000100000000000000011111111111111111u64 as i64))) == 0 {
					if _mm_test_all_ones(_mm_cmpeq_epi32(should_be_zero, _mm_setzero_si128())) == 0 {
						//もはや作成不可能
						return 0;
					} else {
						dbg!(should_be_zero);
						break;
					}*/
			}
		}

		match_score
	}
}