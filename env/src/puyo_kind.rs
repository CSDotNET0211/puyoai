use strum::EnumIter;


pub const COLOR_PUYOS: [PuyoKind; 4] = [PuyoKind::Red, PuyoKind::Yellow, PuyoKind::Blue, PuyoKind::Green];

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, EnumIter)]
pub enum PuyoKind {
	Empty = 0b000,
	Ojama = 0b001,
	Wall = 0b010,
	Preserved = 0b011,//調整用
	Red = 0b100,
	Green = 0b101,
	Blue = 0b110,
	//Purple,
	Yellow = 0b111,

}


impl PuyoKind {
	pub fn from_bits(value: u8) -> PuyoKind {
		match value {
			0b000 => PuyoKind::Empty,
			0b001 => PuyoKind::Ojama,
			0b010 => PuyoKind::Wall,
			0b011 => PuyoKind::Preserved,
			0b100 => PuyoKind::Red,
			0b101 => PuyoKind::Green,
			0b110 => PuyoKind::Blue,
			0b111 => PuyoKind::Yellow,
			_ => PuyoKind::Empty
		}
	}
	pub fn to_string(&self) -> &str {
		match self {
			PuyoKind::Empty => "E",
			PuyoKind::Ojama => "O",
			PuyoKind::Wall => "W",
			PuyoKind::Preserved => "P",
			PuyoKind::Red => "R",
			PuyoKind::Green => "G",
			PuyoKind::Blue => "B",
			PuyoKind::Yellow => "Y",
		}
	}
}

