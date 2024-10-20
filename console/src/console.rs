use std::io::{stdout, Write};

use crossterm::{cursor, queue};
use crossterm::cursor::{DisableBlinking, Hide};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::style::{Color, Print, SetBackgroundColor};
use crossterm::terminal::{Clear, ClearType};

use env;
use env::board::{Board, HEIGHT_WITH_BORDER, WIDTH_WITH_BORDER};
use env::env::{Env, HEIGHT, WIDTH};
use env::puyo_kind::PuyoKind;
use env::puyo_kind::PuyoKind::Ojama;

pub struct Console {}

impl Console {
	pub fn get_input() -> String {
		match crossterm::event::read().unwrap() {
			Event::Key(KeyEvent {
						   code: KeyCode::Char('1'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "mode1".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('2'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "mode2".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('3'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "mode3".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('4'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "mode4".to_owned(),

			Event::Key(KeyEvent {
						   code: KeyCode::Char('.'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "right".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('m'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "left".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('k'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "drop".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('c'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "cw".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('x'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "ccw".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Char('v'),
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Press,
						   state: KeyEventState::NONE
					   }) => "180".to_owned(),
			Event::Key(KeyEvent {
						   code: KeyCode::Enter,
						   modifiers: KeyModifiers::NONE,
						   kind: KeyEventKind::Release,
						   state: KeyEventState::NONE
					   }) => "ai".to_owned(),

			_ => "".to_owned()
		}
	}
	#[allow(unused_must_use)]
	pub fn print_board(board: &Board) {
		let mut stdout = stdout();
		queue!(
        stdout,
        Hide,
        DisableBlinking,
        cursor::MoveTo(0, 0),
       Clear(ClearType::All)
    ).unwrap();
		unsafe {
			let puyo_kind = Ojama;
			let bitboard = board.get_bits(puyo_kind);

			for y in 0..HEIGHT {
				for x in 0..WIDTH {
					if bitboard.get_1_flag((x + y * WIDTH_WITH_BORDER as usize) as i8) {
						let color = Self::get_color(&puyo_kind);

						queue!(
            		stdout,
            		SetBackgroundColor(color),
            		Print("  "),
        		).unwrap();
					}
				}
				queue!(stdout, Print("\n")).unwrap();
			}


			queue!(stdout,SetBackgroundColor(Color::Black));
		}
	}
	#[allow(unused_must_use)]
	pub fn print(env: &Env, player_index: usize, current_visible: bool, clear: bool) {
		let mut stdout = stdout();
		queue!(
        stdout,
        Hide,
        DisableBlinking,
        cursor::MoveTo(0, (player_index*30) as u16),
        //Clear(ClearType::CurrentLine)
    ).unwrap();
		if clear {
			queue!(stdout,Clear(ClearType::All));
		}
		let mut temp_board = [PuyoKind::Empty; 8 * 16];

		unsafe {
			//	let puyo_kind = Ojama;
			for puyo_kind in [PuyoKind::Yellow, PuyoKind::Red, PuyoKind::Blue, PuyoKind::Green, PuyoKind::Ojama, PuyoKind::Wall] {
				let bitboard = env.board.get_bits(puyo_kind);


				for y in 0..HEIGHT_WITH_BORDER {
					for x in 0..WIDTH_WITH_BORDER {
						if bitboard.get_1_flag((x * HEIGHT_WITH_BORDER + y) as i8) {
							temp_board[(x + y * WIDTH_WITH_BORDER) as usize] = puyo_kind;
							//	let color = Self::get_color(&puyo_kind);
						}
					}
				}
			}
		}


		if current_visible {
			temp_board[(env.puyo_status.position.x + env.puyo_status.position.y * WIDTH_WITH_BORDER as i8) as usize] = env.center_puyo;
			temp_board[((env.puyo_status.position.x + env.puyo_status.position_diff.x) +
				(env.puyo_status.position.y + env.puyo_status.position_diff.y) *
					WIDTH_WITH_BORDER as i8) as usize] = env.movable_puyo;
			/*
			if env.puyo_status.position.y >= 0
			{
				queue!(stdout, cursor::MoveTo(((env.puyo_status.position.x)*2) as u16  , ((env.puyo_status.position.y)) as u16),
			SetBackgroundColor(Self::get_color(&env.center_puyo)),
			Print("  "));
			}
	
			if env.puyo_status.position.y + env.puyo_status.position_diff.y >= 0
			{
				queue!(stdout, cursor::MoveTo((( env.puyo_status.position.x + env.puyo_status.position_diff.x )*2) as u16  , (( env.puyo_status.position.y + env.puyo_status.position_diff.y )) as u16),
			SetBackgroundColor(Self::get_color(&env.movable_puyo)),
			Print("  "));
			}*/
		}

		/*if env.board[2 + 2 * WIDTH] == PuyoKind::None {
			queue!(stdout, cursor::MoveTo((( 2)*2) as u16  , (( 3)) as u16),
				 SetBackgroundColor(Color::White),
		  SetForegroundColor(Color::Red),
		 
			Print("✖  "));
		}*/

		for y in (0..HEIGHT_WITH_BORDER).rev() {
			for x in 0..WIDTH_WITH_BORDER {
				let color = Self::get_color(&temp_board[x as usize + y as usize * WIDTH_WITH_BORDER as usize]);

				queue!(
				stdout,
				SetBackgroundColor(color),
				Print("  "),
			).unwrap();
			}

			queue!(stdout,SetBackgroundColor(Color::Black));
			queue!(stdout, Print("\n")).unwrap();
		}


		queue!(stdout, cursor::MoveTo((( 10)*2) as u16  , (( 1 )) as u16),
	  SetBackgroundColor(Self::get_color(&env.next[0][0])),
		Print("  "));
		queue!(stdout, cursor::MoveTo(((10)*2) as u16  , (( 2 )) as u16),
	  SetBackgroundColor(Self::get_color(&env.next[0][1])),
		Print("  "));

		queue!(stdout, cursor::MoveTo((( 10)*2) as u16  , (( 4 )) as u16),
	  SetBackgroundColor(Self::get_color(&env.next[1][0])),
		Print("  "));
		queue!(stdout, cursor::MoveTo(((10)*2) as u16  , (( 5 )) as u16),
	  SetBackgroundColor(Self::get_color(&env.next[1][1])),
		Print("  "));


		queue!(stdout,SetBackgroundColor(Color::Black));
		queue!(stdout, cursor::MoveTo((( 0)*2) as u16  , (( 18 )) as u16));

		//   Print("  "));

		stdout.flush().unwrap();
	}


	fn get_color(puyo_kind: &PuyoKind) -> Color {
		match puyo_kind {
			PuyoKind::Empty => Color::White,
			PuyoKind::Blue => Color::Blue,
			PuyoKind::Red => Color::Red,
			PuyoKind::Green => Color::Green,
			//	PuyoKind::Purple => Color::Rgb { r: 128, g: 0, b: 128 },
			PuyoKind::Yellow => Color::Yellow,
			PuyoKind::Ojama => Color::Grey,
			PuyoKind::Wall => Color::DarkBlue,
			PuyoKind::Preserved => panic!()
		}
	}
}
