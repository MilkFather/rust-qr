pub use crate::{
	ec::ErrorCorrectionLevel,
	encode::{
		EncodeMode,
		test_encode_mode,
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