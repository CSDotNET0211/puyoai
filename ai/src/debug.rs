use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Write;
use env::vector2::Vector2;
use crate::path::Path;
use crate::potential::Potential;

#[derive(Debug, Clone, )]
pub struct Debug {
	pub link2_count: usize,
	pub link3_count: usize,
	pub near_empty_count: usize,
	pub ignite_pos: Vector2,
	pub waste_chain_link: usize,
	pub one_side_chain_count: usize,
	pub potential_added_count: usize,
	pub instant_attack_count:usize
}

impl Debug {
	pub unsafe fn new() -> Debug {
		Debug {
			link2_count: 0,
			link3_count: 0,
			ignite_pos: Vector2::default(),
			near_empty_count: 0,
			potential_added_count: 0,
			waste_chain_link: 0,
			one_side_chain_count: 0,
			instant_attack_count:0
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