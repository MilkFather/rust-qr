use bitvec::prelude::*;
use ndarray::prelude::*;

#[cfg(feature = "rayon")]
use ndarray::{
	Zip,
	parallel::prelude::*
}; 

use crate::ec::ErrorCorrectionLevel;

const POS_ADJ_COOR_TABLE_2: [[usize; 2]; 5] = [[6, 18], [6, 22], [6, 26], [6, 30], [6, 34]]; /* Version 2 - 6 */
const POS_ADJ_COOR_TABLE_3: [[usize; 3]; 7] = [[6, 22, 38], [6, 24, 42], [6, 26, 46], [6, 28, 50], [6, 30, 54], [6, 32, 58], [6, 34, 62]]; /* Version 7 - 13 */
const POS_ADJ_COOR_TABLE_4: [[usize; 4]; 7] = [[6, 26, 46, 66], [6, 26, 48, 70], [6, 26, 50, 74], [6, 30, 54, 78], [6, 30, 56, 82], [6, 30, 58, 86], [6, 34, 62, 90]]; /* Version 14 - 20 */ 
const POS_ADJ_COOR_TABLE_5: [[usize; 5]; 7] = [[6, 28, 50, 72, 94], [6, 26, 50, 74, 98], [6, 30, 54, 78, 102], [6, 28, 54, 80, 106], [6, 32, 58, 84, 110], [6, 30, 58, 86, 114], [6, 34, 62, 90, 118]]; /* Version 21 - 27 */
const POS_ADJ_COOR_TABLE_6: [[usize; 6]; 7] = [[6, 26, 50, 74, 98, 122], [6, 30, 54, 78, 102, 126], [6, 26, 52, 78, 104, 130], [6, 30, 56, 82, 108, 134], [6, 34, 60, 86, 112, 138], [6, 30, 58, 86, 114, 142], [6, 34, 62, 90, 118, 146]]; /* Version 28 - 34 */
const POS_ADJ_COOR_TABLE_7: [[usize; 7]; 6] = [[6, 30, 54, 78, 102, 126, 150], [6, 24, 50, 76, 102, 128, 154], [6, 28, 54, 80, 106, 132, 158], [6, 32, 58, 84, 110, 136, 162], [6, 26, 54, 82, 110, 138, 166], [6, 30, 58, 86, 114, 142, 170]]; /* Version 35 - 40 */

const FORMAT_INFO_BY_MASK_L: [[u8; 15]; 8] = [[1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 2, 2, 1, 2, 2], [1, 1, 1, 2, 2, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1], [1, 1, 1, 1, 1, 2, 1, 1, 2, 1, 2, 1, 2, 1, 2], [1, 1, 1, 1, 2, 2, 2, 1, 2, 2, 1, 1, 1, 2, 1], [1, 1, 2, 2, 1, 1, 2, 2, 2, 1, 2, 1, 1, 1, 1], [1, 1, 2, 2, 2, 1, 1, 2, 2, 2, 1, 1, 2, 2, 2], [1, 1, 2, 1, 1, 2, 2, 2, 1, 2, 2, 2, 2, 2, 1], [1, 1, 2, 1, 2, 2, 1, 2, 1, 1, 1, 2, 1, 1, 2]];
const FORMAT_INFO_BY_MASK_M: [[u8; 15]; 8] = [[1, 2, 1, 2, 1, 2, 2, 2, 2, 2, 1, 2, 2, 1, 2], [1, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1], [1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 2, 2], [1, 2, 1, 1, 2, 1, 1, 2, 1, 2, 2, 1, 2, 1, 1], [1, 2, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 2, 2, 1], [1, 2, 2, 2, 2, 2, 2, 1, 1, 2, 2, 1, 1, 1, 2], [1, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2, 1, 1, 1], [1, 2, 2, 1, 2, 1, 2, 1, 2, 1, 2, 2, 2, 2, 2]];
const FORMAT_INFO_BY_MASK_Q: [[u8; 15]; 8] = [[2, 1, 1, 2, 1, 2, 1, 2, 1, 2, 1, 1, 1, 1, 1], [2, 1, 1, 2, 2, 2, 2, 2, 1, 1, 2, 1, 2, 2, 2], [2, 1, 1, 1, 1, 1, 1, 2, 2, 1, 1, 2, 2, 2, 1], [2, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2, 2, 1, 1, 2], [2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 1, 2, 1, 2, 2], [2, 1, 2, 2, 2, 2, 1, 1, 2, 2, 2, 2, 2, 1, 1], [2, 1, 2, 1, 1, 1, 2, 1, 1, 2, 1, 1, 2, 1, 2], [2, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 1, 2, 1]];
const FORMAT_INFO_BY_MASK_H: [[u8; 15]; 8] = [[2, 2, 1, 2, 1, 1, 2, 1, 2, 2, 2, 1, 2, 2, 1], [2, 2, 1, 2, 2, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2], [2, 2, 1, 1, 1, 2, 2, 1, 1, 1, 2, 2, 1, 1, 1], [2, 2, 1, 1, 2, 2, 1, 1, 1, 2, 1, 2, 2, 2, 2], [2, 2, 2, 2, 1, 1, 1, 2, 1, 1, 2, 2, 2, 1, 2], [2, 2, 2, 2, 2, 1, 2, 2, 1, 2, 1, 2, 1, 2, 1], [2, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 2], [2, 2, 2, 1, 2, 2, 2, 2, 2, 1, 1, 1, 2, 1, 1]];

const VERSION_INFO: [[u8; 18]; 34] = [[2, 2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2], [2, 2, 1, 2, 2, 2, 2, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2], [2, 2, 1, 2, 2, 1, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 2, 1], [2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 1, 2, 1, 2, 2, 1, 1], [2, 2, 1, 2, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2], [2, 2, 1, 1, 2, 2, 2, 1, 1, 1, 2, 1, 1, 2, 2, 2, 1, 2], [2, 2, 1, 1, 2, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 1, 1, 1], [2, 2, 1, 1, 1, 2, 2, 1, 1, 2, 2, 2, 2, 2, 1, 1, 2, 1], [2, 2, 1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 2], [2, 1, 2, 2, 2, 2, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 2], [2, 1, 2, 2, 2, 1, 2, 1, 2, 2, 2, 1, 2, 1, 1, 1, 2, 1], [2, 1, 2, 2, 1, 2, 1, 2, 1, 2, 2, 2, 2, 1, 2, 1, 1, 1], [2, 1, 2, 2, 1, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 2, 1, 2], [2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 1, 2, 1, 2, 2, 1, 1, 2], [2, 1, 2, 1, 2, 1, 2, 1, 1, 2, 1, 2, 2, 2, 2, 2, 1, 1], [2, 1, 2, 1, 1, 2, 1, 2, 2, 2, 1, 1, 2, 2, 1, 2, 2, 1], [2, 1, 2, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 2, 1, 1, 2, 2], [2, 1, 1, 2, 2, 2, 1, 1, 1, 2, 1, 1, 2, 2, 2, 1, 2, 2], [2, 1, 1, 2, 2, 1, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1], [2, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 2, 1, 1], [2, 1, 1, 2, 1, 1, 2, 2, 2, 2, 1, 2, 2, 2, 1, 1, 1, 2], [2, 1, 1, 1, 2, 2, 1, 1, 2, 2, 2, 2, 2, 1, 1, 2, 1, 2], [2, 1, 1, 1, 2, 1, 2, 2, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1], [2, 1, 1, 1, 1, 2, 1, 1, 2, 1, 2, 1, 1, 1, 2, 1, 2, 1], [2, 1, 1, 1, 1, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 2, 2], [1, 2, 2, 2, 2, 2, 1, 2, 2, 1, 1, 1, 2, 1, 2, 1, 2, 1], [1, 2, 2, 2, 2, 1, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 2, 2], [1, 2, 2, 2, 1, 2, 1, 2, 2, 2, 1, 2, 1, 1, 1, 2, 1, 2], [1, 2, 2, 2, 1, 1, 2, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1], [1, 2, 2, 1, 2, 2, 1, 2, 1, 1, 2, 2, 2, 2, 1, 2, 1, 1], [1, 2, 2, 1, 2, 1, 2, 1, 2, 2, 2, 2, 1, 2, 1, 1, 1, 2], [1, 2, 2, 1, 1, 2, 1, 2, 1, 2, 2, 1, 1, 2, 2, 1, 2, 2], [1, 2, 2, 1, 1, 1, 2, 1, 2, 1, 2, 1, 2, 2, 2, 2, 2, 1], [1, 2, 1, 2, 2, 2, 1, 1, 2, 2, 2, 1, 1, 2, 1, 2, 2, 1]];

fn mat_penality_1(mat: &Array2<u8>) -> u32 {
	let func = |row: ArrayBase<ndarray::ViewRepr<&u8>, Dim<[usize; 1]>>| -> u32 {
		let mut accu_score = 0;
		let mut combo = 1;
		let mut last_color = row[0] % 2;
		for b in row.slice(s![1..]) {
			if last_color == b % 2 {
				combo += 1;
			} else {
				if combo >= 5 {
					accu_score += 3 + (combo - 5) * 1;
				}
				last_color = b % 2;
				combo = 1;
			}
		}
		if combo >= 5 {
			accu_score += 3 + (combo - 5) * 1;
		}
		accu_score as u32
	};

	#[cfg(feature = "rayon")]
	{
		mat.axis_iter(Axis(0)).into_par_iter().map(func).sum::<u32>() + mat.axis_iter(Axis(1)).into_par_iter().map(func).sum::<u32>()
	}
	#[cfg(not(feature = "rayon"))]
	{
		mat.axis_iter(Axis(0)).into_iter().map(func).sum::<u32>() + mat.axis_iter(Axis(1)).into_iter().map(func).sum::<u32>()
	}
}

fn mat_penality_2(mat: &Array2<u8>) -> u32 {
	let matsize = mat.shape()[0];
	let penality_func = |((y, x), b)| {
		let clr = b % 2;
		if mat[[y+1, x]] % 2 == clr && mat[[y, x+1]] % 2 == clr && mat[[y+1, x+1]] % 2 == clr {
			return 3;
		} else {
			return 0;
		}
	};

	#[cfg(feature = "rayon")]
	{
		Zip::indexed(mat.slice(s![0..matsize-1, 0..matsize-1])).into_par_iter(penality_func).map().sum::<u32>()
	}
	#[cfg(not(feature = "rayon"))]
	{
		mat.slice(s![0..matsize-1, 0..matsize-1]).indexed_iter().map(penality_func).sum::<u32>()
	}
}

fn mat_penality_3(mat: &Array2<u8>) -> u32 {
	let func = |row: ArrayBase<ndarray::ViewRepr<&u8>, Dim<[usize; 1]>>| -> u32 {
		let mut score = 0;
		for i in 0..=row.dim()-11 {
			if row[i] % 2 == 0 && row[i+1] % 2 == 0 && row[i+2] % 2 == 0 && row[i+3] % 2 == 0 && row[i+4] % 2 == 1 && row[i+5] % 2 == 0 && row[i+6] % 2 == 1 && row[i+7] % 2 == 1 && row[i+8] % 2 == 1 && row[i+9] % 2 == 0 && row[i+10] % 2 == 1 {
				score += 40;
			} else if row[i] % 2 == 1 && row[i+1] % 2 == 0 && row[i+2] % 2 == 1 && row[i+3] % 2 == 1 && row[i+4] % 2 == 1 && row[i+5] % 2 == 0 && row[i+6] % 2 == 1 && row[i+7] % 2 == 0 && row[i+8] % 2 == 0 && row[i+9] % 2 == 0 && row[i+10] % 2 == 0 {
				score += 40;
			}
		}
		score
	};

	#[cfg(feature = "rayon")]
	{
		mat.axis_iter(Axis(0)).into_par_iter().map(func).sum::<u32>() + mat.axis_iter(Axis(1)).into_par_iter().map(func).sum::<u32>()
	}
	#[cfg(not(feature = "rayon"))]
	{
		mat.axis_iter(Axis(0)).into_iter().map(func).sum::<u32>() + mat.axis_iter(Axis(1)).into_iter().map(func).sum::<u32>()
	}
}

fn mat_penality_4(mat: &Array2<u8>) -> u32 {
	let mut black = 0;
	let total = mat.shape()[0] * mat.shape()[1];
	for b in mat.into_iter() {
		if b % 2 == 1 {
			black += 1;
		}
	}
	let percentage = (black as f64) / (total as f64) * 100_f64;
	let prev = f64::floor(percentage / 5_f64) as i32 * 5;
	let next = f64::ceil(percentage / 5_f64) as i32 * 5;
	u32::max((prev - 50).abs() as u32 / 5, (next - 50).abs() as u32 / 5) * 10
}

fn apply_mask_inplace(mat: &mut Array2<u8>, mask: u8) {
	#[cfg(feature = "rayon")]
	{
		match mask {
			0 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if (y + x) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			1 => { Zip::indexed(mat).par_for_each(|(y, _), b| {if y % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			2 => { Zip::indexed(mat).par_for_each(|(_, x), b| {if x % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			3 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if (y + x) % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			4 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if ((y / 2) + (x / 3)) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			5 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if (y * x) % 2 + (y * x) % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			6 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if ((y * x) % 2 + (y * x) % 3) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			7 => { Zip::indexed(mat).par_for_each(|(y, x), b| {if ((y + x) % 2 + (y * x) % 3) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); 	},
			_ => panic!("Mask interal error")
		}
	}
	#[cfg(not(feature = "rayon"))]
	{
		match mask {
			0 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if (y + x) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			1 => { mat.indexed_iter_mut().for_each(|((y, _), b)| {if y % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			2 => { mat.indexed_iter_mut().for_each(|((_, x), b)| {if x % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			3 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if (y + x) % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			4 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if ((y / 2) + (x / 3)) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			5 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if (y * x) % 2 + (y * x) % 3 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			6 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if ((y * x) % 2 + (y * x) % 3) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); },
			7 => { mat.indexed_iter_mut().for_each(|((y, x), b)| {if ((y + x) % 2 + (y * x) % 3) % 2 == 0 {if *b == 3 { *b = 4; } else if *b == 4 { *b = 3; }}}); 	},
			_ => panic!("Mask interal error")
		}
	}
}

fn select_mask(mat: &Array2<u8>, eclevel: ErrorCorrectionLevel, version: u8) -> u8 {
	#[allow(unused_mut)]
	let mut mask_penality_vec: Vec<u32>;
	let penality_handle = [mat_penality_1, mat_penality_2, mat_penality_3, mat_penality_4];
	#[cfg(feature = "rayon")]
	{
		mask_penality_vec = Vec::<u32>::new();
		(0_u8..=7_u8).into_par_iter().map(|mask_id| {
			let mut test_mat = mat.clone();
			apply_mask_inplace(&mut test_mat, mask_id);
			let format_info = match eclevel {
				ErrorCorrectionLevel::L => &FORMAT_INFO_BY_MASK_L[mask_id as usize],
				ErrorCorrectionLevel::M => &FORMAT_INFO_BY_MASK_M[mask_id as usize],
				ErrorCorrectionLevel::Q => &FORMAT_INFO_BY_MASK_Q[mask_id as usize],
				ErrorCorrectionLevel::H => &FORMAT_INFO_BY_MASK_H[mask_id as usize]
			};
			fill_format_info(&mut test_mat, format_info);
			if version >= 7 {
				fill_version_info(&mut test_mat, &VERSION_INFO[version as usize - 7]);
			}
			penality_handle.into_par_iter().map(|handle| handle(&test_mat)).sum::<u32>()
		}).collect_into_vec(&mut mask_penality_vec);
	}
	#[cfg(not(feature = "rayon"))]
	{
		mask_penality_vec = (0_u8..=7_u8).into_iter().map(|mask_id| {
			let mut test_mat = mat.clone();
			apply_mask_inplace(&mut test_mat, mask_id);
			let format_info = match eclevel {
				ErrorCorrectionLevel::L => &FORMAT_INFO_BY_MASK_L[mask_id as usize],
				ErrorCorrectionLevel::M => &FORMAT_INFO_BY_MASK_M[mask_id as usize],
				ErrorCorrectionLevel::Q => &FORMAT_INFO_BY_MASK_Q[mask_id as usize],
				ErrorCorrectionLevel::H => &FORMAT_INFO_BY_MASK_H[mask_id as usize]
			};
			fill_format_info(&mut test_mat, format_info);
			if version >= 7 {
				fill_version_info(&mut test_mat, &VERSION_INFO[version as usize - 7]);
			}
			penality_handle.iter().map(|handle| handle(&test_mat)).sum::<u32>()
		}).collect();
	}

	let mut result_id = 0;
	let mut min_pen = mask_penality_vec[result_id as usize];
	for idx in 1..=7 {
		if mask_penality_vec[idx as usize] < min_pen {
			min_pen = mask_penality_vec[idx as usize];
			result_id = idx;
		}
	}
	result_id
}

fn fill_format_info(mat: &mut Array2<u8>, info: &[u8; 15]) {
	let size = mat.shape()[0];
	mat[[8, 0]] = info[0]; mat[[size - 1, 8]] = info[0];
	mat[[8, 1]] = info[1]; mat[[size - 2, 8]] = info[1];
	mat[[8, 2]] = info[2]; mat[[size - 3, 8]] = info[2];
	mat[[8, 3]] = info[3]; mat[[size - 4, 8]] = info[3];
	mat[[8, 4]] = info[4]; mat[[size - 5, 8]] = info[4];
	mat[[8, 5]] = info[5]; mat[[size - 6, 8]] = info[5];
	mat[[8, 7]] = info[6]; mat[[size - 7, 8]] = info[6];
	mat[[8, 8]] = info[7]; mat[[8, size - 8]] = info[7];
	mat[[7, 8]] = info[8]; mat[[8, size - 7]] = info[8];
	mat[[5, 8]] = info[9]; mat[[8, size - 6]] = info[9];
	mat[[4, 8]] = info[10]; mat[[8, size - 5]] = info[10];
	mat[[3, 8]] = info[11]; mat[[8, size - 4]] = info[11];
	mat[[2, 8]] = info[12]; mat[[8, size - 3]] = info[12];
	mat[[1, 8]] = info[13]; mat[[8, size - 2]] = info[13];
	mat[[0, 8]] = info[14]; mat[[8, size - 1]] = info[14];
}

fn fill_version_info(mat: &mut Array2<u8>, info: &[u8; 18]) {
	let size = mat.shape()[0];
	let mut version_index: usize = 0;
	for col in 0..=5 {
		for row in size-11..=size-9 {
			mat[[row, col]] = info[17 - version_index];
			version_index += 1;
		}
	}
	version_index = 0;
	for row in 0..=5 {
		for col in size-11..=size-9 {
			mat[[row, col]] = info[17 - version_index];
			version_index += 1;
		}
	}
}

pub fn make_matrix(bitstream: &BitBox<Msb0, u8>, eclevel: ErrorCorrectionLevel, version: u8) -> Array2<bool> {
	let pos_pattern = array![
		[1, 1, 1, 1, 1, 1, 1],
		[1, 2, 2, 2, 2, 2, 1],
		[1, 2, 1, 1, 1, 2, 1],
		[1, 2, 1, 1, 1, 2, 1],
		[1, 2, 1, 1, 1, 2, 1],
		[1, 2, 2, 2, 2, 2, 1],
		[1, 1, 1, 1, 1, 1, 1]];

	let alignment_pattern = array![
		[1, 1, 1, 1, 1],
		[1, 2, 2, 2, 1],
		[1, 2, 1, 2, 1],
		[1, 2, 2, 2, 1],
		[1, 1, 1, 1, 1]];

	let matsize: usize = 4 * version as usize + 17;
	/* Statusmat notation: 0 - uninitialized, 1 - system true, 2 - system false, 3 - data true, 4 - data false */
	let mut statusmat: Array2<u8> = Array2::zeros((matsize, matsize));

	// Step 1: Position Detection
	statusmat.slice_mut(s![..7, ..7]).assign(&pos_pattern);
	statusmat.slice_mut(s![..7, matsize-7..]).assign(&pos_pattern);
	statusmat.slice_mut(s![matsize-7.., ..7]).assign(&pos_pattern);

	// Step 2: Seperator
	statusmat.slice_mut(s![..7, 7]).assign(&array![2]);
	statusmat.slice_mut(s![..7, matsize-8]).assign(&array![2]);
	statusmat.slice_mut(s![matsize-7.., 7]).assign(&array![2]);
	statusmat.slice_mut(s![7, ..8]).assign(&array![2]);
	statusmat.slice_mut(s![matsize-8, ..8]).assign(&array![2]);
	statusmat.slice_mut(s![7, matsize-8..]).assign(&array![2]);

	// Step 3: Alignment Pattern
	let coords: &[usize] = if version <= 0 {panic!("Internal version error")}
	else if version <= 1 {&[]} 
	else if version <= 6 {&POS_ADJ_COOR_TABLE_2[version as usize - 2]}
	else if version <= 13 {&POS_ADJ_COOR_TABLE_3[version as usize - 7]}
	else if version <= 20 {&POS_ADJ_COOR_TABLE_4[version as usize - 14]}
	else if version <= 27 {&POS_ADJ_COOR_TABLE_5[version as usize - 21]}
	else if version <= 34 {&POS_ADJ_COOR_TABLE_6[version as usize - 28]}
	else if version <= 40 {&POS_ADJ_COOR_TABLE_7[version as usize - 35]}
	else {panic!("Internal version error")};

	for coordy in coords.iter() {
		for coordx in coords.iter() {
			if statusmat[[*coordy, *coordx]] == 0 {
				statusmat.slice_mut(s![*coordy-2..=*coordy+2, *coordx-2..=*coordx+2]).assign(&alignment_pattern);
			}
		}
	}

	// Step 4: Timing
	for i in 8..matsize-8 {
		statusmat[[6, i]] = if i % 2 == 1 {2} else {1};
		statusmat[[i, 6]] = if i % 2 == 1 {2} else {1};
	}

	// Step 5: Dark Module
	statusmat[[matsize - 8, 8]] = 1;

	// Step 6: Reserve Format Info & Version Info (Fill 2 first)
	statusmat.slice_mut(s![..6, 8]).assign(&array![2]);
	statusmat.slice_mut(s![8, ..6]).assign(&array![2]);
	statusmat.slice_mut(s![8, matsize-8..]).assign(&array![2]);
	statusmat.slice_mut(s![matsize-7.., 8]).assign(&array![2]);
	statusmat[[8, 7]] = 2; statusmat[[8, 8]] = 2; statusmat[[7, 8]] = 2;

	if version >= 7 {
		statusmat.slice_mut(s![..6, matsize-11..matsize-8]).assign(&array![2]);
		statusmat.slice_mut(s![matsize-11..matsize-8, ..6]).assign(&array![2]);
	}

	// Step 7: Fill Data
	let mut going_up: bool = true;
	let mut bit_index: usize = 0;
	let mut col = matsize - 1;
	let mut write_bit_and_advance = |row, col| {
		if statusmat[[row, col]] == 0 {
			statusmat[[row, col]] = if bitstream[bit_index] {3} else {4};
			bit_index += 1;
		}
	};

	loop {
		if col == 6 {
			col -= 1;
		} else {
			if going_up {
				for row in (0..=matsize - 1).rev() {
					write_bit_and_advance(row, col);
					write_bit_and_advance(row, col - 1);
				}
			} else {
				for row in 0..=matsize - 1 {
					write_bit_and_advance(row, col);
					write_bit_and_advance(row, col - 1);
				}
			}
			going_up = !going_up;
			if col <= 1 { break; } else { col -= 2; }
		}
	}
	assert!(bit_index == bitstream.len());

	// Step 8: Masking
	let selected_mask = select_mask(&statusmat, eclevel, version);
	apply_mask_inplace(&mut statusmat, selected_mask);

	// Step 9: Fill Format Info
	let format_info = match eclevel {
		ErrorCorrectionLevel::L => &FORMAT_INFO_BY_MASK_L[selected_mask as usize],
		ErrorCorrectionLevel::M => &FORMAT_INFO_BY_MASK_M[selected_mask as usize],
		ErrorCorrectionLevel::Q => &FORMAT_INFO_BY_MASK_Q[selected_mask as usize],
		ErrorCorrectionLevel::H => &FORMAT_INFO_BY_MASK_H[selected_mask as usize]
	};
	fill_format_info(&mut statusmat, format_info);

	// Step 10: Fill Version Info
	if version >= 7 {
		fill_version_info(&mut statusmat, &VERSION_INFO[version as usize - 7]);
	}

	// Step 11: Convert to 0-1 Matrix
	Array2::from_shape_fn((matsize, matsize), |(y, x)| if statusmat[[y, x]] % 2 == 0 {false} else {true})
}