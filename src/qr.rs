use crate::{
	ec::{
		ErrorCorrectionLevel,
		error_correction
	},
	encode::{
		EncodeMode,
		best_encode_mode,
		test_encode_possible,
		compile_pool,
		encode_text
	},
	matrix::make_matrix,
	version::{
		smallest_version_by_encoding_and_eclevel,
		test_version_possible,
	},
};

type BitMatrix = ndarray::Array2<bool>;

pub struct QrMatrix {
	pub version: u8,
	pub encode_mode: EncodeMode,
	pub error_correction_level: ErrorCorrectionLevel,
	pub matrix: BitMatrix,
}

pub fn make_qr(text: &str, preferred_encode_mode: Option<EncodeMode>, error_correction_level: ErrorCorrectionLevel, preferred_version: Option<u8>) -> Result<QrMatrix, &'static str> {
	/* Test encode mode.
	If preferred_encode_mode is specified, test to see if it is possible.
	Otherwise, find a suitable encoding for it. */
	let encode_mode: EncodeMode;
	if let Some(pref_enc_mode) = preferred_encode_mode {
		if !test_encode_possible(text, pref_enc_mode) {
			return Err("Cannot encode with the specified encode mode");
		} else {
			encode_mode = pref_enc_mode;
		}
	} else {
		let test_result = best_encode_mode(text);
		if test_result.is_none() {
			return Err("Cannot encode this text")
		} else {
			encode_mode = test_result.unwrap();
		}
	}

	/* Test QR code version.
	If preferred_version is specified, test to see if it is possible.
	Otherwise, find the minimal version suitable for the text and its encoding. */
	let version: u8;
	if let Some(pref_ver) = preferred_version {
		if !test_version_possible(text.len(), encode_mode, error_correction_level, pref_ver) {
			return Err("Cannot encode with the specified QR code version");
		} else {
			version = pref_ver;
		}
	} else { 
		let test_result = smallest_version_by_encoding_and_eclevel(text.len(), encode_mode, error_correction_level);
		if test_result.is_none() {
			return Err("No suitable QR code version for this text")
		} else {
			version = test_result.unwrap();
		}
	}

	/* Generating bitstream and breaking them into different groups */
	let codepool = unsafe {
		encode_text(text, encode_mode, error_correction_level, version)
	};
	let ecpool = error_correction(&codepool, error_correction_level, version);
	
	/* Interleaving groups into a single bitstream */
	let data_box = compile_pool(&codepool, &ecpool, error_correction_level, version);

	/* Writing the bitstream into the QR code matrix */
	let matrix: BitMatrix = make_matrix(&data_box, error_correction_level, version);

	Ok(QrMatrix{version, encode_mode, error_correction_level, matrix})
}