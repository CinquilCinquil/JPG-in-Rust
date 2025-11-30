use crate::encoder::{pre_processing};
use crate::types::{Image, YCbCrColorSpace, Pixel, ImageInBlocks, ImageBlock};

pub fn decode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(mut img) => {

            /*
            statistical_decoding();

            undo_dct();

            colorspace_conversion();

            merge_blocks();

            save_image();
            */

        }
        Err(error) => println!("{}", error),
    }
}

pub fn statistical_decoding() {

}

pub fn undo_dct() {

}

pub fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y_f = y as f64;
    let cb_f = cb as f64 - 128.0;
    let cr_f = cr as f64 - 128.0;

    let r = y_f + 1.402 * cr_f;
    let g = y_f - 0.344136 * cb_f - 0.714136 * cr_f;
    let b = y_f + 1.772 * cb_f;

    (r as u8, b as u8, g as u8)
}

pub fn colorspace_conversion(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y_f = y as f64;
    let cb_f = cb as f64 - 128.0;
    let cr_f = cr as f64 - 128.0;

    let r = y_f + 1.402 * cr_f;
    let g = y_f - 0.344136 * cb_f - 0.714136 * cr_f;
    let b = y_f + 1.772 * cb_f;

    (r as u8, b as u8, g as u8)
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