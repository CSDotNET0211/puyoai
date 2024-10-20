use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use crate::path::Path;

#[derive(Debug,Copy, Clone)]
pub struct Debug {
	pub link2_count: i32,
	pub link3_count: i32,
	pub ignite_count: i32,
	pub attack: i32,
	pub dead:bool
}

impl Debug {
	pub fn new() -> Debug {
		Debug {
			link2_count: -1,
			link3_count: -1,
			ignite_count: -1,
			attack: -1,
			dead:false
		}
	}

	pub fn save_hashtable_as_csv(hash: &HashMap<u16, Path>, rotation: i32) -> Result<(), Error> {
		let mut array = vec![String::new(); 6 * 13];
		array.fill("none".parse().unwrap());
		for x in 0..6 {
			for y in 0..13 {
				let key = (rotation * 1000 + x * 100 + y * 1) as u16;
				if let Some(path) = hash.get(&key) {
					let index = x * 13 + y;
					array[index as usize] = format!("{:?}", path.key_type);
				}
			}
		}

		let mut file = File::create(format!("output_{}.csv", rotation))?;
		for y in 0..13 {
			let row: Vec<String> = (0..6).map(|x| array[x * 13 + y].clone()).collect();
			writeln!(file, "{}", row.join(","))?;
		}

		Ok(())
	}
}