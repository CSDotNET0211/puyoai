use std::num::NonZeroUsize;

pub struct OjamaStatus(pub u64);

impl OjamaStatus {
	pub fn clone(&self) -> Self {
		OjamaStatus {
			0: 0
		}
	}

	//1つの火力のセットをsizeとtimeで計32bitで入れる
	//お邪魔一覧はそのセット2つまで入れられる、それ以上は起こりえないため強制シフト
	pub unsafe fn push(&mut self, ojama_size: usize, receive_frame: usize) {
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);
		values[2] = values[2 - 2];
		values[3] = values[3 - 2];
		values[0] = ojama_size as u16;
		values[1] = receive_frame as u16;

		if values[0] == 0 && values[1] == 0 {
			values[0] = values[2];
			values[1] = values[3];
			values[2] = 0;
			values[3] = 0;
		}
		
		self.0 = std::mem::transmute(values);
	}

	pub unsafe fn update_one_frame(&mut self) {
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);

		values[1] = values[1].saturating_sub(1);
		values[3] = values[3].saturating_sub(1);

		if values[0] == 0 && values[1] == 0 {
			values[0] = values[2];
			values[1] = values[3];
			values[2] = 0;
			values[3] = 0;
		}

		self.0 = std::mem::transmute(values);
	}

	///与えた火力分相殺します。余りが返ります。
	pub unsafe fn offset(&mut self, mut attack: usize) -> usize {
		//相殺はfrontから
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);
		if values[1] == 0 {
			if values[0] >= attack as u16 {
				values[0] -= attack as u16;
				attack = 0;
			} else {
				values[0] = 0;
				attack -= values[0] as usize;
			}
		}

		if values[3] == 0 {
			if values[2] >= attack as u16 {
				values[2] -= attack as u16;
				attack = 0;
			} else {
				values[2] = 0;
				attack -= values[2] as usize;
			}
		}

		if values[0] == 0 && values[1] == 0 {
			values[0] = values[2];
			values[1] = values[3];
			values[2] = 0;
			values[3] = 0;
		}

		self.0 = std::mem::transmute(values);
		//self.try_pack();

		attack
	}

	pub fn is_empty(&self) -> bool {
		self.0 == 0
	}

	//1と2のお邪魔両方ともreceiveまでの時間が0だったらまとめる
	//それぞれの操作の前に必ず行う
	unsafe fn try_collect(&mut self) {
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);
		if values[1] == 0 && values[3] == 0 {
			values[0] = values[0] + values[2];
			values[2] = 0;
		}

		self.0 = std::mem::transmute(values);
	}

	//receive_timeが0のお邪魔の数 関数
	pub unsafe fn get_receivable_ojama_size(&self) -> usize {
		let mut ojama_size = 0;
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);

		if values[1] == 0 {
			ojama_size += values[0];
		}
		if values[3] == 0 {
			ojama_size += values[2];
		}

		ojama_size as usize
	}
	//use_garbage 関数 offsetとちょっと似てるかもね receive_timeが0だったら使うよ
	pub unsafe fn use_ojama(&mut self, mut use_size: usize) {
		//使うのはfrontとか関係ない
		let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);
		if values[1] == 0 {
			if values[0] >= use_size as u16 {
				values[0] -= use_size as u16;
				use_size = 0;
			} else {
				values[0] = 0;
				use_size -= values[0] as usize;
			}
		}

		if values[3] == 0 {
			if values[2] >= use_size as u16 {
				values[2] -= use_size as u16;
				use_size = 0;
			} else {
				values[2] = 0;
				use_size -= values[0] as usize;
			}
		}

		if values[0] == 0 && values[1] == 0 {
			values[0] = values[2];
			values[1] = values[3];
			values[2] = 0;
			values[3] = 0;
		}

		self.0 = std::mem::transmute(values);
	}


	/*	unsafe fn try_pack(&mut self) {
			let mut values = std::mem::transmute::<u64, [u16; 4]>(self.0);
			if values[0] == 0 && values[1] == 0 {
				values[0] = values[2];
				values[1] = values[3];
				values[2] = 0;
				values[3] = 0;
			}
			self.0 = std::mem::transmute(values);
		}*/


	/*	pub fn get_ojama_size(&self) -> &u16 { &self.ojama_size }
		pub fn get_receive_time(&self) -> &u16 { &self.receive_time }*/
}