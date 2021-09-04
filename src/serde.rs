#![cfg(feature = "serde")]
#[cfg(feature = "rayon")]
use rayon_::iter::{IntoParallelIterator, ParallelIterator, IndexedParallelIterator};
use serde_::ser::{Serialize, Serializer, SerializeStruct};
use ndarray::{ArrayBase, Axis, ViewRepr, Dim};

use crate::{
	ec::ErrorCorrectionLevel,
	encode::EncodeMode,
	qr::QrMatrix,
};

impl QrMatrix {

	fn to_01_vec(&self) -> Vec<Vec<u8>> {
		/* Not using parallel even if serde is on. Don't make calculation too fragmented */
		let cvt_row = |row: ArrayBase<ViewRepr<&bool>, Dim<[usize; 1]>>| -> Vec<u8> {
			let mut v2 = Vec::new();
			for b in row.iter() {
				if *b {v2.push(1)} else {v2.push(0)}
			}
			v2
		};
		#[cfg(feature = "rayon")]
		{
			let mut v = Vec::new();
			IndexedParallelIterator::collect_into_vec(self.matrix.axis_iter(Axis(0)).into_par_iter().map(cvt_row), &mut v);
			v
		}
		#[cfg(not(feature = "rayon"))]
		{
			self.matrix.axis_iter(Axis(0)).map(cvt_row).collect()
		}
	}

}

impl Serialize for QrMatrix {

	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
			S: Serializer {
		let mut state = serializer.serialize_struct("QrMatrix", 4)?;
		state.serialize_field("version", &self.version)?;
		state.serialize_field("encode_mode", &self.encode_mode)?;
		state.serialize_field("error_correction_level", &self.error_correction_level)?;
		state.serialize_field("matrix", &self.to_01_vec())?;
		state.end()
	}

}

impl Serialize for ErrorCorrectionLevel {

	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
			S: Serializer {
		match *self {
			ErrorCorrectionLevel::L => serializer.serialize_unit_variant("ErrorCorrectionLevel", 0, "L"),
			ErrorCorrectionLevel::M => serializer.serialize_unit_variant("ErrorCorrectionLevel", 1, "M"),
			ErrorCorrectionLevel::Q => serializer.serialize_unit_variant("ErrorCorrectionLevel", 2, "Q"),
			ErrorCorrectionLevel::H => serializer.serialize_unit_variant("ErrorCorrectionLevel", 3, "M"),
		}
	}

}

impl Serialize for EncodeMode {

	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
			S: Serializer {
		match *self {
			EncodeMode::Numeric => serializer.serialize_unit_variant("EncodeMode", 0, "Numeric"),
			EncodeMode::Alphanumeric => serializer.serialize_unit_variant("EncodeMode", 1, "Alphanumeric"),
			EncodeMode::Byte => serializer.serialize_unit_variant("EncodeMode", 2, "Byte"),
			EncodeMode::Kanji => serializer.serialize_unit_variant("EncodeMode", 3, "Kanji"),
		}
	}

}