use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_andnot_si128, _mm_extract_epi64, _mm_or_si128, _mm_set_epi64x, _mm_slli_epi16, _mm_slli_si128, _mm_srli_epi16, _mm_srli_si128, _mm_store_si128, _popcnt32, _popcnt64};
use std::collections::BTreeMap;
use std::mem::transmute;

use revonet::neuro::NeuralNetwork;

use env::board::{Board, WIDTH_WITH_BORDER};
use env::board_bit::BoardBit;
use env::env::DEAD_POSITION;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::{COLOR_PUYOS, PuyoKind};
use env::split_board::SplitBoard;
use crate::build_ai::AI;

use crate::debug::Debug;
use crate::evaluator::Evaluator;
use crate::ignite_key::IgniteKey;
use crate::opener_book::Template;
use crate::opponent_status::OpponentStatus;
use crate::potential::Potential;
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
	fn evaluate(&mut self, board: &Board, sim_board: &Board, chain: &u8, score: &usize, elapse_frame: &u32, debug: &mut Debug, ojama: &OjamaStatus, ojama_rate: &usize, best_potential: &Potential, opponent_status: &OpponentStatus) -> f32 {
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

			//	let mut nn_max_potential_chain = 0u8;
			//	let mut nn_potential_need = 0u8;


			//	let mut potensial = (0, 0);
			let heights = board.get_heights();

			//	AI::get_potential_chain(board, &heights, chain, &cleared_pos_flag, 1, &mut potensial);
			//	nn_max_potential_chain = potensial.0;
			//	nn_potential_need = potensial.1;

			//addedの部分のbitboardを作成しておいて、4種類すべてとand演算で1をとって、置けた数を取得
			//置いた結果のboardを使う
			//bitboard4種類比較して、それぞれでand演算したときのpopcountをすることにより差分と置いた結果のboardで判定できる

			let mut nn_added_count = best_potential.added_count;

			for puyo_type in COLOR_PUYOS {
				let same_count = BoardBit(_mm_and_si128(best_potential.diff_board.get_bits(puyo_type).0, board.get_bits(puyo_type).0)).popcnt128() as u8;
				nn_added_count -= same_count;
			}

			assert!(nn_added_count >= 0);


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
				*chain as f32,
				*score as f32,
				*elapse_frame as f32,
				nn_bump as f32,
				nn_height_sum as f32,
				nn_highest_template_score as f32,
				nn_ojama_size as f32,
				ojama.get_time_to_receive() as f32,
				(nn_ojama_size as isize - (*score / *ojama_rate) as isize) as f32,
				nn_ojama_count_in_board as f32,
				nn_added_count as f32,
				best_potential.chain as f32,
				opponent_status.board_height as f32,
				opponent_status.board_ojama_count as f32,
				opponent_status.instant_attack as f32,
				opponent_status.potential_added_count as f32,
				opponent_status.potential_chain_count as f32,
				height[1] as f32,
				height[2] as f32,
				height[3] as f32,
				height[4] as f32,
				height[5] as f32,
				height[6] as f32,
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
	unsafe fn is_split_score_pos(mask: &BoardBit) -> bool {
		let low_count = _popcnt64(_mm_extract_epi64::<0>(mask.0));
		let high_count = _popcnt64(_mm_extract_epi64::<1>(mask.0));

		if (low_count == 0 && high_count != 0)
			|| (low_count != 0 && high_count == 0) {
			true
		} else {
			false
		}
	}
}
