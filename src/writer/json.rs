#![cfg(feature = "json")]

use serde_json;

use crate::qr::QrMatrix;

impl QrMatrix {

	pub fn to_json(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}

}