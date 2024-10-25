use rand::thread_rng;
use revonet::neproblem::NeuroProblem;
use revonet::neuro::{ActivationFunctionType, MultilayeredNetwork, NeuralArchitecture, NeuralNetwork};

use ai::build_ai::AI;
use ai::evaluator::nn_evaluator::NNEvaluator;

use crate::battle_env::BattleEnv;

#[derive(Clone)]
pub struct BattleProblem {}

#[allow(dead_code)]
impl BattleProblem {
	pub fn new() -> BattleProblem { BattleProblem {} }
}

impl NeuroProblem for BattleProblem {
	fn get_inputs_num(&self) -> usize {
		17
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
		panic!()
	}


	fn compute_with_net_battle<T: NeuralNetwork>(&self, net1: &mut T, net2: &mut T) -> (f32, f32) {

		//return (1., 0.);
		let result;
		unsafe {
			let ai1 = AI::new(NNEvaluator::new(net1.clone()));
			let ai2 = AI::new(NNEvaluator::new(net2.clone()));
			let mut battle = BattleEnv::new(ai1, ai2);

			loop {
				battle.update();
				if battle.check_winner() == 1 {
					result = (1., 0.);
					break;
				} else if battle.check_winner() == 2 {
					result = (0., 1.);
					break;
				}

				if battle.game_frame % 3600 == 0 {
					if battle.player1.board.is_same(&battle.player2.board.0[0], &battle.player2.board.0[1], &battle.player2.board.0[2]) {
						battle.player1.ojama.push(1, 0);
					}
				}

				if battle.game_frame == 36000 {
					println!("警告：何らかの理由によりゲームが終わってない可能性があります、ゲームを強制リセット");
					let ai1 = AI::new(NNEvaluator::new(net1.clone()));
					let ai2 = AI::new(NNEvaluator::new(net2.clone()));

					battle = BattleEnv::new(ai1, ai2);
				}
			}
		}

		result
	}
}

