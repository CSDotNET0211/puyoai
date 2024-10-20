﻿//pub mod test_evaluator;
//pub mod simple_evaluator;

pub mod simple_evaluator;
pub mod nn_evaluator;
//mod test_evaluator;

use env::board::Board;
use crate::debug::Debug;

pub trait Evaluator {
	fn evaluate(&mut self, board: &Board, score: &usize, elapse_frame: &u32, debug: &mut Debug) -> f32;
	fn clone(&self)->Self;
}