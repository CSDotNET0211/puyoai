use std::collections::VecDeque;
use std::time::Instant;
use rand::{Rng, thread_rng};

use ai::build_ai::AI;
use ai::evaluator::Evaluator;
use ai::key_type::KeyType;
use ai::opponent_status::OpponentStatus;
use env::env::{Env, Event};
use env::event_type::EventType;
use env::puyo_kind::PuyoKind;

pub struct BattleEnv<E: Evaluator> {
	pub player1: Env,
	player1_inputs: VecDeque<KeyType>,
	player1_ai: AI<E>,
	player1_opponent_status: OpponentStatus,
	pub player2: Env,
	player2_inputs: VecDeque<KeyType>,
	player2_ai: AI<E>,
	player2_opponent_status: OpponentStatus,
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
			player1_opponent_status: OpponentStatus::default(),
			player2_opponent_status: OpponentStatus::default(),
			player1_inputs: VecDeque::new(),
			player2_inputs: VecDeque::new(),
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
		self.player1.update();
		self.player2.update();

		if self.game_frame % 10 == 0 {
			self.player1_opponent_status = OpponentStatus::new(&self.player2.board);
			self.player2_opponent_status = OpponentStatus::new(&self.player1.board);
		}

		if Self::update_player(self.game_frame, &mut self.player1.events) {
			Self::process_key_inputs(&mut self.player1_inputs, &mut self.player1, &mut self.player1_ai, &mut self.player2, &self.player1_opponent_status);
		}

		if Self::update_player(self.game_frame, &mut self.player2.events) {
			Self::process_key_inputs(&mut self.player2_inputs, &mut self.player2, &mut self.player2_ai, &mut self.player1, &self.player2_opponent_status);
		}

		self.game_frame += 1;
	}

	///イベント処理
	fn update_player(current_frame: usize, events: &mut VecDeque<Event>) -> bool {
		while events.len() != 0 {
			if events[0].frame <= current_frame {
				match events[0].kind {
					EventType::Wait => {
						if events[0].value == 0 {
							events.remove(0);
							continue;
						}

						events[0].value -= 1;
						if events[0].value == 0 {
							events.remove(0);
						} else {
							return false;
						}
					}
					_ => panic!()
				}
			} else {
				break;
			}
		}

		true
	}

	//指定したプレイヤーのAI操作をします
	unsafe fn process_key_inputs(player_inputs: &mut VecDeque<KeyType>, env: &mut Env, ai: &mut AI<E>, opponent: &mut Env, opponent_status: &OpponentStatus) {
		if env.center_puyo == PuyoKind::Empty &&
			env.movable_puyo == PuyoKind::Empty {
			env.create_new_puyo();
			return;
		}

		if player_inputs.len() == 0 {
			let mut next = Vec::new();
			for next_p in env.next[0] {
				next.push(next_p);
			}

			//	for next_p in env.next[1] {
			//		next.push(next_p);
			//	}

			let think_start = Instant::now();
			ai.search(&env.board, &env.puyo_status, &next, &env.ojama, env.center_puyo, env.movable_puyo, env.all_cleared, &env.ojama_rate, opponent_status);
			let time = think_start.elapsed().as_millis();
			//println!("{}", format!("{:04}", time));
				dbg!(think_start.elapsed().as_millis());

			*player_inputs = ai.best_move.as_ref().unwrap().path.to_vec().into();
		} else {
			match player_inputs.pop_front().unwrap() {
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