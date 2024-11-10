use std::{fs, thread};
use std::io::stdin;
use std::time::{Duration, Instant};

#[cfg(feature = "ppc")]
use ppc::scp::Controller;
use revonet::ea::{EA};
use revonet::ne::NE;
use revonet::neuro::MultilayeredNetwork;

use ai::build_ai::AI;
use ai::evaluator::nn_evaluator::NNEvaluator;
use console::console::Console;
use env::env::Env;
use env::puyo_kind::PuyoKind;

use crate::battle_env::BattleEnv;
use crate::log::Log;
use crate::log::LogType::INFO;
#[cfg(feature = "ppc")]
use crate::ppc_wrapper::PpcWrapper;
use crate::problems::battle_problem::BattleProblem;

mod log;
mod battle_env;
mod problems;

#[cfg(feature = "ppc")]
mod ppc_wrapper;
#[cfg(feature = "ppc")]
mod field;

//use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
//use rand::Rng;

static COLOR_PUYOS: [PuyoKind; 4] = [PuyoKind::Green, PuyoKind::Red, PuyoKind::Blue, PuyoKind::Yellow];


fn main() {
	unsafe {
		let mut input = String::new();
		println!("Select mode.\n\
		1.Console AI\n\
		2.Console Human\n\
		3.PPC AI\n\
		4.Training\n\
		5.Console Battle\n\
		6.Console Auto Battle");
		/*	io::stdin()
				.read_line(&mut input)
				.unwrap();
	*/
		input = "4".parse().unwrap();

		match input.trim() {
			"1" => {}
			"2" => {
				let mut env = Env::new(&0);
				env.init();

				let json_str = fs::read_to_string(r"test.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();

				let ai = AI::new(NNEvaluator::new(net));


				loop {

					//待機

					Console::print(&env, 0, true, false);
					let mut next = Vec::new();

					//	for next_p in env.next {
					for next_p2 in env.next[0] {
						next.push(next_p2);
					}
					/*	for next_p2 in env.next[0] {
							next.push(next_p2);
						}*/

					let mut input = Default::default();
					if true {
						input = "ai".parse().unwrap();
					} else {
						input = Console::get_input();
					}
				}
			}
			"3" => {
				#[cfg(feature = "ppc")]
				ppc();

				panic!("ppc module isn't loaded")
			}
			"4" => {
				println!("学習を開始します。");
				println!("途中学習データを読み込みますか。(y/n)");
				/*	let mut input = Default::default();
					stdin().read_line(&mut input).unwrap();
					match input.trim() {
						"y" => {}
						"n" => {}
						_ => panic!()
					}
				*/
				println!("ネットワークを用いた負荷分散を行いますか。(y/n)");

				println!("学習を開始します。学習過程はlog.txtに保存されます。");
				println!("各世代のbestが保存されます。");
				let mut log = Log::open("log.txt");

				log.write(INFO, "Training started");


				let setting = revonet::settings::EASettings::new(64, 999999999, 30);
				//let problem = ScoreProblem::new();
				let problem = BattleProblem::new();
				let mut ne: NE<BattleProblem> = NE::new(&problem);
				//let mut ne: NE<ScoreProblem> = NE::new(&problem);
				//let res = ne.run(setting, &false).unwrap();
				let _ = ne.run(setting, &true).unwrap();
			}
			"5" => {
				//	stdin().read_line(&mut "".to_string());
				const FRAME_DURATION: Duration = Duration::from_millis(17);
				let mut previous_time = Instant::now();

				let json_str = fs::read_to_string(r"test.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();
				let ai = AI::new(NNEvaluator::new(net));

				let json_str = fs::read_to_string(r"test2.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();
				let ai2 = AI::new(NNEvaluator::new(net));

				let mut battle = BattleEnv::new(ai.clone(), ai2.clone());

				Console::clear();
				loop {
					let start_time = Instant::now();

					battle.update();
					if battle.check_winner() != -1 {
						println!("どっちかがGAME OVER");
						stdin().read_line(&mut "".to_string()).unwrap();
					}


					previous_time = Instant::now();
					Console::print(&battle.player1, 0, true, false);
					//		println!("current_ojama:{:?}", &battle.player1.ojama.get_raw());
					Console::print(&battle.player2, 1, true, false);
					//		println!("current_ojama:{:?}", &battle.player1.ojama.get_raw());

					//		println!("current_frame:{} / time:{}", battle.game_frame, battle.game_frame / 60);
					//			println!("current_events:{:?}", battle.player1.events);


					let elapsed_time = start_time - previous_time;

					if elapsed_time < FRAME_DURATION {
						thread::sleep(FRAME_DURATION - elapsed_time);
					}
				}
			}
			"6" => {
				let json_str = fs::read_to_string(r"test.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();
				let ai = AI::new(NNEvaluator::new(net));

				let json_str = fs::read_to_string(r"test2.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();
				let ai2 = AI::new(NNEvaluator::new(net));

				let mut battle = BattleEnv::new(ai.clone(), ai2.clone());

				let first_to = 30;
				let mut player1_won = 0;
				let mut player2_won = 0;
				loop {
					battle.update();
					let result = battle.check_winner();

					if result != -1 {
						if result == 1 {
							player1_won += 1;
							println!("player1 won");
						} else if result == 2 {
							player2_won += 1;
							println!("player2 won");
						}

						if player1_won == first_to || player2_won == first_to {
							println!("player1:{player1_won} / player2:{player2_won}");

							stdin().read_line(&mut "".to_string()).unwrap();
						} else {
							battle = BattleEnv::new(ai.clone(), ai2.clone());
						}
					}
				}
			}
			_ => {}
		}
	}
}


#[cfg(feature = "ppc")]
unsafe fn ppc() {
	let json_str = fs::read_to_string(r"test.json").unwrap();
	let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();
	let mut ai = AI::new(NNEvaluator::new(net));


	let scp = Controller::new();
	thread::sleep(Duration::from_millis(2));

	let mut ppc_player1 = PpcWrapper::new(0, 0, Some(scp));
	let mut ppc_player2 = PpcWrapper::new(1, 0, None);
	ppc_player1.connect();
	ppc_player2.connect();
//	ppc.on_complete_action = Some(Box::new(test));
//	ppc.on_gamestate_changed = Some(Box::new(test2));

	//こっちでupdate回す
	loop {
		ppc_player1.update();
		//println!("{:?}",ppc_player1.field.lock().unwrap().is_movable );
		if ppc_player1.field.lock().unwrap().current.is_some() &&
			ppc_player1.field.lock().unwrap().is_movable &&
			ppc_player1.inputs.len() == 0 {


			//thread::sleep(Duration::from_millis(2));
			let field_lock = ppc_player1.field.lock();
			let field = field_lock.as_ref().unwrap();

			if field.current.as_ref().unwrap().position.x != 3 {
				continue;
			}

			//dbg!(&field.current);
			let mut next = Vec::new();

			next.push(field.next[0].0);
			next.push(field.next[0].1);

			let mut env = Env::new(&0);
			env.board = field.board.clone();
			env.next[0] = [field.next[0].0, field.next[0].0];
			env.movable_puyo = field.movable_puyo;
			env.center_puyo = field.center_puyo;
			env.puyo_status = field.current.as_ref().unwrap().clone();

			ai.search(&field.board, &field.current.as_ref().unwrap(), &next, &ppc_player1.ojama_status, field.center_puyo, field.movable_puyo, false, &70, &ppc_player1.opponent_status);
			//	Console::print(&env, 0, true, false);
			//	dbg!(&ppc_player1.inputs);
			ppc_player1.inputs = ai.best_move.as_ref().unwrap().path.clone();
			//ppc_player1.origin_pos = Some(field.current.as_ref().unwrap().clone());
			//dbg!(&ppc_player1.origin_pos);
			/*
					match &ai.best_move.as_ref() {
						None => { println!("もう無理...") }
						Some(result) => {
							for input in &result.path {
								//	inputs.push(convert_key_input(input));
		
								match input {
									KeyType::Right => env.move_right(),
									KeyType::Left => env.move_left(),
									KeyType::Top => panic!(),
									KeyType::Down => panic!(),
									KeyType::Drop => {
										env.quick_drop(None);
		
										Console::print(&env, 0, true, false);
										//	thread::sleep(Duration::from_millis(100));
										break;
									}
									KeyType::RotateRight => env.rotate_ccw(),//これ逆で作られてるんだよな
									KeyType::RotateLeft => env.rotate_cw(),
									KeyType::Rotate180 => env.rotate_180()
								}
								//	
								//		thread::sleep(Duration::from_millis(100));
							}
						}
					}
			*/
			//	println!("{:?}",ppc.inputs);
		}


		thread::sleep(Duration::from_millis(2));
	}
}