use std::arch::x86_64::{__m128i, _mm_and_si128};

use env::board::{Board, COLOR_PUYOS};
use env::board_bit::BoardBit;

#[derive(Debug, Clone)]
pub struct Template(pub Box<[__m128i]>);

impl Template {
	pub unsafe fn evaluate(&self, board: &Board) -> u8 {
		let mut match_score: u8 = 0;

		let mut color_count: [u8; 4] = [0; 4];
		let template = &self.0;
		
		for color_puyo in COLOR_PUYOS {
		//	let mut used_mask = _mm_setzero_si128();

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

			}
		}

		match_score
	}
}