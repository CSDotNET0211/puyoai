﻿//pub mod test_evaluator;
//pub mod simple_evaluator;

//pub mod simple_evaluator;
pub mod nn_evaluator;
//mod test_evaluator;

use env::board::Board;
use env::ojama_status::OjamaStatus;
use env::vector2::Vector2;
use crate::debug::Debug;
use crate::opponent_status::OpponentStatus;
use crate::potential::Potential;

pub trait Evaluator {
	fn evaluate(&mut self,
				put_board: &Board,
				sim_board: &Board,
				potential: &Potential,
				chain: &u8,
				score: &usize,
				elapse_frame: &u32,
				debug: &mut Debug,
				ojama: &OjamaStatus,
				ojama_rate: &usize,
				opponent_status: &OpponentStatus,
				waste_chain_link: &usize,
				one_side_chain_count: &u8,
				instant_attack_count: &u8,
				attack_value: &usize,
	) -> f32;
	fn clone(&self) -> Self;
}