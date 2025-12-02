use image::DynamicImage;

pub type Image = DynamicImage;
pub type YCbCrColorSpace = (u8, u8, u8);
pub type ImageBlock<T> = Vec<T>;
pub type ImageInBlocks<T> = (Vec<ImageBlock<T>>, Vec<ImageBlock<T>>, Vec<ImageBlock<T>>);

pub type HuffmanEncodedBlocks = (Vec<(Vec<String>, HuffmanTree)>,
                                 Vec<(Vec<String>, HuffmanTree)>,
                                 Vec<(Vec<String>, HuffmanTree)>);

pub struct HuffmanTree {
    pub value : (i8, i8),
    pub frequency : i8,
    pub children : Vec<HuffmanTree>,
}