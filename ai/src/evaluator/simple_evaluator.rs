use std::arch::x86_64::{_mm_andnot_si128, _mm_or_si128, _mm_slli_epi16, _mm_slli_si128, _mm_srli_epi16, _mm_srli_si128};
use env::board::{Board, COLOR_PUYOS, WIDTH_WITH_BORDER};
use env::board_bit::BoardBit;
use env::env::DEAD_POSITION;
use env::ojama_status::OjamaStatus;
use env::puyo_kind::PuyoKind;
use crate::debug::Debug;
use crate::evaluator::Evaluator;
use crate::ignite_key::IgniteKey;

static PUYOS: [PuyoKind; 4] = [PuyoKind::Blue, PuyoKind::Green, PuyoKind::Red, PuyoKind::Yellow];
/*pub static DIRECTIONS: [(i32, i32); 4] = [
	(1, 0),   // 右
	(-1, 0),  // 左
	(0, 1),   // 上
	(0, -1)   // 下
];*/

pub struct SimpleEvaluator {
	weight: [f32; 8],
}

impl Evaluator for SimpleEvaluator {
	fn evaluate(&mut self, board: &Board, sim_board: &Board, chain: &u8, score: &usize, elapse_frame: &u32, debug: &mut Debug, ojama: &OjamaStatus, ojama_rate: &usize) -> f32 {
		panic!();
		//	Console::print_board(&board);
		let mut result = 0.;
		unsafe {
			if !sim_board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				debug.dead = true;
				return f32::MIN;
			}
		}

		//今までの価値も
		// 1から10のランダムな整数を生成
		//let random_number = self.rng.gen_range(1..=10);
		//return random_number as f32;

		/*
		それぞれのAIの育成とニューラルネットワークの選択について
		連鎖構築AI	長い連鎖を作ることがメイン、本線でない連鎖の数とかも学習出来たら面白そう、GAである程度最適化し、最後に対戦で最適化
		攻撃AI		本線とか色々
		防御AI		相殺とか色々
		
		自分と相手の状態を数値化し、ニューラルネットワークでモードを選択する、一定時間どこに再選択し、戦いで重みを調整する
		 */

		/*
		連鎖数、火力、２連結、３連結の個数でも水平とか垂直とか分けてもいいかも？
		ライン上の致死までの高さ、
		発火のキーぷよ数 1から3つの同色ぷよを特定の列に置く、後横バージョン 
		本線からの独立度 本線以外の攻撃や防御に使用 保留
		連鎖後の地形変更割合 上に同じ、amaは崩しやすいからジャブに弱い 各列の消えた段で判断、全部１の場合同じだよね、 連鎖位置をもっておき、対emptyの場合は何もしない、違う場合は
		上要検討、ずれたら普通にぷよ数とか連鎖数の評価下がりそうだから割となくてもいけるかも
		地形の平均高さ、これ単体で重みを付けるべきではないかも
		猶予時間 相手が連鎖中の時どれくらいのフレームの猶予があるか、どれくらいの火力があるか
		リソース管理
		
		
		 */
		/*
		連鎖数
		発火に必要なキーぷよの数
		連鎖の拡張性
		２ダブ
		３ダブ
		盤面の形
		U型
		テンプレ
		２連結
		３連結
		お邪魔の数
		操作必要時間
		 */

		/*
		Chain.
	Ignition y.
	Number of key puyo needed to ignite.
	Chain extendibility.
	2 dub.
	3 dub.
	Field shape.
	U shape.
	Chain form (GTR, Meri, etc.)
	Link 2.
	Link 3.
	Number of garbage puyo.
	Time wasted.
		 */


		//重み最適化、一定フレーム以内にできる限り大きい連鎖を

		//決め方
		//火力数、経過フレーム、火力数2、経過フレーム2、キーぷよの数、キーぷよの所有数
		//相手の状況なども
		//let mut keys = Vec::new();
		let mut key = IgniteKey::new(PuyoKind::Empty, 0, 0, 0, 0);
		unsafe { Self::calc_num_of_key_puyos(&sim_board, &mut key); }

		//keysを使ってそれぞれの起こりうる連鎖を評価
		//選ぶのはニューラルネットワーク？	いやさすがに無理、防御ならお邪魔を減らすことの重みが強い、攻撃なら
		/*	for key in keys {
				result += key.chain as f32 * weight[0];
				result += key.attack as f32 * weight[1];
				result += key.ignite_count as f32 * weight[2];
	
				//	let count = next.iter().filter(|&&puyo| puyo == key.puyo_kind).count();
				//	result += key.ignite_count.saturating_sub(count as u8);
	
				//第2の連鎖
			}*/
		//	result += *attack as f32 * weight[3];

		//	result += count_link2 as f32 * weight[4];
		//	result += count_link3 as f32 * weight[5];

		let mut link2 = 0;
		let mut link3 = 0;

		unsafe {
			for color_puyo in COLOR_PUYOS {
				let mask = sim_board.get_bits(color_puyo)/*.mask_board_12()*/;
				Self::find_links(&mask, &mut link2, &mut link3);
			}
		}

		result += link2 as f32 * self.weight[0];
		result += link3 as f32 * self.weight[1];

		result += key.ignite_count as f32 * self.weight[2];
		result += key.score as f32 * self.weight[3];
		result += *score as f32 * self.weight[4];

		let height = unsafe { sim_board.get_heights() };
		result += height[DEAD_POSITION.x as usize] as f32 * self.weight[5];

		let mut bumpness = 0;
		let mut height_sum = 0;
		unsafe {
			let heights = sim_board.get_heights();
			for x in 1..6 {
				bumpness += (heights[x] as i16 - heights[x + 1] as i16).abs();
			}

			for x in 1..=6 {
				height_sum += heights[x];
			}
		}

		result += bumpness as f32 * self.weight[6];
		result += height_sum as f32 * self.weight[7];


//		(link2 * 2 + link3 * 5) as f32
		result
	}

	fn clone(&self) -> Self {
		SimpleEvaluator {
			weight: self.weight.clone()
		}
	}
}

impl SimpleEvaluator {
	pub fn new(weight: [f32; 8]) -> SimpleEvaluator {
		SimpleEvaluator {
			weight
		}
	}


	///発火に必要なぷよの数と種類を列挙、[ぷよの種類、必要な数、段差変更度]
	unsafe fn calc_num_of_key_puyos(board: &Board, main: &mut IgniteKey/*, keys: &mut Vec<IgniteKey>*/) {
		//それぞれの列に1~3置いて可能性のある連鎖を列挙 6列*3通り
		//それぞれの列に横で2~3個置いて可能性のある連鎖を列挙 5通り+4通り

		for puyo_type in PUYOS {
			for puyo_count in 1..=2 {
				for x in 1..WIDTH_WITH_BORDER - 1 {
					let mut new_board = board.clone();

					//ここから上(-1)に積みあげる
					for _i in 0..puyo_count {
						new_board.put_puyo_1(x, &puyo_type);
					}

					let mut chain: u8 = 0;
					let mut score = 0;
					let mut board_mask = BoardBit::default();
					loop {
						let temp_score = new_board.erase_if_needed(&chain, &mut board_mask);
						if temp_score == 0 {
							break;
						}
						new_board.drop_after_erased(&board_mask);

						score += temp_score;
						chain += 1;
					}

					if main.score < score {
						main.puyo_kind = puyo_type;
						main.chain = 0;
						main.score = score;
						main.ignite_count = puyo_count;
					}


					//置き終わり、評価
					//	keys.push(IgniteKey::new(puyo_type, puyo_count, chain as u8, 0, attack));
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
	}

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

