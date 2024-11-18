use std::arch::x86_64::{_mm_and_si128, _mm_andnot_si128, _mm_extract_epi64, _mm_or_si128, _mm_set_epi64x, _mm_slli_epi16, _mm_slli_si128, _mm_srli_epi16, _mm_srli_si128, _popcnt64};

use revonet::neuro::{MultilayeredNetwork, NeuralNetwork};

use env::board::Board;
use env::board_bit::BoardBit;
use env::env::DEAD_POSITION;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::{COLOR_PUYOS, PuyoKind};
use crate::build_ai::AI;

use crate::debug::Debug;
use crate::evaluator::Evaluator;
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
	fn evaluate(&mut self,
				put_board: &Board,
				sim_board: &Board,
				potential: &Potential,
				chain: &u8,
				score: &usize,
				elapse_frame: &u32,
				debug: &mut Debug,
				ojama: &OjamaStatus,
				ojama_rate: &usize,
				opponent_status: &OpponentStatus,
				waste_chain_link: &usize,
				one_side_chain_count: &u8,
				instant_attack_count: &u8,
	) -> f32 {
		unsafe {
			if !sim_board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				return f32::MIN;
			}

			//TODO: 操作中に志向ができるようになったらこっちで毎回計算
			debug.instant_attack_count = *instant_attack_count as usize;

			let mut nn_added_count = potential.added_count;

			/*	for puyo_type in COLOR_PUYOS {
					let same_count = BoardBit(_mm_and_si128(potential.diff_board.get_bits(puyo_type).0, put_board.get_bits(puyo_type).0)).popcnt128() as u8;
					nn_added_count -= same_count;
				}*/

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
				nn_link2 as f32,//2連結数
				nn_link3 as f32,//3連結数
				*chain as f32,//連鎖数
				*score as f32,//連鎖スコア
				*elapse_frame as f32,//連鎖経過時間
				nn_bump as f32,//盤面のでこぼこ
				//	nn_height_sum as f32,//盤面の高さ合計
				nn_highest_template_score as f32,//土台
				nn_ojama_size as f32,//相手からもらってるお邪魔の合計
				ojama.get_time_to_receive() as f32,//お邪魔を受けるまでの時間
				(nn_ojama_size as isize - (*score / *ojama_rate) as isize) as f32,//自分の火力で相殺できるか
				nn_ojama_count_in_board as f32,//盤面上のお邪魔数
				nn_added_count as f32,//ポテンシャル連鎖の追加数
				potential.chain as f32,//ポテンシャル連鎖の連鎖数
				opponent_status.board_height as f32,//相手の盤面の高さ合計
				opponent_status.board_ojama_count as f32,//相手の盤面のお邪魔合計
				opponent_status.instant_attack as f32,//相手の盤面の一定時間内の2列以上の火力
				opponent_status.potential_added_count as f32,//相手の盤面のポテンシャル連鎖の発火カウント
				opponent_status.potential_chain_count as f32,//相手の盤面のポテンシャル連鎖の連鎖数
				height[1] as f32,
				height[2] as f32,
				height[3] as f32,
				height[4] as f32,
				height[5] as f32,
				height[6] as f32,
				*waste_chain_link as f32,//発火した際、理論値の連鎖数*4からどれだけ離れていた（連結が多かったか）
				*one_side_chain_count as f32,//左右どちらか3列のみで連鎖した数
				potential.near_empty_count as f32,//発火点周辺の空白マス数
				potential.ignite_pos.x as f32,//発火点のx座標
				potential.ignite_pos.y as f32,//発火点のy座標
				*instant_attack_count as f32
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
