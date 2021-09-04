use std::cmp::{min, max};

use bitvec::prelude::*;
use encoding_rs::{WINDOWS_1252, SHIFT_JIS};

#[cfg(feature = "rayon")]
use rayon_::prelude::*;

use crate::{
	ec::{ErrorCorrectionLevel, get_eclength},
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EncodeMode {
	Numeric,
	Alphanumeric,
	Byte,
	Kanji,
}

const ALPHANUMERIC_TABLE: [char; 45] = [ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ', '$', '%', '*', '+', '-', '.', '/', ':' ];
const NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_L: [(u8, u8); 40] = [ (1, 0), (1, 0), (1, 0), (1, 0), (1, 0), (2, 0), (2, 0), (2, 0), (2, 0), (2, 2), (4, 0), (2, 2), (4, 0), (3, 1), (5, 1), (5, 1), (1, 5), (5, 1), (3, 4), (3, 5), (4, 4), (2, 7), (4, 5), (6, 4), (8, 4), (10, 2), (8, 4), (3, 10), (7, 7), (5, 10), (13, 3), (17, 0), (17, 1), (13, 6), (12, 7), (6, 14), (17, 4), (4, 18), (20, 4), (19, 6) ];
const NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_M: [(u8, u8); 40] = [ (1, 0), (1, 0), (1, 0), (2, 0), (2, 0), (4, 0), (4, 0), (2, 2), (3, 2), (4, 1), (1, 4), (6, 2), (8, 1), (4, 5), (5, 5), (7, 3), (10, 1), (9, 4), (3, 11), (3, 13), (17, 0), (17, 0), (4, 14), (6, 14), (8, 13), (19, 4), (22, 3), (3, 23), (21, 7), (19, 10), (2, 29), (10, 23), (14, 21), (14, 23), (12, 26), (6, 34), (29, 14), (13, 32), (40, 7), (18, 31) ];
const NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_Q: [(u8, u8); 40] = [ (1, 0), (1, 0), (2, 0), (2, 0), (2, 2), (4, 0), (2, 4), (4, 2), (4, 4), (6, 2), (4, 4), (4, 6), (8, 4), (11, 5), (5, 7), (15, 2), (1, 15), (17, 1), (17, 4), (15, 5), (17, 6), (7, 16), (11, 14), (11, 16), (7, 22), (28, 6), (8, 26), (4, 31), (1, 37), (15, 25), (42, 1), (10, 35), (29, 19), (44, 7), (39, 14), (46, 10), (49, 10), (48, 14), (43, 22), (34, 34) ];
const NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_H: [(u8, u8); 40] = [ (1, 0), (1, 0), (2, 0), (4, 0), (2, 2), (4, 0), (4, 1), (4, 2), (4, 4), (6, 2), (3, 8), (7, 4), (12, 4), (11, 5), (11, 7), (3, 13), (2, 17), (2, 19), (9, 16), (15, 10), (19, 6), (34, 0), (16, 14), (30, 2), (22, 13), (33, 4), (12, 28), (11, 31), (19, 26), (23, 25), (23, 28), (19, 35), (11, 46), (59, 1), (22, 41), (2, 64), (24, 46), (42, 32), (10, 67), (20, 61) ];
const NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_L: [(u8, u8); 40] = [ (19, 0), (34, 0), (55, 0), (80, 0), (108, 0), (68, 0), (78, 0), (97, 0), (116, 0), (68, 69), (81, 0), (92, 93), (107, 0), (115, 116), (87, 88), (98, 99), (107, 108), (120, 121), (113, 114), (107, 108), (116, 117), (111, 112), (121, 122), (117, 118), (106, 107), (114, 115), (122, 123), (117, 118), (116, 117), (115, 116), (115, 116), (115, 0), (115, 116), (115, 116), (121, 122), (121, 122), (122, 123), (122, 123), (117, 118), (118, 119) ];
const NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_M: [(u8, u8); 40] = [ (16, 0), (28, 0), (44, 0), (32, 0), (43, 0), (27, 0), (31, 0), (38, 39), (36, 37), (43, 44), (50, 51), (36, 37), (37, 38), (40, 41), (41, 42), (45, 46), (46, 47), (43, 44), (44, 45), (41, 42), (42, 0), (46, 0), (47, 48), (45, 46), (47, 48), (46, 47), (45, 46), (45, 46), (45, 46), (47, 48), (46, 47), (46, 47), (46, 47), (46, 47), (47, 48), (47, 48), (46, 47), (46, 47), (47, 48), (47, 48) ];
const NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_Q: [(u8, u8); 40] = [ (13, 0), (22, 0), (17, 0), (24, 0), (15, 16), (19, 0), (14, 15), (18, 19), (16, 17), (19, 20), (22, 23), (20, 21), (20, 21), (16, 17), (24, 25), (19, 20), (22, 23), (22, 23), (21, 22), (24, 25), (22, 23), (24, 25), (24, 25), (24, 25), (24, 25), (22, 23), (23, 24), (24, 25), (23, 24), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25), (24, 25) ];
const NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_H: [(u8, u8); 40] = [ (9, 0), (16, 0), (13, 0), (9, 0), (11, 12), (15, 0), (13, 14), (14, 15), (12, 13), (15, 16), (12, 13), (14, 15), (11, 12), (12, 13), (12, 13), (15, 16), (14, 15), (14, 15), (13, 14), (15, 16), (16, 17), (13, 0), (15, 16), (16, 17), (15, 16), (16, 17), (15, 16), (15, 16), (15, 16), (15, 16), (15, 16), (15, 16), (15, 16), (16, 17), (15, 16), (15, 16), (15, 16), (15, 16), (15, 16), (15, 16) ];

pub fn best_encode_mode(text: &str) -> Option<EncodeMode> {
	// Test in order
	if test_encode_possible(text, EncodeMode::Numeric) { return Some(EncodeMode::Numeric); }
	else if test_encode_possible(text, EncodeMode::Alphanumeric) { return Some(EncodeMode::Alphanumeric); }
	else if test_encode_possible(text, EncodeMode::Byte) { return Some(EncodeMode::Byte); }
	else if test_encode_possible(text, EncodeMode::Kanji) { return Some(EncodeMode::Kanji); }
	else { return None; }
}

pub fn test_encode_possible(text: &str, encoding: EncodeMode) -> bool {
	let numeric_detect = |c: char| -> bool {
		return c.is_digit(10);
	};
	let alphanumeric_detect = |c: char| -> bool {
		return ALPHANUMERIC_TABLE.iter().position(|&x| x == c).is_some();
	};
	let byte_detect = |c: char| -> bool {
		let mut buf = [0; 4];
		let (encvec, _, err) = WINDOWS_1252.encode(char::encode_utf8(c, &mut buf));
		return !err && !(0x80_u8 <= encvec[0] && encvec[0] <= 0x9f_u8) && !(encvec[0] <= 0x1f_u8);
	};
	let kanji_detect = |c: char| -> bool {
		let mut buf = [0; 4];
		let (encvec, _enc, err) = SHIFT_JIS.encode(char::encode_utf8(c, &mut buf));
		if err || encvec.len() != 2 { return false; }
		let enc: u16 = ((encvec[0] as u16) << 8) | (encvec[1] as u16);
		return ((0x8140_u16 <= enc) && (enc <= 0x9ffc_u16)) || ((0xe040_u16 <= enc) && (enc <= 0xebbf_u16));
	};

	#[cfg(feature = "rayon")]
	{
		match encoding {
			EncodeMode::Numeric => text.par_chars().map(numeric_detect).all(|x| x),
			EncodeMode::Alphanumeric => text.par_chars().map(alphanumeric_detect).all(|x| x),
			EncodeMode::Byte => text.par_chars().map(byte_detect).all(|x| x),
			EncodeMode::Kanji => text.par_chars().map(kanji_detect).all(|x| x)
		}
	}
	#[cfg(not(feature = "rayon"))]
	{
		match encoding {
			EncodeMode::Numeric => text.chars().map(numeric_detect).all(|x| x),
			EncodeMode::Alphanumeric => text.chars().map(alphanumeric_detect).all(|x| x),
			EncodeMode::Byte => text.chars().map(byte_detect).all(|x| x),
			EncodeMode::Kanji => text.chars().map(kanji_detect).all(|x| x)
		}
	}
}

unsafe fn encode_numeric(text: &str, bb: &mut BitBox<Msb0, u8>, offset: usize) -> usize {
	let mut written: usize = 0;
	let mut iter = text.chars();
	let mut stored = 0;
	let mut stored_dig = 0;
	loop {
		let t_dig = iter.next();
		if t_dig.is_none() { break }
		let t_dig = t_dig.unwrap().to_digit(10).unwrap();
		stored = stored * 10 + t_dig;
		stored_dig += 1;
		if stored_dig >= 3 {
			if stored < 10 { bb[offset + written..offset + written + 4].store_be(stored); written += 4; }
			else if stored < 100 { bb[offset + written..offset + written + 7].store_be(stored); written += 7; }
			else { bb[offset + written..offset + written + 10].store_be(stored); written += 10; }
			stored = 0;
			stored_dig = 0;
		}
	}
	if stored_dig >= 1 {
		if stored < 10 { bb[offset + written..offset + written + 4].store_be(stored); written += 4; }
		else if stored < 100 { bb[offset + written..offset + written + 7].store_be(stored); written += 7; }
		else { bb[offset + written..offset + written + 10].store_be(stored); written += 10; }
	}
	written
}

unsafe fn encode_alphanumeric(text: &str, bb: &mut BitBox<Msb0, u8>, offset: usize) -> usize {
	let mut written: usize = 0;
	let mut iter = text.chars();
	loop {
		let t_c1 = iter.next();
		let t_c2 = iter.next();

		if t_c1.is_none() { break }
		else if t_c2.is_none() { 
			let t_c1 = ALPHANUMERIC_TABLE.iter().position(|&x| x == t_c1.unwrap()).unwrap();
			bb[offset + written..offset + written + 6].store_be(t_c1);
			written += 6;
			break;
		} else {
			let t_c1 = ALPHANUMERIC_TABLE.iter().position(|&x| x == t_c1.unwrap()).unwrap();
			let t_c2 = ALPHANUMERIC_TABLE.iter().position(|&x| x == t_c2.unwrap()).unwrap();
			bb[offset + written..offset + written + 11].store_be(t_c1 * 45 + t_c2);
			written += 11;
		}
	}
	written
}

unsafe fn encode_byte(text: &str, bb: &mut BitBox<Msb0, u8>, offset: usize) -> usize {
	let mut written: usize = 0;
	let (encvec, _, _) = WINDOWS_1252.encode(text);
	for code in encvec.iter() {
		bb[offset + written..offset + written + 8].store_be(*code);
		written += 8;
	}
	written
}

unsafe fn encode_kanji(text: &str, bb: &mut BitBox<Msb0, u8>, offset: usize) -> usize {
	let mut written: usize = 0;
	for c in text.chars() {
		let c_str = c.to_string();
		let (encvec, _, _) = SHIFT_JIS.encode(c_str.as_str());
		let mut enc: u16 = ((encvec[0] as u16) << 8) | (encvec[1] as u16);
		if (0x8140_u16 <= enc) && (enc <= 0x9ffc_u16) { enc = enc - 0x8140_u16 }
		else if (0xe040_u16 <= enc) && (enc <= 0xebbf_u16) { enc = enc - 0xc140_u16 }
		else { panic!("Invalid codepoint while encoding kanji. Did you perform encoding check beforehand?") }
		let comp = ((enc & 0xff00) >> 8) * 0xc0_u16 + (enc & 0x00ff_u16);
		bb[offset + written..offset + written + 13].store_be(comp);
		written += 13;
	}
	written
}

unsafe fn encode_textmode(bv: &mut BitBox<Msb0, u8>, encode_mode: EncodeMode) {
	match encode_mode {
		EncodeMode::Numeric => { bv[..4].store_be(0b_0001_u8) },
		EncodeMode::Alphanumeric => { bv[..4].store_be(0b_0010_u8) },
		EncodeMode::Byte => { bv[..4].store_be(0b_0100_u8) },
		EncodeMode::Kanji => { bv[..4].store_be(0b_1000_u8) }
	}
}

fn get_len_size(encode_mode: EncodeMode, version: u8) -> usize {
	match encode_mode {
		EncodeMode::Numeric => { if version <= 9 { 10 } else if version <= 26 { 12 } else { 14 } },
		EncodeMode::Alphanumeric => { if version <= 9 { 9 } else if version <= 26 { 11 } else { 13 } },
		EncodeMode::Byte => { if version <= 9 { 8 } else if version <= 26 { 16 } else { 16 } },
		EncodeMode::Kanji => { if version <= 9 { 8 } else if version <= 26 { 10 } else { 12 } }
	}
}

fn get_block_and_codewords_count(eclevel: ErrorCorrectionLevel, version: u8) -> ((u8, u8), (u8, u8)) {
	match eclevel {
		ErrorCorrectionLevel::L => { (NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_L[version as usize - 1], NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_L[version as usize - 1]) },
		ErrorCorrectionLevel::M => { (NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_M[version as usize - 1], NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_M[version as usize - 1]) },
		ErrorCorrectionLevel::Q => { (NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_Q[version as usize - 1], NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_Q[version as usize - 1]) },
		ErrorCorrectionLevel::H => { (NUM_OF_BLOCKS_IN_GROUPS_BY_VERSION_H[version as usize - 1], NUM_OF_CODEWORDS_IN_BLOCKS_BY_VERSION_H[version as usize - 1]) }
	}
}

pub unsafe fn encode_text(text: &str, encode_mode: EncodeMode, eclevel: ErrorCorrectionLevel, version: u8) -> Vec<Vec<u8>> {
	let ((block_in_g1, block_in_g2), (codew_in_b1, codew_in_b2)) = get_block_and_codewords_count(eclevel, version);
	let total_bits: usize = 8 * ((block_in_g1 as usize) * (codew_in_b1 as usize) + (block_in_g2 as usize) * (codew_in_b2 as usize));
	let mut bv = bitbox![Msb0, u8; 0; total_bits as usize];
	let mut idxtrack: usize = 0;

	encode_textmode(&mut bv, encode_mode);
	idxtrack += 4;
	bv[idxtrack..idxtrack + get_len_size(encode_mode, version)].store_be(text.chars().count() as u32);
	idxtrack += get_len_size(encode_mode, version);

	let encode_size = {
		match encode_mode {
			EncodeMode::Numeric => { encode_numeric(text, &mut bv, idxtrack) },
			EncodeMode::Alphanumeric => { encode_alphanumeric(text, &mut bv, idxtrack) },
			EncodeMode::Byte => { encode_byte(text, &mut bv, idxtrack) },
			EncodeMode::Kanji => { encode_kanji(text, &mut bv, idxtrack) }
		}
	};

	idxtrack += encode_size;
	bv[idxtrack..idxtrack + min(4, (total_bits as usize) - idxtrack)].store_be(0b_0000_u8);
	idxtrack += min(4, total_bits - idxtrack);

	if idxtrack % 8 != 0 {
		bv[idxtrack..idxtrack + 8 - (idxtrack % 8)].store_be(0b_00000000_u8);
		idxtrack += 8 - (idxtrack % 8);
	}
	while idxtrack + 1 < total_bits {
		bv[idxtrack..idxtrack + 8].store_be(0b_11101100_u8);
		idxtrack += 8;
		if idxtrack + 1 < total_bits {
			bv[idxtrack..idxtrack + 8].store_be(0b_00010001_u8);
			idxtrack += 8;
		}
	}

	let buf = bv.as_slice();
	let mut result: Vec<Vec<u8>> = Vec::new();
	let mut glb_idx: usize = 0;
	for _ in 0..block_in_g1 {
		let mut blockvec = Vec::<u8>::new();
		for _ in 0..codew_in_b1 {
			blockvec.push(buf[glb_idx]);
			glb_idx += 1;
		}
		result.push(blockvec);
	}
	for _ in 0..block_in_g2 {
		let mut blockvec = Vec::<u8>::new();
		for _ in 0..codew_in_b2 {
			blockvec.push(buf[glb_idx]);
			glb_idx += 1;
		}
		result.push(blockvec);
	}
	result
}

pub fn compile_pool(codeword_pool: &Vec<Vec<u8>>, ec_pool: &Vec<Vec<u8>>, eclevel: ErrorCorrectionLevel, version: u8) -> BitBox<Msb0, u8> {
	let mut interleaved: Vec<u8> = Vec::<u8>::new();
	let ((block_in_g1, block_in_g2), (codew_in_b1, codew_in_b2)) = get_block_and_codewords_count(eclevel, version);

	for i in 0..max(codew_in_b1, codew_in_b2) {
		for j in 0..block_in_g1+block_in_g2 {
			if i as usize >= codeword_pool[j as usize].len() {
				continue;
			}
			interleaved.push(codeword_pool[j as usize][i as usize]);
		}
	}
	for i in 0..get_eclength(eclevel, version) {
		for j in 0..block_in_g1+block_in_g2 {
			if i >= ec_pool[j as usize].len() {
				continue;
			}
			interleaved.push(ec_pool[j as usize][i]);
		}
	}

	let mut bvec = BitVec::<Msb0, u8>::from_vec(interleaved);
	let extra_bits: usize = {
		if version <= 1 { 0 }
		else if version <= 6 { 7 }
		else if version <= 13 { 0 }
		else if version <= 20 { 3 }
		else if version <= 27 { 4 }
		else if version <= 34 { 3 }
		else if version <= 40 { 0 }
		else { panic!("Version internal error") }
	};
	bvec.reserve_exact(extra_bits);
	for _ in 0..extra_bits {
		bvec.push(false);
	}
	bvec.into_boxed_bitslice()
}

/* For Testing */
#[cfg(test)]
mod tests {
    use super::{
		EncodeMode,
		best_encode_mode,
	};
	use crate::{
		encode::encode_text,
		ec::ErrorCorrectionLevel,
	};

	#[test]
	fn encode_mode_test() {
		assert_eq!(Some(EncodeMode::Numeric), best_encode_mode("8675309"));
		assert_eq!(Some(EncodeMode::Alphanumeric), best_encode_mode("HELLO WORLD"));
		assert_eq!(Some(EncodeMode::Byte), best_encode_mode("Hello, world!"));
		assert_eq!(Some(EncodeMode::Kanji), best_encode_mode("茗荷"));
		assert_eq!(None, best_encode_mode("茗荷12345"));
	}

	#[test]
	fn encode_alphanumeric_test() {
		assert_eq!(
			unsafe { encode_text("HELLO WORLD", EncodeMode::Alphanumeric, ErrorCorrectionLevel::Q, 1) }, 
			vec![
				vec![0b00100000, 0b01011011, 0b00001011, 0b01111000, 0b11010001, 0b01110010, 0b11011100, 0b01001101, 0b01000011, 0b01000000, 0b11101100, 0b00010001, 0b11101100]
			]
		);
		assert_eq!(
			unsafe { encode_text("HELLO WORLD", EncodeMode::Alphanumeric, ErrorCorrectionLevel::M, 1) }, 
			vec![
				vec![32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 17, 236, 17]
			]
		);
	}

	#[test]
	fn encode_kanji_test() {
		assert_eq!(
			unsafe { encode_text("茗荷", EncodeMode::Kanji, ErrorCorrectionLevel::M, 1) },
			vec![
				vec![128, 45, 85, 26, 92, 0, 236, 17, 236, 17, 236, 17, 236, 17, 236, 17]
			]
		);
	}
}
