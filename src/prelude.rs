pub use crate::{
	ec::ErrorCorrectionLevel,
	encode::{
		EncodeMode,
		best_encode_mode,
		test_encode_possible,
	},
	qr::{
		QrMatrix,
		make_qr,
	},
	version::{
		smallest_version_by_encoding_and_eclevel,
		test_version_possible,
	},
};

#[cfg(feature = "serde")]
pub use crate::serde::*;