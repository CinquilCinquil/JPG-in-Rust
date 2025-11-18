use image::DynamicImage;
use image::Rgba;

pub type Image = DynamicImage;
pub type YCbCrColorSpace = (u8, u8, u8);
pub type ImageBlock<T> = Vec<T>;
pub type ImageInBlocks<T> = (Vec<ImageBlock<T>>, Vec<ImageBlock<T>>, Vec<ImageBlock<T>>);
pub type Pixel = (u32, u32, Rgba<u8>);