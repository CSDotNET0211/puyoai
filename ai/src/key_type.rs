#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum KeyType {
	Right,
	Left,
	Top,
	Down,
	Drop,
	RotateRight,
	RotateLeft,
	Rotate180
}