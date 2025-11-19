use image::DynamicImage;
use image::Rgba;

pub type Image = DynamicImage;
pub type YCbCrColorSpace = (u8, u8, u8);
pub type ImageBlock<T> = Vec<T>;
pub type ImageInBlocks<T> = (Vec<ImageBlock<T>>, Vec<ImageBlock<T>>, Vec<ImageBlock<T>>);
pub type Pixel = (u32, u32, Rgba<u8>);

pub type HuffmanEncodedBlocks = (Vec<(Vec<u8>, HuffmanTree)>, Vec<(Vec<u8>, HuffmanTree)>, Vec<(Vec<u8>, HuffmanTree)>);

pub struct HuffmanTree {
    pub value : (i8, i8),
    pub frequency : i8,
    pub children : Vec<HuffmanTree>,
}