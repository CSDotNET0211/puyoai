use std::{fs, thread};
use std::collections::{HashMap, VecDeque};
use std::io::stdin;
use std::time::{Duration, Instant};

#[cfg(feature = "ppc")]
use ppc::PpcPuyoKind;
use revonet::ea::{EA, Individual};
use revonet::ne::NE;
use revonet::neuro::MultilayeredNetwork;

use ai::build_ai::AI;
use ai::evaluator::nn_evaluator::NNEvaluator;
use ai::evaluator::simple_evaluator::SimpleEvaluator;
use ai::key_type::KeyType;
use console::console::Console;
use env::env::Env;
use env::puyo_kind::PuyoKind;
use env::puyo_status::PuyoStatus;
use env::rotation::Rotation;
use env::vector2::Vector2;

use crate::battle_env::BattleEnv;
use crate::log::Log;
use crate::log::LogType::INFO;
use crate::problems::battle_problem::BattleProblem;

mod log;
mod battle_env;
mod problems;
#[cfg(feature = "ppc")]
mod ppc_wrapper;

//use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
//use rand::Rng;

static COLOR_PUYOS: [PuyoKind; 4] = [PuyoKind::Green, PuyoKind::Red, PuyoKind::Blue, PuyoKind::Yellow];

#[cfg(feature = "ppc")]
fn check_and_register_using_puyos(raw_puyo: &PpcPuyoKind, left_puyos: &mut Vec<PuyoKind>, puyo_mapping: &mut HashMap<PpcPuyoKind, PuyoKind>) -> PuyoKind {
//mappingになくて変換失敗したら

//let 
	if !puyo_mapping.contains_key(raw_puyo) {
//	if !*using_puyos.contains(raw_puyo) {
		let temp = convert_puyo_kind(raw_puyo);
		let mut result = left_puyos.iter().position(|x| *x == temp);
		if result == None {
			result = Option::from(0usize);
		}

		let index = result.unwrap();
		let selected_puyo = left_puyos[index];

		puyo_mapping.insert(*raw_puyo, selected_puyo);
		println!("added {:?} to {:?}", raw_puyo, selected_puyo);
		left_puyos.remove(index);
		selected_puyo
	} else {
		puyo_mapping[raw_puyo]
	}
}

fn main() {
	unsafe {
		let mut input = String::new();
		println!("Select mode.\n\
		1.Console AI\n\
		2.Console Human\n\
		3.PPC AI\n\
		4.Training\n\
		5.Battle");
		/*	io::stdin()
				.read_line(&mut input)
				.unwrap();
	*/
		input = "4".parse().unwrap();

		match input.trim() {
			"1" => {}
			"2" => {
				let mut env = env::env::Env::new(&0);
				env.init();

				let json_str = fs::read_to_string(r"test.json").unwrap();
				let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();

				let mut ai = AI::new(NNEvaluator::new(net));


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


					//	let mut input_str = Default::default();
					//	stdin().read_line(&mut input_str);

					/*match input_str.trim() {
						"model1" => {
							ai.style = Style::AttackMain;
						}
						"model2" => {}
						"model3" => {}
						"model4" => {
							ai.style = Style::Build;
						}
						_ => {}
					}*/
					panic!();
					let start = Instant::now();
					//			ai.search(&env.board, &env.puyo_status, &next, &env.ojama, env.center_puyo, env.movable_puyo, env.all_cleared, &env.ojama_rate);
					let duration = start.elapsed();


					match input.as_str() {
						"right" => env.move_right(),
						"left" => env.move_left(),
						"drop" => env.quick_drop(None),
						"cw" => env.rotate_cw(),
						"ccw" => env.rotate_ccw(),
						"180" => env.rotate_180(),
						"ai" => {
							let path = ai.best_move.as_ref().unwrap();
							//let mut clear_console_flag = false;
							for key in path.path.iter() {
								match key {
									KeyType::Right => env.move_right(),
									KeyType::Left => env.move_left(),
									KeyType::Top => panic!(),
									KeyType::Down => panic!(),
									KeyType::Drop => {
										env.quick_drop(None);
										//			clear_console_flag = true;
										break;
									}
									KeyType::RotateRight => env.rotate_ccw(),
									KeyType::RotateLeft => env.rotate_cw(),
									KeyType::Rotate180 => { env.rotate_180() }
								}

								Console::print(&env, 0, true, false);

								thread::sleep(Duration::from_millis(100));
							}

							//	println!("{:?}", ai.debug);
							//		println!("{:?}", ai.best_move);
							println!("elapsed time: {:?}", duration);
							println!("score: {:?}", env.current_score);
							println!("frame: {:?}", env.current_frame);
							thread::sleep(Duration::from_millis(1000));
						}
						_ => {}
					}
				}
			}
			"3" => {
				#[cfg(feature = "ppc")]
				ppc();

				panic!("no PPC supports found")
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


				let setting = revonet::settings::EASettings::new(64, 999999999, 25);
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
				let mut battle = BattleEnv::new(ai.clone(), ai.clone());

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
			_ => {}
		}
	}
}

#[cfg(feature = "ppc")]
unsafe fn ppc() {
	//		let mut scp2 = ScpBus::new().unwrap();
	//		let controller = scp2.plug_in(1).unwrap();
	//		thread::sleep(Duration::from_secs(1000));
	let mut scp = ppc::scp::Scp::new();
	let mut controller = ppc::controller::Controller::new();
	let mut left_puyos = COLOR_PUYOS.to_vec();
	let mut puyo_mapping = HashMap::new();
	puyo_mapping.insert(PpcPuyoKind::Null, PuyoKind::Empty);
	puyo_mapping.insert(PpcPuyoKind::Garbage, PuyoKind::Ojama);

	let mut env = Env::new(&0);
	env.init();

	/*let mut prev = 0;
	loop {
		let a = controller.get_frame();
		if a != prev {
			println!("{}", a);
			prev = a;
		}


		thread::sleep(Duration::from_micros(100));
	}*/

	let json_str = fs::read_to_string("./test.json").unwrap();
	let net: MultilayeredNetwork = serde_json::from_str(&json_str).unwrap();

	/*	let mut rng = rand::thread_rng();
		let mut net: MultilayeredNetwork = MultilayeredNetwork::new(25, 1);
		net.add_hidden_layer(10 as usize, ActivationFunctionType::Relu)
			.add_hidden_layer(2 as usize, ActivationFunctionType::Relu)
			.build(&mut rng, NeuralArchitecture::Multilayered);
*/


	//let mut ai = AI::new(SimpleEvaluator::new([3., 5., 10., 18., 26., -15., -10.4, -0.2]));
	let mut ai = AI::new(NNEvaluator::new(net));

	loop {
		loop {
			let state = controller.ppc.get_movable_state();

			let result = match state {
				Ok(state) => { state }
				Err(_) => {
					false
				}
			};

			if result == false {
				thread::sleep(Duration::from_micros(50));
			} else {
				break;
			}
		}

		let next = controller.get_next();
		let mut i = 0;
		for next_puyo in [next[0].0, next[0].1, next[1].0, next[1].1] {
			//mappingを作成

			let puyo = check_and_register_using_puyos(&next_puyo, &mut left_puyos, &mut puyo_mapping);
			env.next[i / 2][i % 2] = puyo;
			i += 1;
		}

		//let mut dest_board = unsafe { Board::new() };
		let board = controller.get_board();
		for y in 1..=13 {
			for x in 0..6 {
				let raw_puyo = &board[(x + (y) * 6) as usize];
				env.board.set_flag(x + 1, 14 - (y + 1) + 1, &check_and_register_using_puyos(&raw_puyo, &mut left_puyos, &mut puyo_mapping));
			}
		}

		for x in 0..6 {
			let raw_puyo = &board[(x + 0 * 6) as usize];
			env.board.set_flag(x + 1, 14, &check_and_register_using_puyos(&raw_puyo, &mut left_puyos, &mut puyo_mapping));
		}


		let current;
		loop {
			while controller.get_movable_state() != Ok(true) { thread::sleep(Duration::from_micros(100)); }
			let temp_current = controller.get_current();
			match temp_current {
				Ok(value) => {
					current = value;
					break;
				}
				Err(_) => { let _ = Duration::from_micros(100); }
			}
		}

		let rotation = match current.rotation {
			0 => Rotation(3),
			1 => Rotation(2),
			2 => Rotation(1),
			3 => Rotation(0),
			_ => { panic!() }
		};

		//		unsafe { env.board.set_flag(current.position.0 + 1, current.position.1, &PuyoKind::Empty); }
		// unsafe { env.board.set_flag(current.position.0 + ROTATE_DIFF[current.rotation as usize][0] + 1, current.position.1 + ROTATE_DIFF[current.rotation as usize][1], &PuyoKind::Empty); }


		env.puyo_status = PuyoStatus::new(Vector2::new(current.position.0, 16 - current.position.1 - 1), rotation);

		env.movable_puyo = check_and_register_using_puyos(&current.movable_puyo, &mut left_puyos, &mut puyo_mapping);
		env.center_puyo = check_and_register_using_puyos(&current.center_puyo, &mut left_puyos, &mut puyo_mapping);

		let mut next = Vec::new();
		for next_p2 in env.next[0] {
			next.push(next_p2);
		}
		for next_p2 in env.next[1] {
			next.push(next_p2);
		}


		let start = Instant::now();
		ai.search(&env.board, &env.puyo_status, &next, env.center_puyo, env.movable_puyo, 0);
		let duration = start.elapsed();
		println!("think time:{:?}", duration);
		println!("current frame:{:?}", env.current_frame);
		println!("current score:{:?}", env.current_score);

		let mut inputs = Vec::new();
		match &ai.best_move.as_ref() {
			None => { println!("もう無理...") }
			Some(result) => {
				for input in &result.path {
					inputs.push(convert_key_input(input));

					match input {
						KeyType::Right => env.move_right(),
						KeyType::Left => env.move_left(),
						KeyType::Top => panic!(),
						KeyType::Down => panic!(),
						KeyType::Drop => {
							env.quick_drop(None);

							Console::print(&env, 0, true, false);
							thread::sleep(Duration::from_millis(100));
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

		Console::print(&env, 0, true, false);
		println!("{:?}", inputs);

		controller.operate(&inputs, &mut scp);


//					thread::sleep(Duration::from_millis(1000));


		//Console::print(&env, false, false);
		//
	}
}