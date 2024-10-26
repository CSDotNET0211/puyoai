use rand::{Rng, thread_rng};
use revonet::neproblem::NeuroProblem;
use revonet::neuro::{ActivationFunctionType, MultilayeredNetwork, NeuralArchitecture, NeuralNetwork};

use ai::build_ai::AI;
use ai::evaluator::nn_evaluator::NNEvaluator;
use ai::key_type::KeyType;
use env::env::Env;

#[derive(Clone)]
pub struct ScoreProblem {}

#[allow(dead_code)]
impl ScoreProblem {
	pub fn new() -> ScoreProblem { ScoreProblem {} }
}

impl NeuroProblem for ScoreProblem {
	fn get_inputs_num(&self) -> usize {
		13
	}

	fn get_outputs_num(&self) -> usize {
		1
	}

	fn get_default_net(&self) -> MultilayeredNetwork {
		let mut rng1 = thread_rng();
		let mut net: MultilayeredNetwork = MultilayeredNetwork::new(self.get_inputs_num(), self.get_outputs_num());
		net.add_hidden_layer(10usize, ActivationFunctionType::Relu)
			.build(&mut rng1, NeuralArchitecture::Multilayered);

		net
	}

	fn compute_with_net<T: NeuralNetwork>(&self, net: &mut T) -> f32 {
		//7200フレームゲームをプレイ
		let clone_net = net.clone();
		let mut ai = AI::new(unsafe { NNEvaluator::new(clone_net) });
		let mut next = Vec::new();
		unsafe {
			let seed = thread_rng().gen_range(0, 65535) as u32;
			let mut env = Env::new(&seed);
			env.init();

			while env.current_frame <= 7200 && !env.dead {
				next.clear();
				for next_p2 in env.next[0] {
					next.push(next_p2);
				}
				ai.search(&env.board, &env.puyo_status, &next, &env.ojama, env.center_puyo, env.movable_puyo, env.all_cleared,&env.ojama_rate);

				let path = ai.best_move.as_ref().unwrap();
				for key in path.path.iter() {
					match key {
						KeyType::Right => env.move_right(),
						KeyType::Left => env.move_left(),
						KeyType::Top => panic!(),
						KeyType::Down => panic!(),
						KeyType::Drop => {
							env.quick_drop(None);
							break;
						}
						KeyType::RotateRight => env.rotate_ccw(),
						KeyType::RotateLeft => env.rotate_cw(),
						KeyType::Rotate180 => { env.rotate_180() }
					}
				}
			}

			return -1. * env.current_score as f32;
		};
	}

	fn compute_with_net_battle<T: NeuralNetwork>(&self, net1: &mut T, net2: &mut T) -> (f32, f32) {
		panic!()
	}
}

