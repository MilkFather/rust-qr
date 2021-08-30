#![cfg(feature = "image")]

use image_::{
	ImageBuffer,
	Pixel,
	RgbImage,
	Rgb,
	GrayImage,
	Luma,
	imageops::{
		FilterType,
		resize,
	},
};

use crate::qr::QrMatrix;

#[derive(Clone, Copy, Debug)]
pub enum QrSizeConfig {
	TotalSize(u32),
	BitSize(u32),
}

impl QrMatrix {

	pub fn to_image<P>(&self, size: QrSizeConfig, foreground: P, background: P, silent_zone_size: u32) -> ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>
	where
			P: Pixel + 'static,
			P::Subpixel: 'static {
		let bitsize = match size {
			QrSizeConfig::TotalSize(t) => f64::ceil(t as f64 / (self.matrix.shape()[1] as f64 + 2.0 * silent_zone_size as f64)) as u32,
			QrSizeConfig::BitSize(b) => b,
		};
		let mut image = ImageBuffer::from_fn((self.matrix.shape()[1] as u32 + 2 * silent_zone_size) * bitsize, (self.matrix.shape()[0] as u32 + 2 * silent_zone_size) * bitsize, |_, _| background);
		for ((y, x), q) in self.matrix.indexed_iter() {
			if *q {
				for i in 0..bitsize {
					for j in 0..bitsize {
						image.put_pixel((x as u32 + silent_zone_size) * bitsize + i, (y as u32 + silent_zone_size) * bitsize + j, foreground);
					}
				}
			}
		}

		if let QrSizeConfig::TotalSize(total) = size {
			resize(&image, total, total, FilterType::Triangle)
		} else {
			image
		}
	}

	pub fn to_image_rgb(&self, size: QrSizeConfig, foreground: &[u8; 3], background: &[u8; 3], silent_zone_size: u32) -> RgbImage {
		let foreground_color = Rgb::<u8>(*foreground);
		let background_color = Rgb::<u8>(*background);
		self.to_image(size, foreground_color, background_color, silent_zone_size)
	}

	pub fn to_image_luma(&self, size: QrSizeConfig, foreground: &[u8; 1], background: &[u8; 1], silent_zone_size: u32) -> GrayImage {
		let foreground_color = Luma(*foreground);
		let background_color = Luma(*background);
		self.to_image(size, foreground_color, background_color, silent_zone_size)
	}

}