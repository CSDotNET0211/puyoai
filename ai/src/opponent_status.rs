use revonet::neuro::MultilayeredNetwork;
use env::board::Board;
use env::puyo_kind::PuyoKind;
use crate::build_ai::AI;
use crate::evaluator::nn_evaluator::NNEvaluator;

pub struct OpponentStatus {
	pub potential_chain_count: usize,
	pub potential_added_count: usize,
	pub board_height: usize,
	pub board_ojama_count: usize,
	pub instant_attack: usize,//一定時間以内で打てる1列以上
}

impl OpponentStatus {
	pub unsafe fn new(board: &Board) -> Self {
		let mut opponent_status = Self::default();

		let heights = board.get_heights();
		for x in 1..=6 {
			opponent_status.board_height += heights[x] as usize;
		}

		opponent_status.board_ojama_count = board.get_bits(PuyoKind::Ojama).popcnt128() as usize;
		let result_potential = AI::<NNEvaluator<MultilayeredNetwork>>::get_potential_chain_all(board);
		let result_instant_attack_count = AI::<NNEvaluator<MultilayeredNetwork>>::get_instant_attack(board,&70);


		opponent_status.potential_added_count = result_potential.added_count as usize;
		opponent_status.potential_chain_count = result_potential.chain as usize;
		opponent_status.instant_attack = result_instant_attack_count as usize;

		opponent_status
	}

	pub fn clone(&self) -> Self {
		Self {
			board_height: self.board_height,
			instant_attack: self.instant_attack,
			board_ojama_count: self.board_ojama_count,
			potential_chain_count: self.potential_chain_count,
			potential_added_count: self.potential_added_count,
		}
	}
	pub fn default() -> Self {
		OpponentStatus {
			board_height: 0,
			instant_attack: 0,
			board_ojama_count: 0,
			potential_chain_count: 0,
			potential_added_count: 0,
		}
	}
}

