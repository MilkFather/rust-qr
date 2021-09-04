#[cfg(feaature = "rayon")]
use rayon_::prelude::*;

#[derive(Clone, Copy)]
pub enum ErrorCorrectionLevel { L, M, Q, H }

const GF_LOGS: [u8; 256] = [
	1, 2, 4, 8, 16, 32, 64, 128, 29, 58, 116, 232, 205, 135, 19, 38,
	76, 152, 45, 90, 180, 117, 234, 201, 143, 3, 6, 12, 24, 48, 96, 192,
	157, 39, 78, 156, 37, 74, 148, 53, 106, 212, 181, 119, 238, 193, 159, 35,
	70, 140, 5, 10, 20, 40, 80, 160, 93, 186, 105, 210, 185, 111, 222, 161,
	95, 190, 97, 194, 153, 47, 94, 188, 101, 202, 137, 15, 30, 60, 120, 240,
	253, 231, 211, 187, 107, 214, 177, 127, 254, 225, 223, 163, 91, 182, 113, 226,
	217, 175, 67, 134, 17, 34, 68, 136, 13, 26, 52, 104, 208, 189, 103, 206,
	129, 31, 62, 124, 248, 237, 199, 147, 59, 118, 236, 197, 151, 51, 102, 204,
	133, 23, 46, 92, 184, 109, 218, 169, 79, 158, 33, 66, 132, 21, 42, 84,
	168, 77, 154, 41, 82, 164, 85, 170, 73, 146, 57, 114, 228, 213, 183, 115,
	230, 209, 191, 99, 198, 145, 63, 126, 252, 229, 215, 179, 123, 246, 241, 255,
	227, 219, 171, 75, 150, 49, 98, 196, 149, 55, 110, 220, 165, 87, 174, 65,
	130, 25, 50, 100, 200, 141, 7, 14, 28, 56, 112, 224, 221, 167, 83, 166,
	81, 162, 89, 178, 121, 242, 249, 239, 195, 155, 43, 86, 172, 69, 138, 9,
	18, 36, 72, 144, 61, 122, 244, 245, 247, 243, 251, 235, 203, 139, 11, 22,
	44, 88, 176, 125, 250, 233, 207, 131, 27, 54, 108, 216, 173, 71, 142, 1
];

const GF_ANTILOGS: [u8; 256] = [
	0 /* garbage value */, 0, 1, 25, 2, 50, 26, 198, 3, 223, 51, 238, 27, 104, 199, 75,
	4, 100, 224, 14, 52, 141, 239, 129, 28, 193, 105, 248, 200, 8, 76, 113,
	5, 138, 101, 47, 225, 36, 15, 33, 53, 147, 142, 218, 240, 18, 130, 69,
	29, 181, 194, 125, 106, 39, 249, 185, 201, 154, 9, 120, 77, 228, 114, 166,
	6, 191, 139, 98, 102, 221, 48, 253, 226, 152, 37, 179, 16, 145, 34, 136,
	54, 208, 148, 206, 143, 150, 219, 189, 241, 210, 19, 92, 131, 56, 70, 64,
	30, 66, 182, 163, 195, 72, 126, 110, 107, 58, 40, 84, 250, 133, 186, 61,
	202, 94, 155, 159, 10, 21, 121, 43, 78, 212, 229, 172, 115, 243, 167, 87,
	7, 112, 192, 247, 140, 128, 99, 13, 103, 74, 222, 237, 49, 197, 254, 24,
	227, 165, 153, 119, 38, 184, 180, 124, 17, 68, 146, 217, 35, 32, 137, 46,
	55, 63, 209, 91, 149, 188, 207, 205, 144, 135, 151, 178, 220, 252, 190, 97,
	242, 86, 211, 171, 20, 42, 93, 158, 132, 60, 57, 83, 71, 109, 65, 162,
	31, 45, 67, 216, 183, 123, 164, 118, 196, 23, 73, 236, 127, 12, 111, 246,
	108, 161, 59, 82, 41, 157, 85, 170, 251, 96, 134, 177, 187, 204, 62, 90,
	203, 89, 95, 176, 156, 169, 160, 81, 11, 245, 22, 235, 122, 117, 44, 215,
	79, 174, 213, 233, 230, 231, 173, 232, 116, 214, 244, 234, 168, 80, 88, 175
];

const EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_L: [usize; 40] = [7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28, 30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30];
const EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_M: [usize; 40] = [10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28];
const EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_Q: [usize; 40] = [13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30, 30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30];
const EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_H: [usize; 40] = [17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30];

fn xor(a: &Vec<u8>, b: &Vec<u8>) -> Vec<u8> {
	let mut a = a.clone();
	let mut b = b.clone();
	let lendiff = b.len() as i32 - a.len() as i32;
	if lendiff < 0 {
		for _ in 0..-lendiff {
			b.push(0);
		}
	} else {
		for _ in 0..lendiff {
			a.push(0);
		}
	}
	let mut remainder: Vec<u8> = Vec::new();
	for i in 1..b.len() {
		remainder.push(b[i]^a[i]);
	}
	return remainder;
}

fn divide(mp: &Vec<u8>, gp: &Vec<u8>) -> Vec<u8> {
	let mp = mp.clone();
	let mut gp = gp.clone();
	if mp[0] > 0 {
		for i in 0..gp.len() {
			let mut temp: u32 = gp[i] as u32 + GF_ANTILOGS[mp[0] as usize] as u32;
			if temp > 255 { temp %= 255; }
			gp[i] = temp as u8;
			gp[i] = GF_LOGS[gp[i] as usize];
		}
		return xor(&gp, &mp);
	} else {
		let mut temp: Vec<u8> = Vec::new();
		for _ in 0..gp.len() {
			temp.push(0);
		}
		return xor(&temp, &mp);
	}
}

fn get_ec(codeword_block: &Vec<u8>, gp_length: usize) -> Vec<u8> {
	let generator_poly: Vec<u8> = match gp_length {
		7 => [0, 87, 229, 146, 149, 238, 102, 21].to_vec(),
		10 => [0, 251, 67, 46, 61, 118, 70, 64, 94, 32, 45].to_vec(),
		13 => [0, 74, 152, 176, 100, 86, 100, 106, 104, 130, 218, 206, 140, 78].to_vec(),
		15 => [0, 8, 183, 61, 91, 202, 37, 51, 58, 58, 237, 140, 124, 5, 99, 105].to_vec(),
		16 => [0, 120, 104, 107, 109, 102, 161, 76, 3, 91, 191, 147, 169, 182, 194, 225, 120].to_vec(),
		17 => [0, 43, 139, 206, 78, 43, 239, 123, 206, 214, 147, 24, 99, 150, 39, 243, 163, 136].to_vec(),
		18 => [0, 215, 234, 158, 94, 184, 97, 118, 170, 79, 187, 152, 148, 252, 179, 5, 98, 96, 153].to_vec(),
		20 => [0, 17, 60, 79, 50, 61, 163, 26, 187, 202, 180, 221, 225, 83, 239, 156, 164, 212, 212, 188, 190].to_vec(),
		22 => [0, 210, 171, 247, 242, 93, 230, 14, 109, 221, 53, 200, 74, 8, 172, 98, 80, 219, 134, 160, 105, 165, 231].to_vec(),
		24 => [0, 229, 121, 135, 48, 211, 117, 251, 126, 159, 180, 169, 152, 192, 226, 228, 218, 111, 0, 117, 232, 87, 96, 227, 21].to_vec(),
		26 => [0, 173, 125, 158, 2, 103, 182, 118, 17, 145, 201, 111, 28, 165, 53, 161, 21, 245, 142, 13, 102, 48, 227, 153, 145, 218, 70].to_vec(),
		28 => [0, 168, 223, 200, 104, 224, 234, 108, 180, 110, 190, 195, 147, 205, 27, 232, 201, 21, 43, 245, 87, 42, 195, 212, 119, 242, 37, 9, 123].to_vec(),
		30 => [0, 41, 173, 145, 152, 216, 31, 179, 182, 50, 48, 110, 86, 239, 96, 222, 125, 42, 173, 226, 193, 224, 130, 156, 37, 251, 216, 238, 40, 192, 180].to_vec(),
		_ => panic!("Internal error")
	};
	let mut remainder = codeword_block.clone();
	for _ in 0..codeword_block.len() {
		remainder = divide(&remainder, &generator_poly);
	}
	return remainder;
}

pub fn get_eclength(eclevel: ErrorCorrectionLevel, version: u8) -> usize {
	match eclevel {
		ErrorCorrectionLevel::L => EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_L[version as usize - 1],
		ErrorCorrectionLevel::M => EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_M[version as usize - 1],
		ErrorCorrectionLevel::Q => EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_Q[version as usize - 1],
		ErrorCorrectionLevel::H => EC_CODEWORDS_PER_BLOCK_BY_EC_LEVEL_AND_VERSION_H[version as usize - 1],
	}
}

pub fn error_correction(codewords: &Vec<Vec<u8>>, eclevel: ErrorCorrectionLevel, version: u8) -> Vec<Vec<u8>> {
	let gplength = get_eclength(eclevel, version);
	#[cfg(feature = "rayon")]
	{
		let mut ecpool = Vec::new();
		codewords.par_iter().map(|cw_block| get_ec(cw_block, gplength)).collect_into_vec(&mut ecpool);
		ecpool
	}
	#[cfg(not(feature = "rayon"))]
	{
		codewords.iter().map(|cw_block| get_ec(cw_block, gplength)).collect()
	}
}

#[cfg(test)]
mod tests {
    use crate::ec::{ErrorCorrectionLevel, get_ec, get_eclength};

	#[test]
	fn get_eclength_test() {
		assert_eq!(get_eclength(ErrorCorrectionLevel::M, 1), 10);
	}

	#[test]
	fn get_ec_test() {
		assert_eq!(
			get_ec(&vec![32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17], 10), 
			vec![196, 35, 39, 119, 235, 215, 231, 226, 93, 23]
		);
	}
}