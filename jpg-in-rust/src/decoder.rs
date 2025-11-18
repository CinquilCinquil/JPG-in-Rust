use crate::encoder::{pre_processing};
use crate::types::{Image, YCbCrColorSpace, Pixel, ImageInBlocks, ImageBlock};

pub fn decode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(mut img) => {

            statistical_decoding();

            undo_dct();

            colorspace_conversion();

            merge_blocks();

            save_image();

        }
        Err(error) => println!("{}", error),
    }
}

pub fn statistical_decoding() {

}

pub fn undo_dct() {

}

pub fn colorspace_conversion() {

}

pub fn merge_blocks() {

}

/*
    Idealy, we would visualize the image at this point, but that would require
    programming a display, which is not our goal. Therefore, saving it as a png,
    for example, is enough to demonstrate the correct functioning of the algorithm.
*/
pub fn save_image() {

}