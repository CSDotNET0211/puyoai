use std::arch::x86_64::{__m128i, _mm_andnot_si128, _mm_or_si128, _mm_set_epi64x, _mm_slli_epi16, _mm_slli_si128, _mm_srli_epi16, _mm_srli_si128, _mm_store_si128, _popcnt32};
use std::collections::BTreeMap;

use revonet::neuro::NeuralNetwork;

use env::board::{Board, COLOR_PUYOS, WIDTH_WITH_BORDER};
use env::board_bit::BoardBit;
use env::env::DEAD_POSITION;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::PuyoKind;
use env::split_board::SplitBoard;

use crate::debug::Debug;
use crate::evaluator::Evaluator;
use crate::ignite_key::IgniteKey;
use crate::opener_book::Template;

static PUYOS: [PuyoKind; 4] = [PuyoKind::Blue, PuyoKind::Green, PuyoKind::Red, PuyoKind::Yellow];
/*pub static DIRECTIONS: [(i32, i32); 4] = [
	(1, 0),   // 右
	(-1, 0),  // 左
	(0, 1),   // 上
	(0, -1)   // 下
];*/

pub struct NNEvaluator<T: NeuralNetwork> {
	neuralnetwork: T,
	templates: Vec<Template>,
}


impl<T: NeuralNetwork> Evaluator for NNEvaluator<T> {
	fn evaluate(&mut self, board: &Board, sim_board: &Board, chain: &u8, score: &usize, elapse_frame: &u32, debug: &mut Debug, ojama: &OjamaStatus, ojama_rate: &usize) -> f32 {
		unsafe {
			if !sim_board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				debug.dead = true;
				return f32::MIN;
			}

			let mut erase_mask = BoardBit::default();
			board.erase_if_needed(&0, &mut erase_mask);

//TODO: ここら辺関数化して
			//マスクの情報を使って消えるラインを特定
			let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
			_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, erase_mask.0);

			let mut cleared_pos_flag = 0u8;
			for i in 1..7 {
				cleared_pos_flag |= (!(board_split_aligned.0[i] == 0) as u8) << i;
			}

			let mut nn_max_potential_chain = 0u8;
			let mut nn_potential_need = 0u8;


			let mut potensial = (0, 0);
			let heights = board.get_heights();

			Self::get_potential_chain(board, &heights, chain, &cleared_pos_flag, 1, &mut potensial);
			nn_max_potential_chain = potensial.0;
			nn_potential_need = potensial.1;

			let nn_chain = 0usize;

			let nn_ojama_count_in_board = sim_board.get_bits(PuyoKind::Ojama).popcnt128();
			let height = sim_board.get_heights();

			let mut nn_link2 = 0u32;
			let mut nn_link3 = 0u32;


			for color_puyo in COLOR_PUYOS {
				let mask = sim_board.get_bits(color_puyo).mask_board_12();
				Self::find_links(&mask, &mut nn_link2, &mut nn_link3);
			}


			let mut nn_bump = 0;
			let mut nn_height_sum = 0;

			let heights = sim_board.get_heights();
			for x in 1..6 {
				nn_bump += (heights[x] as i16 - heights[x + 1] as i16).abs();
			}

			for x in 1..=6 {
				nn_height_sum += heights[x];
			}


			debug.link2_count = nn_link2;
			debug.link3_count = nn_link3;

			let mut nn_highest_template_score = 0;

			for template in &self.templates {
				let score = template.evaluate(sim_board);
				if nn_highest_template_score < score {
					nn_highest_template_score = score;
				}
			}

			//全消し状態はスコアとして
			//相殺した後のお邪魔と送る火力+2
			//連鎖の位置平均を算出し、前連鎖との距離の合計
			//盤面のお邪魔数+1
			//毎フレーム更新される相手の盤面情報　仮想発火の連鎖数、ありうる最大の連鎖数、
			//置いたぷよの
			let nn_ojama_size = unsafe { ojama.get_all_ojama_size() };
//12 + 2 + 1 + 3 + 2 = 20
			let result = self.neuralnetwork.compute(&[
				nn_link2 as f32,
				nn_link3 as f32,
				nn_chain as f32,
				*score as f32,
				*elapse_frame as f32,
				height[DEAD_POSITION.x as usize] as f32,
				nn_bump as f32,
				nn_height_sum as f32,
				nn_highest_template_score as f32,
				nn_ojama_size as f32,
				ojama.get_time_to_receive() as f32,
				(nn_ojama_size as isize - (*score / *ojama_rate) as isize) as f32,
				nn_ojama_count_in_board as f32,
				nn_potential_need as f32,
				nn_max_potential_chain as f32
			]);

			result[0]
		}
	}

	fn clone(&self) -> Self {
		NNEvaluator {
			templates: self.templates.clone(),
			neuralnetwork: self.neuralnetwork.clone(),
		}
	}
}

impl<T: NeuralNetwork> NNEvaluator<T> {
	pub unsafe fn new(network: T) -> Self {
		let mut templates = Vec::new();
		templates.push(Template(Box::new([
			_mm_set_epi64x(8590589956, 0),
			_mm_set_epi64x(51539869696, 0),
			_mm_set_epi64x(10, 1125917086711808),
		])));

		NNEvaluator {
			neuralnetwork: network,
			templates,
		}
	}

	#[inline]
	pub unsafe fn find_links(mask: &BoardBit, link2: &mut u32, link3: &mut u32) {
		//連結を2つもってる→3連結、周りのマスクも作成
		//右との連結、下との連結を取得し、3連結とのnot-and


		let right = BoardBit(_mm_srli_si128::<2>(mask.0)) & *mask;
		let left = BoardBit(_mm_slli_si128::<2>(mask.0)) & *mask;
		let up = BoardBit(_mm_srli_epi16::<1>(mask.0)) & *mask;
		let down = BoardBit(_mm_slli_epi16::<1>(mask.0)) & *mask;

		let up_down_and = up & down;
		let left_right_and = left & right;
		let up_down_or = up | down;
		let left_right_or = left | right;

		let twos = up_down_and | left_right_and | (up_down_or & left_right_or);
		*link3 += twos.popcnt128() as u32;
		let link_3_mask = twos.expand_1(mask);

		//link3のマスクをして、調べる。downとrightのorをマスク
		let mask2_frags = BoardBit(_mm_andnot_si128(link_3_mask.0, _mm_or_si128(down.0, right.0)));
		*link2 += BoardBit(mask2_frags.0).popcnt128() as u32;

		//	let two_down = BoardBit(_mm_slli_epi16::<1>(twos.0)) & twos;
//		let two_left = BoardBit(_mm_slli_si128::<2>(twos.0)) & twos;
	}

	#[inline]
	pub unsafe fn get_potential_chain(board: &Board, heights: &[u16; 8], current_chain: &u8, cleared_pos_flag: &u8, added_count: u8, best_potential: &mut (u8, u8)) {
		//最後の連鎖のx情報を使って連鎖を実行、cleared_pos_flagが0なら
		//連鎖数を見てチェック


		//forで順番に仮想落下をして、合計の連鎖数が元よりも大きくなった場合は再帰

		//clear_pos_flagの場所にぷよを落下させる、本当は隣接の色が良いかもしれんけど、とりあえず4色
		//フラグが立ってるx一覧を取得
		'pos_x: for x in 1..=6u8 {
			if ((*cleared_pos_flag >> x) & 1) != 0 {
				'puyo: for puyo in PUYOS {
					//置いて連鎖実行した結果、置く前の連鎖と比べて連鎖が伸びたら
					///最後の連鎖のx情報
					let mut test_board = board.clone();
					let mut test_heights = heights.clone();

					if test_heights[x as usize] < 13 {
						//	test_board.put_puyo_1(x, &puyo);
						test_board.put_puyo_direct(&x, &mut test_heights, &puyo);
					} else {
						continue 'pos_x;
					}

					let mut test_chain = 0;
					let mut test_cleared_pos_flag = 0;

				//	let a = test_board.to_str();
					/*	dbg!(x);
						dbg!(test_heights[x as usize]);
						dbg!(puyo);
						dbg!(a);*/

					Self::simulate(&test_board, &mut test_chain, &mut test_cleared_pos_flag);

					if best_potential.0 < test_chain {
						*best_potential = (test_chain, added_count);
					}

					if *current_chain < test_chain {
						//	assert_eq!(test_cleared_pos_flag, u8::MAX);
						Self::get_potential_chain(&test_board, &test_heights, &test_chain, &test_cleared_pos_flag, added_count + 1, best_potential);
					} else {
						continue 'puyo;
					}
				}
			}
		}
	}

	///連鎖のシミュレートを実行、最後のx情報と連鎖数を取得
	#[inline]
	unsafe fn simulate(board: &Board, chain: &mut u8, x_pos_flag: &mut u8) {
		*chain = 0;
		let mut test_board = board.clone();

		loop {
			let mut erase_mask = BoardBit::default();

			let temp_score = test_board.erase_if_needed(&chain, &mut erase_mask);
			if temp_score == 0 {
				break;
			}


			*x_pos_flag = 0u8;
			//マスクの情報を使って消えるラインを特定
			let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
			_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, erase_mask.0);

			for i in 1..=6 {
				*x_pos_flag |= (!(board_split_aligned.0[i] == 0) as u8) << i;
			}

			test_board.drop_after_erased(&erase_mask);


			*chain += 1;
		}
	}
}

#[cfg(test)]
mod tests {
	use std::fs;
	use revonet::neuro::MultilayeredNetwork;
	use super::*;

	#[test]
	fn potential_chain_test() {
		let board =
			"WWWWWWWW\
		 WYYYBGGW\
		 WBBBGRRW\
		 WEEEBGRW\
		 WEEEYYYW\
		 WEEEBGGW\
		 WEEEBBGW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW";


		unsafe {
			let board = Board::from_str(&board);

			let mut erase_mask = BoardBit::default();
			let temp_score = board.erase_if_needed(&0, &mut erase_mask);

			if temp_score == 0 { //panic!();
			}

			let mut board_split_aligned: SplitBoard = SplitBoard([0; 8]);
			_mm_store_si128(board_split_aligned.0.as_mut_ptr() as *mut __m128i, erase_mask.0);

			let mut cleared_pos_flag = 0u8;
			for i in 1..7 {
				cleared_pos_flag |= (!(board_split_aligned.0[i] == 0) as u8) << i;
			}

			let mut chain = 0;
			let mut board_clone = board.clone();
			let mut erase_mask = BoardBit::default();
			loop {
				let temp_score = board_clone.erase_if_needed(&chain, &mut erase_mask);
				if temp_score == 0 {
					break;
				}

				board_clone.drop_after_erased(&erase_mask);

				chain += 1;
			}


			let heights = board.get_heights();
			let mut potensial = (0, 0);

			NNEvaluator::<MultilayeredNetwork>::get_potential_chain(&board, &heights, &chain, &cleared_pos_flag, 1, &mut potensial);
			dbg!(potensial);
		}
	}
}