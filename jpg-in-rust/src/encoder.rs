use std::collections::HashMap;
use image::GenericImageView;

use crate::types::{Image, YCbCrColorSpace, Pixel, ImageInBlocks, ImageBlock, 
    HuffmanTree, HuffmanEncodedBlocks};

pub fn encode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(mut img) => {
            let crominance_values = colorspace_conversion(&img);

            let blocks = split_into_blocks(&crominance_values);

            let dct_blocks = discrete_cosine_transform(blocks);

            let quantized_blocks = quantization(dct_blocks);

            statistical_enconding(quantized_blocks);

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
    
    let mut avg_Cb: Vec<u8> = Vec::new();
    let mut avg_Cr: Vec<u8> = Vec::new();
    for chunk in (*img).chunks(4) {
        let mut sum_Cb: u64 = 0;
        let mut sum_Cr: u64 = 0;
        for &(_, Cb, Cr) in chunk {
            sum_Cb += Cb;
            sum_Cr += Cr;
        }
        avg_Cb.push((sum_Cb/chunk.len()) as u8);
        avg_Cr.push((sum_Cr/chunk.len()) as u8);
    }
    let Y: Vec<u8> = img.iter().map(|(x, _, _)| *x).collect();

    //Recalculate here
    (Y, avg_Cb, avg_Cr)

    //todo!()
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
pub fn quantization(img_blocks : ImageInBlocks<f64>) -> ImageInBlocks<i8> {
    todo!()
}

// Step 5
pub fn statistical_enconding(img_blocks : ImageInBlocks<i8>) -> HuffmanEncodedBlocks {

    // Stores values in each block following a Zig Zag pattern
    fn get_values_in_zigzag(block : ImageBlock<i8>) -> Vec<i8> {
        let path_lengths = [1, 2, 3, 4, 5, 6, 7, 8, 7, 6, 5, 4, 3, 2, 1];
        let mut values : Vec<i8> = vec![];

        let mut x : i8 = 0;
        let mut y : i8 = 0;
        let mut step : i8 = 1;

        for i in 0..path_lengths.len() {
            let path_length = path_lengths[i];
            step *= -1;

            for j in 0..path_length {

                values.push(block[x as usize + 8 * y as usize]);

                if i < path_lengths.len()/2 {
                    x += step;
                    y -= step;
                }
                else if j == path_length - 1 {
                    let step_rotated = if x < y {-step} else {step};
                    x += step_rotated;
                    y += step_rotated;
                }

                x = x.clamp(0, 7);
                y = y.clamp(0, 7);
            }
        }

        return values;
    }

    // Stores a sequence of integers as tuples of (interger, frequency)
    // Ex: 1, 1, 2, 0, 4, 0, 0 becomes (1, 2), (2, 1), (0, 1) (4, 1) (0, 2)
    fn run_length_enconding(values : Vec<i8>) -> Vec<(i8, i8)> {

        let mut run_length_values : Vec<(i8, i8)> = vec![];
        let mut n = 1;

        for i in 0..values.len() - 1 {
            if values[i] == values[i + 1] {
                n += 1;
            }
            else {
                run_length_values.push((values[i], n));
                n = 1;
            }
        }

        return run_length_values;
    }

    fn huffman_enconding(run_length_values : Vec<(i8, i8)>) -> (Vec<u8>, HuffmanTree) {
        let mut frequencies : HashMap<(i8, i8), i8> = HashMap::new();

        // Gathering frequencies
        for value in &run_length_values {
            let entry = frequencies.entry(*value).or_insert(0);
            *entry += 1;
        }

        // Ordering frequencies
        let mut frequencies_vec: Vec<(&(i8, i8), &i8)> = frequencies.iter().collect();
        frequencies_vec.sort_by(|a, b| b.1.cmp(a.1));

        // Building Tree

        let mut nodes : Vec<HuffmanTree> = vec![];
        for value in frequencies_vec {
            let node_value = (value.0.0, value.0.1);
            let frequency = *value.1;
            nodes.push(HuffmanTree{value : node_value, frequency : frequency, children : vec![]});
        }

        let mut enconded_values : HashMap<(i8, i8), u8> = HashMap::new();
        while nodes.len() > 2 {
            todo!();
        }

        // Not actual node, just start of tree
        let mut root = HuffmanTree{value : (0, 0), frequency : 0, children : nodes};

        // Replacing values with new words
        let mut new_run_length_values : Vec<u8> = vec![];
        for value in run_length_values {
            new_run_length_values.push(enconded_values[&value]);
        }

        return (new_run_length_values, root);
    }

    // This function applies the previously defined functions in all blocks of 'img_blocks'
    fn final_func(img_blocks : Vec<ImageBlock<i8>>) -> Vec<(Vec<u8>, HuffmanTree)> {
        let mut huffman_encoded_blocks : Vec<(Vec<u8>, HuffmanTree)> = vec![];

        for block in img_blocks {
            let huffman_encoded_block = huffman_enconding(
                run_length_enconding(
                    get_values_in_zigzag(block)
                )
            );

            huffman_encoded_blocks.push(huffman_encoded_block);
        }

        return huffman_encoded_blocks;
    }

    return (final_func(img_blocks.0), final_func(img_blocks.1), final_func(img_blocks.2));
}

// Step 6
pub fn save_image(img : &mut Image) {
    todo!()
}