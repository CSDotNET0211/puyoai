use std::arch::x86_64::{__m128i, _mm_and_si128, _mm_cmpeq_epi64, _mm_cmpeq_epi8, _mm_set_epi32, _mm_set_epi64x, _mm_setr_epi32, _mm_store_si128, _pext_u32};
use ai::debug::Debug;
use ai::evaluator::Evaluator;
use ai::evaluator::simple_evaluator::SimpleEvaluator;
use env::board::Board;
use env::board_bit::BoardBit;
use env::puyo_kind::PuyoKind;
use log::debug;
use revonet::neuro::NeuralNetwork;
use ai::evaluator::nn_evaluator::NNEvaluator;
use ai::opener_book::Template;
use env::env::DEAD_POSITION;
use env::split_board::SplitBoard;


#[test]
fn get_bits() {
	unsafe {
		let board_str = "YRGBOWE";
		//	let board_str = "YEEEEEE";
		let board = Board::from_str(board_str);
		let y = board.get_bits(PuyoKind::Yellow);
		let r = board.get_bits(PuyoKind::Red);
		let g = board.get_bits(PuyoKind::Green);
		let b = board.get_bits(PuyoKind::Blue);
		let o = board.get_bits(PuyoKind::Ojama);
		let e = board.get_bits(PuyoKind::Empty);
		let w = board.get_bits(PuyoKind::Wall);

		let mut left;
		let mut right;
		let mut ans;

		{
			ans = 0b1000000;
			left = y.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			ans = 0b0100000;
			left = r.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			ans = 0b0010000;
			left = g.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			ans = 0b0001000;
			left = b.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			ans = 0b0000100;
			left = o.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			ans = 0b0000010;
			left = w.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}

		{
			let a = _mm_set_epi64x(0b0, 0b1);

			ans = 0b0000001;
			left = e.0;
			left = _mm_and_si128(left, _mm_set_epi64x(0, 0b1111111));

			right = _mm_set_epi32(0, 0, 0, ans);
			let result = _mm_cmpeq_epi64(left, right);
			let result: [u32; 4] = std::mem::transmute(result);
			assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
		}
	}
}

#[test]
fn get_erase_flag() {
	unsafe {
		let board =
			"EEEEEE\
		 ERRRRE\
		 EEEEEE";
		let board = Board::from_str(&board);
		let mut board_mask = BoardBit::default();
		board.erase_if_needed(&0, &mut board_mask);
		let ans =
			_mm_set_epi64x(0, 0b000000_011110_000000);

		let result = _mm_cmpeq_epi64(board_mask.0, ans);
		let result: [u32; 4] = std::mem::transmute(result);
		assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
	}
}

#[test]
fn get_erase_frag_with_ojama() {
	let board =
		"WWWWWWWW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEOW\
		 WERRRROW\
		 WEEOOEEW\
		 WWWWWWWW";
	unsafe {
		let board = Board::from_str(&board);
		//	let test = board.get_bits(PuyoKind::Ojama);

		let mut board_mask = BoardBit::default();
		board.erase_if_needed(&0, &mut board_mask);

		let ans =
			_mm_set_epi64x(0b0000000000000000_0010000000000000_0010000000000000_0110000000000000, 0b0110000000000000_0010000000000000_0000000000000000_0000000000000000);


		let result = _mm_cmpeq_epi64(board_mask.0, ans);
		let result: [u32; 4] = std::mem::transmute(result);
		assert_eq!(result, [u32::MAX, u32::MAX, u32::MAX, u32::MAX, ]);
	}
//	assert_eq!(board, ans);
}

#[test]
fn get_erase_drop() {
	let board =
		"WWWWWWWW\
		 WYYRRREW\
		 WRRYYREW\
		 WBBEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW";


	unsafe {
		let mut board1 = Board::from_str(&board);
		let mut board_mask = BoardBit::default();
		board1.erase_if_needed(&0, &mut board_mask);
		board1.drop_after_erased(&board_mask);
		let after = Board::to_str(&board1);
		//	let test_before = Board::new();
		//let test_after = test_before.to_str();
		let ojama = board1.get_bits(PuyoKind::Ojama);
		assert_eq!(true, true);
	}

//	assert_eq!(board, ans);
}

#[test]
fn get_link2() {
	let board =
		"WWWWWWWW\
		 WEEEERRW\
		 WEEEREEW\
		 WEEEEREW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW";


	unsafe {
		let mut board1 = Board::from_str(&board);
		let mut dead = false;

		if !board1.is_empty_cell(DEAD_POSITION.x as i16, DEAD_POSITION.y as i16) {
			dead = true;
		}
		let temp = board1.get_not_empty_board();

		let mut board_mask = BoardBit::default();
		board1.erase_if_needed(&0, &mut board_mask);
		board1.drop_after_erased(&board_mask);
		let after = Board::to_str(&board1);
		//	let test_before = Board::new();
		//let test_after = test_before.to_str();
		let ojama = board1.get_bits(PuyoKind::Yellow);
		let mut debug = Debug::new();
		//	SimpleEvaluator::new().evaluate(&[0., 0., 0., 0., 0., 0.], &board1, &0, &0, &mut debug);

		assert_eq!(true, true);
	}

//	assert_eq!(board, ans);
}


#[test]
fn gtr_score() {
	let board =
		"WWWWWWWW\
		 WEEEERRW\
		 WEGYRGGW\
		 WEEEERGW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW\
		 WEEEEEEW";

	println!("{}", board);
	unsafe {
		let mut board1 = Board::from_str(&board);

		let mut templates = Vec::new();
		templates.push(Template(Box::new([
			_mm_set_epi64x(8590589956, 0),
			_mm_set_epi64x(51539869696, 0),
			_mm_set_epi64x(10, 1125917086711808),
		])));

		let score = templates[0].evaluate(&board1);
		dbg!(score);
	}

//	assert_eq!(board, ans);
}

