#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rotation(pub u8);

impl Rotation {
	// コンストラクタ（0から3の間で初期化）
	#[inline]
	pub fn new(value: u8) -> Self {
		Rotation(value & 0b11) // 2ビットにマスク
	}
	#[inline]
	pub fn rotate_ccw(&mut self) {
		self.0 = (self.0 + 1) & 0b11
	}
	#[inline]
	pub fn rotate_cw(&mut self) {
		self.0 = (self.0.wrapping_sub(1)) & 0b11
	}
	#[inline]
	pub fn rotate_180(&mut self) {
		self.0 = (self.0.wrapping_sub(2)) & 0b11
	}

/*	// 現在の状態を取得
	#[inline]
	pub fn value(&self) -> u8 {
		self.0
	}*/
}


pub enum Rotate {
	Cw,
	Ccw,
	Turn,
}