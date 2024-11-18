use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_andnot_si128, _mm_cvtsi128_si64, _mm_extract_epi16, _mm_extract_epi64, _mm_set_epi64x, _mm_setzero_si128, _mm_store_si128, _mm_xor_si128, _popcnt64};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use rand::rngs::ThreadRng;
use rand::thread_rng;
use revonet::neuro::MultilayeredNetwork;
use serde::__private::de::Content::U8;

use env::board::Board;
use env::board_bit::BoardBit;
use env::env::{ALL_CLEAR_BONUS, DEAD_POSITION, Env, FrameNeeded, SPAWN_POS};
use env::ojama_status::OjamaStatus;
use env::puyo_kind::{COLOR_PUYOS, PuyoKind};
use env::puyo_status::PuyoStatus;
use env::rotation::Rotation;
use env::split_board::SplitBoard;
use env::vector2::Vector2;

use crate::ai_move::AIMove;
use crate::debug::Debug;
use crate::evaluator::Evaluator;
use crate::evaluator::nn_evaluator::NNEvaluator;
use crate::key_type::KeyType;
use crate::key_type::KeyType::{Drop, Left, Right, Rotate180, RotateLeft, RotateRight};
use crate::opponent_status::OpponentStatus;
use crate::path::Path;
use crate::potential::Potential;

pub const POTENTIAL_SEARCH_DEPTH: usize = 4;

//盤面とネクスト、カレントを渡す
//凝視情報？相手のそのまま？
pub struct AI<E: Evaluator> {
	pub evaluator: E,
	pub eval: Option<f32>,
	pub best_move: Option<AIMove>,
	pub debug: Option<Debug>,
}

impl<E: Evaluator> AI<E> {
	pub fn new(evaluator: E) -> Self {
		AI {
			best_move: None,
			eval: None,
			evaluator,
			debug: None,
		}
	}

	pub fn clone(&self) -> Self {
		AI {
			best_move: self.best_move.clone(),
			eval: self.eval,
			evaluator: self.evaluator.clone(),
			debug: Option::from(self.debug.clone()),
		}
	}

	pub unsafe fn search(&mut self, board: &Board, current: &PuyoStatus, next: &Vec<PuyoKind>, ojama: &OjamaStatus, center_puyo: PuyoKind, movable_puyo: PuyoKind, all_cleared: bool, ojama_rate: &usize, opponent_status: &OpponentStatus) {
		//let debug = board.get_not_empty_board();
		self.best_move = Option::from(AIMove::new(-999., vec![Drop]));
		//self.best_move = None;
		self.debug = None;

		let mut rng = thread_rng();

		let instant_attack_count = AI::<NNEvaluator<MultilayeredNetwork>>::get_instant_attack(&board,&ojama_rate);


		self.search_internal(&board, &current, &next, ojama, center_puyo, movable_puyo, &Vec::new(), 0, 0, all_cleared, ojama_rate, &mut rng, opponent_status, 0,&instant_attack_count);


		if let Some(pos) = self.best_move.as_mut().unwrap().path.iter().position(|&x| x == Drop) {
			let mut new = self.best_move.clone().unwrap().path;
			new.truncate(pos + 1);
			self.best_move.as_mut().unwrap().path = new;
		}


		//	let mut debug = Debug::new();
		/*		debug.pos = potential.added_pos;
				debug.added = potential.added_count as usize;
				debug.empty = potential.empty_around_count as usize;*/
		//self.debug = Option::from(debug);
	}

	///1,2つの同色ぷよを全68パターン仮想落下し、潜在的連鎖情報を取得
	pub unsafe fn get_potential_chain_all(board: &Board) -> Potential {
		let mut best_potential = Potential::default();
		let mut best_potential_count = 0;

		let mut eval = |board: &Board, put_board: &Board, heights: &[u16; 8], added_count: &u8, ignite_pos: Vector2| {
			let mut potential = Potential::default();
			let mut chain = 0;
			let mut cleared_pos_flag = 0;

			Self::simulate(put_board, &mut chain, &mut cleared_pos_flag);


			Self::get_potential_chain(&put_board, &heights, &chain, &cleared_pos_flag, 1 + *added_count, &mut potential, 0);
			if best_potential.chain < potential.chain {
				best_potential = Potential::new(potential.chain, potential.added_count, put_board.clone(), u8::MAX, ignite_pos);
				best_potential_count = 1;
			} else if best_potential.chain == potential.chain {
				if best_potential.added_count < potential.added_count {
					best_potential = Potential::new(potential.chain, potential.added_count, put_board.clone(), u8::MAX, ignite_pos);
					best_potential_count = 1;
				} else if best_potential.added_count == potential.added_count {
					best_potential_count += 1;
				}
			}
		};


		//ぷよの種類
		for puyo_type in COLOR_PUYOS {
			//縦
			'put: for x in 1..=6u8 {
				//落下するぷよの数
				for puyo_count in 1..=2 {
					let mut board_clone = board.clone();
					let mut heights = board.get_heights();

					for _ in 0..puyo_count {
						if heights[x as usize] > 12 {
							continue 'put;
						}
						board_clone.put_puyo_direct(&x, &mut heights, &puyo_type);
					}


					eval(board, &board_clone, &heights, &puyo_count, Vector2::new(x as i8, heights[x as usize] as i8));
				}
			}
		}

		//横2
		for puyo_type in COLOR_PUYOS {
			//縦
			'put: for x in 1..6u8 {
				let mut board_clone = board.clone();
				let mut heights = board.get_heights();
				//落下するぷよの数
				for x_diff in 0..2 {
					if heights[x as usize] > 12 {
						continue 'put;
					}
					board_clone.put_puyo_direct(&(x + x_diff), &mut heights, &puyo_type);
				}
				eval(board, &board_clone, &heights, &2, Vector2::new(x as i8, heights[x as usize] as i8));
			}
		}

		best_potential
	}
	#[inline]
	///仮想落下で、
	pub unsafe fn get_instant_attack(board: &Board,ojama_rate:&usize) -> u8 {
		let mut instant_attack_count = 0;

		let mut eval = |board: &Board| {
			let mut chain = 0;
			let mut score = 0usize;
			let mut elapsed_frame = 0usize;

			chain = 0;
			let mut test_board = board.clone();

			loop {
				let mut erase_mask = BoardBit::default();

				let temp_score = test_board.erase_if_needed(&chain, &mut erase_mask, &mut 0);
				if temp_score == 0 {
					break;
				}

				elapsed_frame += FrameNeeded::VANISH_PUYO_ANIMATION;
				let drop_count = test_board.drop_after_erased(&erase_mask);
				if drop_count > 0 {
					elapsed_frame += drop_count as usize * FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK;
					elapsed_frame += FrameNeeded::LAND_PUYO_ANIMATION;
				}

				chain += 1;
				score += temp_score as usize;
			}

			if elapsed_frame <= 210 && (score / ojama_rate) >= 12 {
				instant_attack_count += 1;
			}
		};


		//ぷよの種類
		for puyo_type in COLOR_PUYOS {
			//縦
			'put: for x in 1..=6u8 {
				//落下するぷよの数
				//	for puyo_count in 1..=2 {
				let mut board_clone = board.clone();
				let mut heights = board.get_heights();

				for _ in 0..2 {
					if heights[x as usize] > 12 {
						continue 'put;
					}
					board_clone.put_puyo_direct(&x, &mut heights, &puyo_type);
				}


				eval(board);
				//	}
			}
		}

		instant_attack_count
	}

	unsafe fn search_internal(&mut self,
							  board: &Board,
							  current: &PuyoStatus,
							  next: &Vec<PuyoKind>,
							  ojama: &OjamaStatus,
							  center_puyo: PuyoKind,
							  movable_puyo: PuyoKind,
							  movements: &Vec<KeyType>,
							  mut elapsed_frame: usize,
							  score: usize,
							  mut all_cleared: bool,
							  ojama_rate: &usize,
							  rng: &mut ThreadRng,
							  opponent_status: &OpponentStatus,
							  mut waste_chain_link: usize,
							  instant_attack_count:&u8
	) {
		let mut places: HashMap<u16, (u8, PuyoStatus)> = HashMap::new();
		let mut hash_position = HashMap::new();
		Self::get_put_places(&board, &current, &mut hash_position, 0, &mut places, &(center_puyo as u8), &(movable_puyo as u8));

		for place in places {
			///操作ミノを適用しただけの盤面
			let mut put_board = board.clone();
			let mut put_place = Vector2::default();
			put_board.put_puyo(&place.1.1, &center_puyo, &movable_puyo, &mut put_place);
			///連鎖、落下のシミュレーションを実行した盤面
			let mut sim_board = put_board.clone();
			let mut ojama_clone = ojama.clone();
			if ojama_clone.get_receivable_ojama_size() != 0 {
				sim_board.try_put_ojama(&mut ojama_clone);
			}

			//	let mut waste_chain_link = 0;

			let mut new_score = score;
			let mut chain = 0u8;
			let mut chain_one_side = 0u8;
			let mut cleared_pos_flag = 0;

			let mut erase_mask = BoardBit::default();
			loop {
				let temp_score = sim_board.erase_if_needed(&chain, &mut erase_mask, &mut waste_chain_link);
				if temp_score == 0 {
					break;
				}

				let board_popcount = erase_mask.popcnt128();
				let left_popcount = _popcnt64(_mm_extract_epi64::<0>(erase_mask.0));
				let right_popcount = _popcnt64(_mm_extract_epi64::<1>(erase_mask.0));

				if board_popcount == left_popcount
					|| board_popcount == right_popcount {
					chain_one_side += 1;
				}

				elapsed_frame += FrameNeeded::VANISH_PUYO_ANIMATION;
				let drop_count = sim_board.drop_after_erased(&erase_mask);
				if drop_count > 0 {
					elapsed_frame += drop_count as usize * FrameNeeded::TEAR_PUYO_DROP_PER_1_BLOCK;
					elapsed_frame += FrameNeeded::LAND_PUYO_ANIMATION;
				}

				let mut new_x_pos_flag = 0u8;

				let value = _mm_extract_epi16::<1>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 1;
				let value = _mm_extract_epi16::<2>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 2;
				let value = _mm_extract_epi16::<3>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 3;
				let value = _mm_extract_epi16::<4>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 4;
				let value = _mm_extract_epi16::<5>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 5;
				let value = _mm_extract_epi16::<6>(erase_mask.0);
				new_x_pos_flag |= (!(value == 0) as u8) << 6;

				if new_x_pos_flag != 0 {
					cleared_pos_flag = new_x_pos_flag;
				}

				chain += 1;
				new_score += temp_score as usize;
			}

			if !sim_board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				continue;
			}

			if sim_board.is_same(&_mm_setzero_si128(),
								 &_mm_set_epi64x(0b1111111111111111000000000000000100000000000000010000000000000001u64 as i64,
												 0b0000000000000001000000000000000100000000000000011111111111111111u64 as i64),
								 &_mm_setzero_si128()) {
				all_cleared = true;
			}

			if all_cleared {
				new_score += ALL_CLEAR_BONUS;
				all_cleared = false;
			}


			let calculated_movement = Self::calculate_move(&hash_position, &place.1.1, current.position.x, current.position.y, current.rotation);

			//TODO: 
			elapsed_frame += calculated_movement.len() * FrameNeeded::MOVE;

			let mut new_movements = movements.clone();
			new_movements.extend(calculated_movement);


			//path
			if next.len() != 0 {
				let new_current = PuyoStatus::new(Vector2::new(SPAWN_POS.0, SPAWN_POS.1), Rotation::new(3));

				let mut new_next = next.clone();

				let new_center_puyo = new_next.pop().unwrap();
				let new_movable_puyo = new_next.pop().unwrap();


				self.search_internal(&sim_board, &new_current, &new_next, &ojama_clone, new_center_puyo, new_movable_puyo, &new_movements, 0, new_score, all_cleared, ojama_rate, rng, opponent_status, waste_chain_link,instant_attack_count);
			} else {
				let mut potential = Potential::default();
				AI::<NNEvaluator<MultilayeredNetwork>>::get_potential_chain(&put_board, &put_board.get_heights(), &chain, &cleared_pos_flag, 0, &mut potential, 0);

				/*	let mask = board.get_not_empty_board();
					let mut diff_board = Board::default();
					diff_board.0[0] = _mm_andnot_si128(mask.0, potential.diff_board.0[0]);
					diff_board.0[1] = _mm_andnot_si128(mask.0, potential.diff_board.0[1]);
					diff_board.0[2] = _mm_andnot_si128(mask.0, potential.diff_board.0[2]);
					potential.diff_board = diff_board;*/

				potential.ignite_pos = put_place;

				//置く前と置いた後の差分で置いた場所を取得
				let diff_board = _mm_xor_si128(board.get_not_empty_board().0, put_board.get_not_empty_board().0);
				//置いた場所を一回り拡張
				let neighbor_mask = BoardBit(diff_board).expand_1_without_mask();
				//置いた後のboardとand演算して置いてある場所を列挙
				let neighbor_flag = _mm_and_si128(neighbor_mask.0, put_board.get_not_empty_board().0);
				let empty_count = neighbor_mask.popcnt128() - BoardBit(neighbor_flag).popcnt128();
				potential.near_empty_count = empty_count as u8;


				let mut debug = Debug::new();
				debug.near_empty_count = potential.near_empty_count as usize;
				debug.ignite_pos = potential.ignite_pos;
				debug.waste_chain_link = waste_chain_link;
				debug.one_side_chain_count = chain_one_side as usize;
				debug.potential_added_count = potential.added_count as usize;

				let eval = self.evaluator.evaluate(&put_board, &sim_board, &potential, &chain, &new_score, &(elapsed_frame as u32), &mut debug, &ojama_clone, ojama_rate, opponent_status, &waste_chain_link, &chain_one_side,instant_attack_count);

				//highest_evalよりも評価が高かったら、計算したpath、
				if self.best_move == None || self.best_move.as_ref().unwrap().eval < eval {
					//現在の位置（最初はplace）の位置ハッシュを求め、見つかる間pathの行動を登録し続ける、元の位置は引数のやつ
					self.best_move = Option::from(AIMove::new(eval, new_movements));
					self.debug = Option::from(debug);
				}
			}
		}
	}

	pub fn calculate_move(hash_position: &HashMap<u16, Path>, puyo_status: &PuyoStatus, x: i8, y: i8, rotation: Rotation) -> Vec<KeyType> {
		//TODO: こういうvecとかなんとか
		let mut vec = vec![Drop];
		let mut new_puyo_status = puyo_status.clone();

		loop {
			if vec.len() > 40 {
				//逆計算できなくね？ 
				//dump
				panic!("どこさまよってるの");
				/*Debug::save_hashtable_as_csv(&hash_position, 0).unwrap();
				Debug::save_hashtable_as_csv(&hash_position, 90).unwrap();
				Debug::save_hashtable_as_csv(&hash_position, 270).unwrap();
				Debug::save_hashtable_as_csv(&hash_position, 180).unwrap();*/
			}

			if x == new_puyo_status.position.x && y == new_puyo_status.position.y && new_puyo_status.rotation == rotation {
				//所定の位置になったら終わり
				vec.reverse();
				return vec;
			}

			match hash_position.get(&new_puyo_status.create_hash(0, 0)) {
				Some(item) => {
					vec.push(item.key_type);
					match item.key_type {
						Right => {
							new_puyo_status.position.x -= 1;
						}
						Left => {
							new_puyo_status.position.x += 1;
						}
						KeyType::Top => {}
						KeyType::Down => {}
						KeyType::Drop => {}
						RotateRight => {
							//壁キックの代わりに事前保存を復元
							//rotation変えてもdiffのこう変わらないとだめ
							new_puyo_status.position.x = item.before_x;
							new_puyo_status.position.y = item.before_y;
							new_puyo_status.position_diff.x = item.before_x_diff;
							new_puyo_status.position_diff.y = item.before_y_diff;

							new_puyo_status.rotation.rotate_ccw();
						}
						RotateLeft => {
							new_puyo_status.position.x = item.before_x;
							new_puyo_status.position.y = item.before_y;
							new_puyo_status.position_diff.x = item.before_x_diff;
							new_puyo_status.position_diff.y = item.before_y_diff;

							new_puyo_status.rotation.rotate_cw();
						}
						Rotate180 => {
							new_puyo_status.position.x = item.before_x;
							new_puyo_status.position.y = item.before_y;
							new_puyo_status.position_diff.x = item.before_x_diff;
							new_puyo_status.position_diff.y = item.before_y_diff;

							new_puyo_status.rotation.rotate_180();
						}
					}
				}
				None => {
					//noneだとまずい、戻るべき位置を指定しないと
					panic!("目的地までつけないが？");
				}
			}
		}
	}

	//置きうる場所列挙、同色とかどうする？
	//盤面とcurrent
	//再帰、一度位置が決定したらソフドロのみ
	//	
	unsafe fn get_put_places(board: &Board,
							 puyo_status: &PuyoStatus,
							 mut hash_position: &mut HashMap<u16, Path>,
							 move_count: u8,
							 mut results: &mut HashMap<u16, (u8, PuyoStatus)>,
							 center_puyo: &u8,
							 movable_puyo: &u8,
	) {
		let mut key;
		//TODO: 最初に到達チェックできないかな？

		//右移動
		//さきにhash調べたほうがパフォーンス良さそう
		if Env::is_valid_position(board, &puyo_status, 1, 0) {
			key = puyo_status.create_hash(1, 0);
			match hash_position.get_mut(&key) {
				Some(value) => {
					//すでに到達済みということはこの後も全て探索済みだからパス、
					if value.move_count > move_count {
						value.key_type = Right;
						value.move_count = move_count + 1;

						value.before_x = puyo_status.position.x;
						value.before_y = puyo_status.position.y;
						value.before_x_diff = puyo_status.position_diff.x;
						value.before_y_diff = puyo_status.position_diff.y;
					}
				}
				None => {
					hash_position.insert(key, Path::new(Right, move_count + 1, puyo_status.position.x, puyo_status.position.y, puyo_status.position_diff.x as i8, puyo_status.position_diff.y as i8));

					let mut new_puyo_status = puyo_status.clone();
					Env::move_puyo(&board, &mut new_puyo_status, 1, 0);
					Self::get_put_places(&board, &new_puyo_status, &mut hash_position, move_count + 1, &mut results, center_puyo, movable_puyo);
				}
			}
		}


		//左移動
		if Env::is_valid_position(&board, &puyo_status, -1, 0) {
			key = puyo_status.create_hash(-1, 0);
			match hash_position.get_mut(&key) {
				Some(value) => {
					if value.move_count > move_count {
						if value.move_count > move_count {
							value.key_type = Left;
							value.move_count = move_count + 1;

							value.before_x = puyo_status.position.x;
							value.before_y = puyo_status.position.y;
							value.before_x_diff = puyo_status.position_diff.x;
							value.before_y_diff = puyo_status.position_diff.y;
						}
					}
				}
				None => {
					hash_position.insert(key, Path::new(Left, move_count + 1, puyo_status.position.x, puyo_status.position.y, puyo_status.position_diff.x as i8, puyo_status.position_diff.y as i8));

					let mut new_puyo_status = puyo_status.clone();
					Env::move_puyo(&board, &mut new_puyo_status, -1, 0);
					Self::get_put_places(&board, &new_puyo_status, &mut hash_position, move_count + 1, &mut results, center_puyo, movable_puyo);
				}
			}
		}


		//右回転
		let mut kick = Vector2::new(0, 0);

		if Env::is_valid_rotation(puyo_status, board, true, &mut kick) {
			let mut new_puyo_status = puyo_status.clone();
			Env::rotate_puyo(&mut new_puyo_status, 0);
			Env::move_puyo(&board, &mut new_puyo_status, kick.x, kick.y);

			let key = new_puyo_status.create_hash(0, 0);

			match hash_position.get_mut(&key) {
				Some(value) => {
					if value.move_count > move_count {
						value.key_type = RotateRight;//これいらんかも
						value.move_count = move_count + 1;

						value.before_x = puyo_status.position.x;
						value.before_y = puyo_status.position.y;
						value.before_x_diff = puyo_status.position_diff.x;
						value.before_y_diff = puyo_status.position_diff.y;
					}
				}
				None => {
					hash_position.insert(key, Path::new(RotateRight, move_count + 1, puyo_status.position.x, puyo_status.position.y, puyo_status.position_diff.x as i8, puyo_status.position_diff.y as i8));

					Self::get_put_places(&board, &new_puyo_status, &mut hash_position, move_count + 1, &mut results, center_puyo, movable_puyo);
				}
			}
		}


		//左回転
		let mut kick = Vector2::new(0, 0);

		if Env::is_valid_rotation(puyo_status, board, false, &mut kick) {
			let mut new_puyo_status = puyo_status.clone();
			Env::rotate_puyo(&mut new_puyo_status, 1);
			Env::move_puyo(&board, &mut new_puyo_status, kick.x, kick.y);

			let key = new_puyo_status.create_hash(0, 0);

			match hash_position.get_mut(&key) {
				Some(value) => {
					if value.move_count > move_count {
						value.key_type = RotateLeft;
						value.move_count = move_count + 1;

						value.before_x = puyo_status.position.x;
						value.before_y = puyo_status.position.y;
						value.before_x_diff = puyo_status.position_diff.x;
						value.before_y_diff = puyo_status.position_diff.y;
					}
				}
				None => {
					hash_position.insert(key, Path::new(RotateLeft, move_count + 1, puyo_status.position.x, puyo_status.position.y, puyo_status.position_diff.x as i8, puyo_status.position_diff.y as i8));

					Self::get_put_places(&board, &new_puyo_status, &mut hash_position, move_count + 1, &mut results, center_puyo, movable_puyo);
				}
			}
		} else if move_count == 0 {
			//180回転。回転ができなかった時に見る、初期状態飲み

			let mut new_puyo_status = puyo_status.clone();
			Env::rotate_puyo(&mut new_puyo_status, 2);
			//Env::move_puyo(&board, &mut new_puyo_status, kick.x, kick.y);
			if new_puyo_status.rotation.0 == 3 {
				new_puyo_status.position.y -= 1;
			} else if new_puyo_status.rotation.0 == 1 {
				new_puyo_status.position.y += 1;
			}

			let key = new_puyo_status.create_hash(0, 0);

			match hash_position.get_mut(&key) {
				Some(value) => {
					if value.move_count > move_count {
						value.key_type = Rotate180;
						value.move_count = move_count + 1;

						value.before_x = puyo_status.position.x;
						value.before_y = puyo_status.position.y;
						value.before_x_diff = puyo_status.position_diff.x;
						value.before_y_diff = puyo_status.position_diff.y;
					}
				}
				None => {
					hash_position.insert(key, Path::new(Rotate180, move_count + 1, puyo_status.position.x, puyo_status.position.y, puyo_status.position_diff.x as i8, puyo_status.position_diff.y as i8));

					Self::get_put_places(&board, &new_puyo_status, &mut hash_position, move_count + 1, &mut results, center_puyo, movable_puyo);
				}
			}
		}


		//ハードドロップ
		//この検索時点での最速だからおかしくなる、
		{
			let new_puyo_status = puyo_status.clone();

			let mut hash: u16 = 0;

			hash |= (*center_puyo as u16 & 0b111) << 0;
			hash |= (*movable_puyo as u16 & 0b111) << 3;
			hash |= (puyo_status.position.x as u16 & 0b111) << 6;
			hash |= ((puyo_status.position.x + puyo_status.position_diff.x) as u16 & 0b111) << 9;


			let data = results.entry(hash);

			match data {
				Entry::Occupied(mut value) => {
					let value = value.get_mut();
					if value.0 > move_count {
						value.0 = move_count;
						value.1 = new_puyo_status;
					}
				}
				Entry::Vacant(_) => {
					results.insert(hash, (move_count, new_puyo_status));
				}
			}
		}
	}

	///現在の連鎖フラグの箇所にぷよをドロップして連鎖が伸びるかを見る
	#[inline]
	pub unsafe fn get_potential_chain(board: &Board, heights: &[u16; 8], current_chain: &u8, cleared_pos_flag: &u8, added_count: u8, best_potential: &mut Potential, current_depth: usize) {
		/*if current_depth > POTENTIAL_SEARCH_DEPTH {
			return;
		}*/

		if added_count > 5 {
			return;
		}

		//最後の連鎖のx情報を使って連鎖を実行、cleared_pos_flagが0なら
		//連鎖数を見てチェック


		//forで順番に仮想落下をして、合計の連鎖数が元よりも大きくなった場合は再帰

		//clear_pos_flagの場所にぷよを落下させる、本当は隣接の色が良いかもしれんけど、とりあえず4色
		//フラグが立ってるx一覧を取得
		'pos_x: for x in 1..=6u8 {
			if ((*cleared_pos_flag >> x) & 1) == 1 {
				'puyo: for puyo in COLOR_PUYOS {
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

					Self::simulate(&test_board, &mut test_chain, &mut test_cleared_pos_flag);


					//連鎖が伸びた場合
					if *current_chain < test_chain {
						Self::get_potential_chain(&test_board, &test_heights, &test_chain, &test_cleared_pos_flag, added_count + 1, best_potential, current_depth + 1);

						//最高連鎖
						if best_potential.chain < test_chain {
							*best_potential = Potential {
								chain: test_chain,
								added_count: added_count,
								//		diff_board: test_board,
								near_empty_count: 0,
								ignite_pos: Vector2::default(),
							};
						}
					} else {
						continue 'puyo;
					}
				}
			}
		}
	}

	///連鎖のシミュレートを実行、最後のx情報と連鎖数を取得
	#[inline]
	pub unsafe fn simulate(board: &Board, chain: &mut u8, x_pos_flag: &mut u8) {
		*chain = 0;
		let mut test_board = board.clone();

		loop {
			let mut erase_mask = BoardBit::default();


			let temp_score = test_board.erase_if_needed(&chain, &mut erase_mask, &mut 0);
			if temp_score == 0 {
				break;
			}
			let mut new_x_pos_flag = 0u8;

			//*x_pos_flag = 0u8;
			//マスクの情報を使って消えるラインを特定

			let value = _mm_extract_epi16::<1>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 1;
			let value = _mm_extract_epi16::<2>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 2;
			let value = _mm_extract_epi16::<3>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 3;
			let value = _mm_extract_epi16::<4>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 4;
			let value = _mm_extract_epi16::<5>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 5;
			let value = _mm_extract_epi16::<6>(erase_mask.0);
			new_x_pos_flag |= (!(value == 0) as u8) << 6;


			if new_x_pos_flag != 0 {
				*x_pos_flag = new_x_pos_flag;
			}

			test_board.drop_after_erased(&erase_mask);


			*chain += 1;
		}
	}
}


#[cfg(test)]
mod tests {
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
		 WEEEBEEW\
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
			let temp_score = board.erase_if_needed(&0, &mut erase_mask, &mut 0);

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
				let temp_score = board_clone.erase_if_needed(&chain, &mut erase_mask, &mut 0);
				if temp_score == 0 {
					break;
				}

				board_clone.drop_after_erased(&erase_mask);

				chain += 1;
			}


			let heights = board.get_heights();
			let mut potential = Potential::default();

			AI::<NNEvaluator<MultilayeredNetwork>>::get_potential_chain(&board, &heights, &chain, &cleared_pos_flag, 1, &mut potential, 0);
			dbg!(potential.diff_board.to_str());
			dbg!(potential);
		}
	}
}