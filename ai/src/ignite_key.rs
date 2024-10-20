use env::puyo_kind::PuyoKind;

//発火キーぷよ管理
pub struct IgniteKey {
	pub puyo_kind: PuyoKind,
	pub ignite_count: u8,
	pub chain: u8,
	pub score: u32,
	pub elapse_frame: u16,
}

impl IgniteKey {
	pub fn new(puyo_kind: PuyoKind, ignite_count: u8, chain: u8, elapse_frame: u16, score: u32) -> IgniteKey {
		IgniteKey {
			ignite_count,
			puyo_kind,
			chain,
			score,
			elapse_frame,
		}
	}
}