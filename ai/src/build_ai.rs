use std::arch::x86_64::{_mm_set_epi64x, _mm_setzero_si128};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use rand::rngs::ThreadRng;
use rand::thread_rng;

use env::board::Board;
use env::board_bit::BoardBit;
use env::env::{ALL_CLEAR_BONUS, DEAD_POSITION, Env, FrameNeeded, SPAWN_POS};
use env::ojama_status::OjamaStatus;
use env::puyo_kind::PuyoKind;
use env::puyo_status::PuyoStatus;
use env::rotation::Rotation;
use env::vector2::Vector2;

use crate::ai_move::AIMove;
use crate::debug::Debug;
use crate::evaluator::Evaluator;
use crate::key_type::KeyType;
use crate::key_type::KeyType::{Drop, Left, Right, Rotate180, RotateLeft, RotateRight};
use crate::path::Path;

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
			debug: self.debug.clone(),
		}
	}

	pub unsafe fn search(&mut self, board: &Board, current: &PuyoStatus, next: &Vec<PuyoKind>, ojama: &OjamaStatus, center_puyo: PuyoKind, movable_puyo: PuyoKind, all_cleared: bool, ojama_rate: &usize) {
		//let debug = board.get_not_empty_board();
		self.best_move = Option::from(AIMove::new(-999., vec![Drop]));
		//self.best_move = None;
		self.debug = None;

		/*if ojama.0!=0{
			panic!("test");
		}*/
		/*
		if !board.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
			return;;
		}*/

		let mut rng = thread_rng();

		self.search_internal(&board, &current, &next, ojama, center_puyo, movable_puyo, &Vec::new(), 0, 0, all_cleared, ojama_rate, &mut rng);

		if let Some(pos) = self.best_move.as_mut().unwrap().path.iter().position(|&x| x == Drop) {
			let mut new = self.best_move.clone().unwrap().path;
			new.truncate(pos + 1);
			self.best_move.as_mut().unwrap().path = new;
		}
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
	) {
		let mut places: HashMap<u16, (u8, PuyoStatus)> = HashMap::new();
		let mut hash_position = HashMap::new();
		Self::get_put_places(&board, &current, &mut hash_position, 0, &mut places, &(center_puyo as u8), &(movable_puyo as u8));

		for place in places {
			///操作ミノを適用しただけの盤面
			let mut new_board = board.clone();
			new_board.put_puyo(&place.1.1, &center_puyo, &movable_puyo);
			///連鎖、落下のシミュレーションを実行した盤面
			let mut new_board_sim = new_board.clone();
			let mut ojama_clone = ojama.clone();
			if ojama_clone.get_receivable_ojama_size() != 0 {
				new_board_sim.try_put_ojama(&mut ojama_clone, rng);
			}
			if !new_board_sim.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
				continue;
			}

			let mut new_score = score;
			let mut chain = 0u8;

			let mut erase_mask = BoardBit::default();
			loop {
				let temp_score = new_board_sim.erase_if_needed(&chain, &mut erase_mask);
				if temp_score == 0 {
					break;
				}

				new_board_sim.drop_after_erased(&erase_mask);


				chain += 1;
				new_score += temp_score as usize;
			}

			if new_board_sim.is_same(&_mm_setzero_si128(),
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

			elapsed_frame += calculated_movement.len() * FrameNeeded::MOVE;

			let mut new_movements = movements.clone();
			new_movements.extend(calculated_movement);


			//path
			if next.len() != 0 {
				let new_current = PuyoStatus::new(Vector2::new(SPAWN_POS.0, SPAWN_POS.1), Rotation::new(3));

				let mut new_next = next.clone();

				let new_center_puyo = new_next.pop().unwrap();
				let new_movable_puyo = new_next.pop().unwrap();

				//経過フレームとか生成火力を引き継ぐ
				self.search_internal(&new_board_sim, &new_current, &new_next, &ojama_clone, new_center_puyo, new_movable_puyo, &new_movements, 0, new_score, all_cleared, ojama_rate, rng);
			} else {
				let mut debug = Debug::new();
				let eval = self.evaluator.evaluate(&new_board, &new_board_sim, &chain, &new_score, &0, &mut debug, &ojama_clone, ojama_rate);

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
}
