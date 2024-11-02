//pub mod test_evaluator;
//pub mod simple_evaluator;

pub mod simple_evaluator;
pub mod nn_evaluator;
//mod test_evaluator;

use env::board::Board;
use env::ojama_status::OjamaStatus;
use crate::debug::Debug;
use crate::opponent_status::OpponentStatus;
use crate::potential::Potential;

pub trait Evaluator {
	fn evaluate(&mut self, board: &Board, sim_board: &Board, chain: &u8, score: &usize, elapse_frame: &u32, debug: &mut Debug, ojama: &OjamaStatus, ojama_rate: &usize,best_potential: &Potential,opponent_status: &OpponentStatus) -> f32;
	fn clone(&self) -> Self;
}