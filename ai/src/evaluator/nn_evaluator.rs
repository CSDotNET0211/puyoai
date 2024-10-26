use std::arch::x86_64::{_mm_andnot_si128, _mm_or_si128, _mm_set_epi64x, _mm_slli_epi16, _mm_slli_si128, _mm_srli_epi16, _mm_srli_si128};
use std::collections::BTreeMap;

use revonet::neuro::NeuralNetwork;

use env::board::{Board, COLOR_PUYOS, WIDTH_WITH_BORDER};
use env::board_bit::BoardBit;
use env::env::DEAD_POSITION;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::PuyoKind;

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
	fn evaluate(&mut self, board: &Board, score: &usize, elapse_frame: &u32, debug: &mut Debug, ojama: &OjamaStatus, ojama_rate: &usize) -> f32 {
		unsafe {
			if !board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				debug.dead = true;
				return f32::MIN;
			}
		}

		let ojama_count_in_board = unsafe { board.get_bits(PuyoKind::Ojama).popcnt128() };
//		let mut key = IgniteKey::new(PuyoKind::Empty, 0, 0, 0, 0);
		let height = unsafe { board.get_heights() };
		let virtual_ignites: BTreeMap<u32, IgniteKey> = BTreeMap::new();
		//unsafe { Self::calc_num_of_key_puyos(&board, &mut virtual_ignites, &height); }

		let mut link2 = 0;
		let mut link3 = 0;

		unsafe {
			for color_puyo in COLOR_PUYOS {
				let mask = board.get_bits(color_puyo).mask_board_12();
				Self::find_links(&mask, &mut link2, &mut link3);
			}
		}


		let mut bumpness = 0;
		let mut height_sum = 0;
		unsafe {
			let heights = board.get_heights();
			for x in 1..6 {
				bumpness += (heights[x] as i16 - heights[x + 1] as i16).abs();
			}

			for x in 1..=6 {
				height_sum += heights[x];
			}
		}


		debug.link2_count = link2;
		debug.link3_count = link3;
		//	debug.ignite_count = key.ignite_count as i32;
		//	debug.attack = key.score as i32;
		//let virtual_ignite_top3: Vec<_> = virtual_ignites.iter().rev().take(3).collect();
		//	let mut top1 = &IgniteKey::new(PuyoKind::Empty, 0, 0, 0, 0);
		//	let mut top2 = &IgniteKey::new(PuyoKind::Empty, 0, 0, 0, 0);
		//	let mut top3 = &IgniteKey::new(PuyoKind::Empty, 0, 0, 0, 0);


		/*	if virtual_ignite_top3.len() >= 1 {
				top1 = virtual_ignite_top3[0].1;
			}
			if virtual_ignite_top3.len() >= 2 {
				top2 = virtual_ignite_top3[1].1;
			}
			if virtual_ignite_top3.len() >= 3 {
				top3 = virtual_ignite_top3[2].1;
			}*/


		let mut highest_template_score = 0;
		unsafe {
			for template in &self.templates {
				let score = template.evaluate(board);
				if highest_template_score < score {
					highest_template_score = score;
				}
			}
		}

		//全消し状態はスコアとして
		//相殺した後のお邪魔と送る火力+2
		//連鎖の位置平均を算出し、前連鎖との距離の合計
		//盤面のお邪魔数+1
		//毎フレーム更新される相手の盤面情報　仮想発火の連鎖数、ありうる最大の連鎖数、
		//置いたぷよの
		let ojama_size = unsafe { ojama.get_all_ojama_size() };
//12 + 2 + 1 + 3 + 2 = 20
		let result = self.neuralnetwork.compute(&[
			link2 as f32,
			link3 as f32,
			*score as f32,
			*elapse_frame as f32,
			height[DEAD_POSITION.x as usize] as f32,
			bumpness as f32,
			height_sum as f32,
			highest_template_score as f32,
			ojama_size as f32,
			unsafe { ojama.get_time_to_receive() } as f32,
			(ojama_size as isize - (*score / *ojama_rate) as isize) as f32,
			ojama_count_in_board as f32
		]);

		result[0]
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

	/*
		///発火に必要なぷよの数と種類を列挙、[ぷよの種類、必要な数、段差変更度]
		unsafe fn calc_num_of_key_puyos(board: &Board, keys: &mut BTreeMap<u32, IgniteKey>, heights: &[u16; 8]) {
			//それぞれの列に1~3置いて可能性のある連鎖を列挙 6列*3通り
			//それぞれの列に横で2~3個置いて可能性のある連鎖を列挙 5通り+4通り
	
			//縦 TODO:数要検討
			for puyo_type in PUYOS {
				for puyo_count in 1..=2 {
					for x in 1..WIDTH_WITH_BORDER - 1 {
						let mut new_board = board.clone();
	
						//ここから上(-1)に積みあげる
						//TODO:そもそも仮想発火で13段目って危なくね？
						for _i in 0..puyo_count {
							if heights[x as usize] + 2 < 14 {
								new_board.put_puyo_1(x, &puyo_type);
							} else {
								continue;
							}
						}
	
						let mut chain: i32 = 0;
						let mut score = 0;
						let mut board_mask = BoardBit::default();
						loop {
							let temp_score = new_board.erase_if_needed(chain, &mut board_mask);
							if temp_score == 0 {
								break;
							}
							new_board.drop_after_erased(&board_mask);
	//TODO: elapsed time
							score += temp_score;
							chain += 1;
						}
	
						let key = IgniteKey::new(puyo_type, puyo_count, chain as u8, 0, score);
	
	
						//置き終わり、評価
						if score != 0 {
							keys.insert(score, key);
						}
					}
				}
			}
	//●●〇〇〇〇
			/*
					//横
					for puyo_type in PUYOS {
						for puyo_count in 2..4 {
							for base_x in 0..WIDTH {
								let mut new_board = board.clone();
								for puyo_x in 0..puyo_count {
									//	let mut diff_x = 0;
									let x = base_x + puyo_x;
									if x >= WIDTH {
										break;
									}
			
			
									//一番上から落とす、要検討
									Env::put_puyo(&mut new_board, x as i32, 0, puyo_type);
								}
			
			
								let mut chain: u8 = 0;
								let mut attack = 0;
								loop {
									let score = Env::erase_if_needed(&mut new_board, &chain);
									if score == 0 {
										break;
									} else {
										attack += score / 70;
										chain += 1;
									}
								}
			
								//置き終わり、評価
								keys.push(IgniteKey::new(puyo_type, puyo_count as u8, chain, 0, attack));
							}
						}
					}
				*/
		}*/


	#[inline]
	pub unsafe fn find_links(mask: &BoardBit, link2: &mut i32, link3: &mut i32) {
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
		*link3 += twos.popcnt128();
		let link_3_mask = twos.expand_1(mask);

		//link3のマスクをして、調べる。downとrightのorをマスク
		let mask2_frags = BoardBit(_mm_andnot_si128(link_3_mask.0, _mm_or_si128(down.0, right.0)));
		*link2 += BoardBit(mask2_frags.0).popcnt128();

		//	let two_down = BoardBit(_mm_slli_epi16::<1>(twos.0)) & twos;
//		let two_left = BoardBit(_mm_slli_si128::<2>(twos.0)) & twos;
	}

	//
}

