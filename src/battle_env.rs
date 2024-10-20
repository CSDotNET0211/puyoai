use std::collections::VecDeque;
use rand::{Rng, thread_rng};

use ai::build_ai::AI;
use ai::evaluator::Evaluator;
use ai::key_type::KeyType;
use env::env::Env;
use env::event_type::EventType;

pub struct BattleEnv<E: Evaluator> {
	pub player1: Env,
	player1_inputs: Vec<KeyType>,
	player1_ai: AI<E>,
	pub player2: Env,
	player2_inputs: Vec<KeyType>,
	player2_ai: AI<E>,
	pub game_frame: usize,
}

impl<E: Evaluator> BattleEnv<E> {
	pub unsafe fn new(player1_ai: AI<E>, player2_ai: AI<E>) -> Self {
		let seed = thread_rng().gen();
		let mut battle_env = BattleEnv {
			player1: Env::new(&seed),
			player2: Env::new(&seed),
			player1_ai,
			player2_ai,
			player1_inputs: Vec::new(),
			player2_inputs: Vec::new(),
			game_frame: 0,
		};

		battle_env.player1.init();
		battle_env.player2.init();

		battle_env
	}

	///死んだプレイヤーを判定する、勝ったプレイヤー番号を返す、いなければ-1
	pub fn check_winner(&self) -> i8 {
		if self.player1.dead {
			return 2;
		} else if self.player2.dead {
			return 1;
		}

		-1
	}

	pub unsafe fn update(&mut self) {
		self.game_frame += 1;
		self.player1.update();
		self.player2.update();
		//self.player1.current_frame = self.game_frame;
		//self.player2.current_frame = self.game_frame;


		Self::process_key_inputs(&mut self.player1_inputs, &mut self.player1, &mut self.player1_ai, &mut self.player2);
		Self::process_key_inputs(&mut self.player2_inputs, &mut self.player2, &mut self.player2_ai, &mut self.player1);

		/*	if Self::update_player(self.game_frame as u32, &mut self.player1.events, &mut self.player2) {
				Self::process_key_inputs(&mut self.player1_inputs, &mut self.player1, &mut self.player1_ai, &mut self.player2);
			}
			if Self::update_player(self.game_frame as u32, &mut self.player2.events, &mut self.player1) {
				Self::process_key_inputs(&mut self.player2_inputs, &mut self.player2, &mut self.player2_ai, &mut self.player1);
			}*/
	}

	///イベント処理
	fn update_player_none(current_frame: u32, player_events: &mut VecDeque<(u32, EventType, u32)>, opponent_env: &mut Env) -> bool {
		if player_events.len() != 0 &&
			player_events[0].0 <= current_frame {
			match player_events[0].1 {
				EventType::Wait => {
					panic!();
					/*player_events[0].2 -= 1;
					if player_events[0].2 == 0 {
						player_events.remove(0);
					} else {
						return false;
					}*/
				}
				EventType::Attack => {
					panic!();
					/*	opponent_env.ojama += player_events[0].2 as u32;
						player_events.remove(0);*/
				}
			}
		}

		true
	}

	//指定したプレイヤーのAI操作をします
	unsafe fn process_key_inputs(player_inputs: &mut Vec<KeyType>, env: &mut Env, ai: &mut AI<E>, opponent: &mut Env) {
		//入力予定があればそれを入力、無ければ
		if player_inputs.len() == 0 {
			let mut next = Vec::new();
			for next_p in env.next[0] {
				next.push(next_p);
			}

			ai.search(&env.board, &env.puyo_status, &next, &env.ojama, env.center_puyo, env.movable_puyo);
			*player_inputs = ai.best_move.as_ref().unwrap().path.to_vec();
		} else {
			//テトリオみたいな感じでイベント管理する
			match player_inputs.pop().unwrap() {
				KeyType::Right => { env.move_right() }
				KeyType::Left => { env.move_left() }
				KeyType::Top => { panic!() }
				KeyType::Down => { panic!() }
				KeyType::Drop => {
					env.quick_drop(Some(opponent))
				}
				KeyType::RotateRight => { env.rotate_ccw() }
				KeyType::RotateLeft => { env.rotate_cw() }
				KeyType::Rotate180 => { env.rotate_180() }
			}
		}
	}
}