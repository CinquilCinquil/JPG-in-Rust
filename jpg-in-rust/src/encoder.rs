use image::GenericImageView;

use crate::types::{Image, YCbCrColorSpace, Pixel, ImageInBlocks, ImageBlock};

pub fn encode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(mut img) => {
            let crominance_values = colorspace_conversion(&img);

            let blocks = split_into_blocks(&crominance_values);

            let dct_blocks = discrete_cosine_transform(blocks);

            quantization(dct_blocks);

            statistical_enconding(&mut img);

            save_image(&mut img);
        }
        Err(error) => println!("{}", error),
    }
}

// Step 0
pub fn pre_processing(filepath : &str) -> Result<Image, String> {
    match image::open(filepath) {
        Ok(img) => Ok(img),
        Err(error) => Err(format!(
            "We can't open the image: {}. Try again.",
            error
        )),
    }
}

/* Step 1
    - Convert from RGB colorspace into Y Cb Cr
*/
pub fn colorspace_conversion(img : &Image) -> Vec<YCbCrColorSpace> {
    let pixels = img.pixels();
    let (w, h) = img.dimensions();

    let mut crominance_values : Vec<YCbCrColorSpace> = vec![(0, 0, 0); (w * h) as usize];
    let red = |pixel : Pixel| {pixel.2.0[0] as f64};
    let blue = |pixel : Pixel| {pixel.2.0[1] as f64};
    let green = |pixel : Pixel| {pixel.2.0[2] as f64};

    for pixel in pixels {
        let i = (pixel.0 + pixel.1 * h) as usize;
        crominance_values[i] = (
        /* Y */     (0.299 * red(pixel) + 0.587 * green(pixel) + 0.114 * blue(pixel)) as u8,
        /* Cb */    (-0.1687 * red(pixel) - 0.3313 * green(pixel) + 0.5 * blue(pixel) + 128.0) as u8,
        /* Cr */    (0.5 * red(pixel) - 0.4187 * green(pixel) - 0.0813 * blue(pixel) + 128.0) as u8
        );
    }

    return crominance_values;
}

/* Step 2
    - Divide the Cb and Cr vectors into 2x2 blocks
    - Make each of the 4 blocks the same value: The average between them
    - Recalculate the RGB values for the image
    - Return 8x8 blocks of the image in RGB
*/
pub fn split_into_blocks(img : &Vec<YCbCrColorSpace>) -> ImageInBlocks<u8> {
    todo!()
}

// Step 3
pub fn discrete_cosine_transform(img_blocks : ImageInBlocks<u8>) -> ImageInBlocks<f64> {

    fn do_dct(blocks : Vec<ImageBlock<u8>>) -> Vec<ImageBlock<f64>> {
        let mut new_blocks : Vec<ImageBlock<f64>> = vec![];
        let alpha_constant = 1.0 / 2.0_f64.sqrt();
        let pi = std::f64::consts::PI;

        for block in blocks {

            // Applying transformations block by block

            let mut new_block : ImageBlock<f64> = vec![];
            for i in 0..8 { for j in 0..8 {

                // Shifting values from [0, 255] to [-128, 127]
                let mut value = block[i + j * 8] as f64 - 128.0;

                // Calculating DCT matrix
                let alpha = |i| { if i == 0 {alpha_constant} else {1.0} }; /* normalization function */
                let _g = |i, j| {
                    let mut sum = 0.0;
                    for x in 0..8 { for y in 0..8 {
                        let part1 = ((2.0 * x as f64 + 1.0) * i as f64 * pi / 16.0).cos();
                        let part2 = ((2.0 * y as f64 + 1.0) * j as f64 * pi / 16.0).cos();
                        sum += value * part1 * part2;
                    }}
                    return sum;
                };

                value = (0.25) * alpha(i) * alpha(j) * _g(i, j);
                new_block.push(value);
            }}

            new_blocks.push(new_block);
        }

        return new_blocks;
    }

    return (do_dct(img_blocks.0), do_dct(img_blocks.1), do_dct(img_blocks.2));
}

// Step 4
pub fn quantization(img_blocks : ImageInBlocks<f64>) {
    todo!()
}

// Step 5
pub fn statistical_enconding(img : &mut Image) {
    todo!()
}

// Step 6
pub fn save_image(img : &mut Image) {
    todo!()
}