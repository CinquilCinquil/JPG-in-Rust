use std::collections::HashMap;
use image::GenericImageView;
use itertools::izip;

use crate::types::{Image, YCbCrColorSpace, Pixel, ImageInBlocks, ImageBlock, 
    HuffmanTree, HuffmanEncodedBlocks};
//use crate::decoder::{ycbcr_to_rgb};

pub fn encode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(img) => {
            println!("start colorspace_conversion");
            let crominance_values = colorspace_conversion(&img);

            println!("start split_into_blocks");
            let (width, height) = img.dimensions();
            let blocks = split_into_blocks(&crominance_values, width, height);

            println!("start discrete_cosine_transform");
            let dct_blocks = discrete_cosine_transform(blocks);

            println!("start quantization");
            let quantized_blocks = quantization(dct_blocks);

            println!("start statistical_enconding");
            let huffman_encoded_blocks = statistical_enconding(quantized_blocks);

            println!("start save_compressed");
            save_compressed(huffman_encoded_blocks, width, height);
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
pub fn colorspace_conversion(img: &Image) -> Vec<YCbCrColorSpace> {
    let pixels = img.pixels();
    let (w, h) = img.dimensions();
    
    let mut crominance_values = vec![(0, 0, 0); (w * h) as usize];
    
    for pixel in pixels {
        let i = (pixel.0 + pixel.1 * h) as usize;
        let r = pixel.2.0[0] as f64;
        let g = pixel.2.0[1] as f64;
        let b = pixel.2.0[2] as f64;
        
        // Fórmulas JPEG padrão
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let cb = 128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b;
        let cr = 128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b;
        
        crominance_values[i] = (
            y.clamp(0.0, 255.0).round() as u8,
            cb.clamp(0.0, 255.0).round() as u8,
            cr.clamp(0.0, 255.0).round() as u8,
        );
    }
    
    crominance_values
}

/* Step 2
    - Divide the Cb and Cr vectors into 2x2 blocks 
    - Make each of the 4 blocks the same value: The average between them
    - Recalculate the RGB values for the image
    - Return 8x8 blocks of the image in RGB
*/
pub fn split_into_blocks(ycbcr : &Vec<YCbCrColorSpace>, width : u32 , height: u32) -> ImageInBlocks<u8> {
    let width_usize = width as usize;
    let height_usize = height as usize;

    let y_image: ImageBlock<u8> = ycbcr.iter().map(|(y, _, _)| *y).collect();
    let mut cb_image: ImageBlock<u8> = ycbcr.iter().map(|(_, cb, _)| *cb).collect();
    let mut cr_image: ImageBlock<u8> = ycbcr.iter().map(|(_, _, cr)| *cr).collect();

    println!("height: {height}, width: {width}");

    let horizontal = if width % 8 == 0 {
        width / 8
    } else {
        (width + 7) / 8
    };
    let vertical = if height % 8 == 0 {
        height / 8
    } else {
        (height + 7) / 8
    };
    println!("horizon: {horizontal}, vertical: {vertical}");

    for h in (0..height_usize).step_by(2) {
        for w in (0..width_usize).step_by(2) {

            //Take the value of 2x2 blocks
            let (_, cb0, cr0) = ycbcr[h * width_usize + w];
            let (_, cb1, cr1) = ycbcr[h * width_usize + (w + 1)];
            let (_, cb2, cr2) = ycbcr[(h + 1) * width_usize + w];
            let (_, cb3, cr3) = ycbcr[(h + 1) * width_usize + (w + 1)];

            let avg_cb = ((cb0 as u16 + cb1 as u16 + cb2 as u16 + cb3 as u16) / 4) as u8;
            let avg_cr = ((cr0 as u16 + cr1 as u16 + cr2 as u16 + cr3 as u16) / 4) as u8;

            cb_image[h * width_usize + w] = avg_cb;
            cb_image[h * width_usize + (w + 1)] = avg_cb;
            cb_image[(h + 1) * width_usize + w] = avg_cb;
            cb_image[(h + 1) * width_usize + (w + 1)] = avg_cb;
            cr_image[h * width_usize + w] = avg_cr;
            cr_image[h * width_usize + (w + 1)] = avg_cr;
            cr_image[(h + 1) * width_usize + w] = avg_cr;
            cr_image[(h + 1) * width_usize + (w + 1)] = avg_cr;
            //cb.push(avg_cb);
            //cr.push(avg_cr);
        }
    }
    
    //let cb_image: ImageBlock<u8> = cb.iter().flat_map(|&chro_b| std::iter::repeat(chro_b).take(4)).collect();

    //let cr_image: ImageBlock<u8> = cr.iter().flat_map(|&chro_r| std::iter::repeat(chro_r).take(4)).collect();

    /*let mut y_block: ImageBlock<u8> = ImageBlock::with_capacity(y_image.len());
    let mut cb_block: ImageBlock<u8> = ImageBlock::with_capacity(y_image.len());
    let mut cr_block: ImageBlock<u8> = ImageBlock::with_capacity(y_image.len());

    //let ycbcr_iter = y_image.iter().zip(Cb_Image.iter().zip(Cr_Image.iter()));
    for (y, cb, cr) in izip!(y_image.iter(), cb_image.iter(), cr_image.iter()) {
        //let (r, g, b) = ycbcr_to_rgb(*y, *cb, *cr);
        y_block.push(*y);
        cb_block.push(*cb);
        cr_block.push(*cr);
    }

    /*let r_block = convert_in_blocks(&R, width, height);
    let g_block = convert_in_blocks(&G, width, height);
    let b_block = convert_in_blocks(&B, width, height);*/

    (convert_in_blocks(&y_block, width, height, horizontal, vertical), convert_in_blocks(&cb_block, width, height, horizontal, vertical), convert_in_blocks(&cr_block, width, height, horizontal, vertical))*/

    (convert_in_blocks(&y_image, width, height, horizontal, vertical), convert_in_blocks(&cb_image, width, height, horizontal, vertical), convert_in_blocks(&cr_image, width, height, horizontal, vertical))
}

/*pub fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y_f = y as f64;
    let cb_f = cb as f64 - 128.0;
    let cr_f = cr as f64 - 128.0;

    let r = y_f + 1.402 * cr_f;
    let g = y_f - 0.344136 * cb_f - 0.714136 * cr_f;
    let b = y_f + 1.772 * cb_f;

    (r as u8, b as u8, g as u8)
}*/

fn convert_in_blocks(channel: &ImageBlock<u8>, width : u32, height : u32, horizontal : u32, vertical : u32) -> Vec<ImageBlock<u8>> {
    
    let mut blocks = Vec::new();
    //let mut count = 0;
    //pub type ImageBlock<T> = Vec<T>;
    //pub type ImageInBlocks<T> = (Vec<ImageBlock<T>>, Vec<ImageBlock<T>>, Vec<ImageBlock<T>>);
    /*
    ╔══════════════════════════════════════╗
    ║  A00 A01 A02 A03 A04 A05 A06 A07 B08 ║
    ║  A10 A11 A12 A13 A14 A15 A16 A17 B18 ║
    ║  A20 A21 A22 A23 A24 A25 A26 A27 B28 ║
    ║  A30 A31 A32 A33 A34 A35 A36 A37 B38 ║
    ║  A40 A41 A42 A43 A44 A45 A46 A47 B48 ║
    ║  A50 A51 A52 A53 A54 A55 A56 A57 B58 ║
    ║  A60 A61 A62 A63 A64 A65 A66 A67 B68 ║
    ║  A70 A71 A72 A73 A74 A75 A76 A77 B78 ║
    ║  B80 B81 B82 B83 B84 B85 B86 B87 B88 ║
    ╚══════════════════════════════════════╝
    */
    for y_block in 0..vertical {
        for x_block in 0..horizontal {
            let mut block = Vec::with_capacity(64); 
            
            for i in 0..8 {
                let y_image = y_block * 8 + i;
                
                for j in 0..8 {
                    let x_image = x_block * 8 + j;
                    
                    let index = (y_image * width + x_image) as usize;
                    
                    if y_image < height && x_image < width {
                        block.push(channel[index]);
                    } 
                    else {
                        block.push(0); 
                    }
                }
            }
            /*if count == 0 {
                
            }*/
            if block.len() != 64 {
                for i in 0..8 {
                    for j in 0..8 {
                        let index = i * 8 + j;
                    print!(" {}", block[index]);
                    }
                    println!{""};
                }
            }
            //println!("{}", block.len());
            blocks.push(block);
            //count = 1;
        }
    }
    println!("{}", blocks.len());
    blocks
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
    let LUMINANCE_QUANTIZATION_TABLE: [u8; 64] = [
        4, 3, 4, 4, 4, 6, 11, 15,
        3, 3, 3, 4, 5, 8, 14, 19,
        3, 4, 4, 5, 8, 12, 16, 20,
        4, 5, 6, 7, 12, 14, 18, 20,
        6, 6, 9, 11, 14, 17, 21, 23,
        9, 12, 12, 18, 23, 22, 25, 21,
        11, 13, 15, 17, 21, 23, 25, 21,
        13, 12, 12, 13, 16, 19, 21, 21,
    ];
    let CHROMINANCE_QUANTIZATION_TABLE: [u8; 64] = [
        4, 4, 6, 10, 21, 21, 21, 21,
        4, 5, 6, 21, 21, 21, 21, 21,
        6, 6, 12, 21, 21, 21, 21, 21,
        10, 14, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 21,
        21, 21, 21, 21, 21, 21, 21, 21,
    ];

    let mut new_r: Vec<ImageBlock<i8>> = Vec::new();
    let mut new_g: Vec<ImageBlock<i8>> = Vec::new();
    let mut new_b: Vec<ImageBlock<i8>> = Vec::new();

    for i in 0..img_blocks.0.len() {
        let block_r = &img_blocks.0[i]; 
        let mut quantized_block_r = ImageBlock::new();

        let block_g = &img_blocks.1[i]; 
        let mut quantized_block_g = ImageBlock::new();

        let block_b = &img_blocks.2[i]; 
        let mut quantized_block_b = ImageBlock::new();
        
        for j in 0..block_r.len() {
            let quant_value_r = (block_r[j] / LUMINANCE_QUANTIZATION_TABLE[j] as f64).round() as i8;
            quantized_block_r.push(quant_value_r);

            let quant_value_g = (block_g[j] / CHROMINANCE_QUANTIZATION_TABLE[j] as f64).round() as i8;
            quantized_block_g.push(quant_value_g);

            let quant_value_b = (block_b[j] / CHROMINANCE_QUANTIZATION_TABLE[j] as f64).round() as i8;
            quantized_block_b.push(quant_value_b);
        }
        
        new_r.push(quantized_block_r);
        new_g.push(quantized_block_g);
        new_b.push(quantized_block_b);
    }
    //println!("{}", new_r.len());
    //println!("{}", new_g.len());
    //println!("{}", new_b.len());
    (new_r, new_g, new_b)
    //todo!()
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

                if i >= path_lengths.len()/2 && j == path_length - 1 {
                    let step_rotated = if y < x {-step} else {step};
                    x += step_rotated;
                    y += step_rotated;
                }
                else {
                    x -= step;
                    y += step;
                }

                x = x.clamp(0, 7);
                y = y.clamp(0, 7);
            }
        }
        //println!("zigzag values: {}", values.len());
        /*let mut count = 0;
        for value in &values {
            eprint!("Linha {}:", count);
            count += 1;
            for _ in 0..8 {
                eprint!(" {}", *value);
            }
            //print!(" {}", *value);
            eprintln!();
        }*/
        
        return values;
    }

    // Stores a sequence of integers as tuples of (interger, frequency)
    // Ex: 1, 1, 2, 0, 4, 0, 0 becomes (1, 2), (2, 1), (0, 1) (4, 1) (0, 2)
    fn run_length_enconding(values : Vec<i8>) -> Vec<(i8, i8)> {

        let mut run_length_values : Vec<(i8, i8)> = vec![];
        let mut n = 1;
        let lenght = values.len();
        //eprintln!("values length: {}", lenght);
        for i in 0..values.len() - 1 {
            /*eprintln!("values[{}]: {}, values[{} + 1]: {}", i, values[i], i, values[i + 1]);*/
            if values[i] == values[i + 1] {
                n += 1;
            }
            else {
                run_length_values.push((values[i], n));
                n = 1;
            }
        }
        if n > 1 {
            run_length_values.push((values[lenght - 1], n));
        }
        if values[lenght - 1] != values[lenght - 2] {
            run_length_values.push((values[lenght - 1], 1));
        } 
        //eprintln!("run_length_values: {}", run_length_values.len());
        run_length_values
    }

    fn huffman_enconding(run_length_values : Vec<(i8, i8)>) -> (Vec<String>, HuffmanTree) {
        let mut frequencies : HashMap<(i8, i8), i8> = HashMap::new();

        // Gathering frequencies
        //println!("{}", run_length_values.len());
        for value in &run_length_values {
            let entry = frequencies.entry(*value).or_insert(0);
            *entry += 1;
            let (number, frequency) = *value;
            //println!("(number, frequency): ({},{}) entry: {}", number, frequency, *entry);
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

        let msg = "(??) huffman_enconding: Unable to move first two elements of list with len() > 2";
        while nodes.len() > 2 {
            
            let combined_frequency = nodes[0].frequency + nodes[1].frequency;
            let new_node = HuffmanTree{
                value : (0, 0), /* this value indicates its not a leaf */
                frequency : combined_frequency,
                children : vec![nodes.pop().expect(msg), nodes.pop().expect(msg)] // nodes[0] and nodes[1]
            };

            // Find correct position in list
            for i in 0..nodes.len() {
                if combined_frequency <= nodes[i].frequency {
                    nodes.insert(i, new_node);
                    break;
                }
                else if i == nodes.len() - 1 {
                    nodes.push(new_node);
                    break;
                }
            }
        }

        let root = HuffmanTree{value : (0, 0), frequency : 0, children : nodes};

        // Registering encoded values for every node on the tree
        let mut encoded_values : HashMap<(i8, i8), String> = HashMap::new();
        fn walk_tree(node : &HuffmanTree, path : String, encoded_values : &mut HashMap<(i8, i8), String>) {
            if node.value != (0, 0) {
                encoded_values.insert(node.value, path.clone());
            }
            for i in 0..node.children.len() {
                let new_path = format!("{}{}", path, if i % 2 == 0 {"0"} else {"1"});
                walk_tree(&(node.children[i]), new_path, encoded_values);
            }
        }

        walk_tree(&root, String::new(), &mut encoded_values);

        // Replacing run length values with encoded ones
        let mut new_run_length_values : Vec<String> = vec![];
        for value in &run_length_values {
            new_run_length_values.push(encoded_values[value].clone());
        }

        return (new_run_length_values, root);
    }

    // This function applies the previously defined functions in all blocks of 'img_blocks'
    fn final_func(img_blocks : Vec<ImageBlock<i8>>) -> Vec<(Vec<String>, HuffmanTree)> {
        let mut huffman_encoded_blocks : Vec<(Vec<String>, HuffmanTree)> = vec![];

        for block in img_blocks {
            /*for value in &block {
                for _ in 0..8 {
                    print!(" {}", *value);
                }
                println!();
            }*/
            
            huffman_encoded_blocks.push(
                huffman_enconding(run_length_enconding(get_values_in_zigzag(block)))
            );
        }

        return huffman_encoded_blocks;
    }
    //println!("{}", img_blocks.0.len());
    return (final_func(img_blocks.0), final_func(img_blocks.1), final_func(img_blocks.2));
}

// Step 6
pub fn save_compressed(huffman_encoded_blocks : HuffmanEncodedBlocks, width: u32, height: u32) {

    let file_name = "result.compressed";

    fn write_channel(huffman_encoded_blocks : Vec<(Vec<String>, HuffmanTree)>) -> String {
        let mut content = String::new();

        /* Channel Format:
            (number of words)(words)(tree) ...(repeat)
        */
        content.push_str(" BEGIN_OF_CHANNEL ");
        fn write_tree(node : &HuffmanTree, content : &mut String) {
            if node.value != (0, 0) {content.push_str(&format!("{:?}", node.value));}
            for i in 0..node.children.len() {
                content.push_str(if i % 2 == 0 {"0"} else {"1"});
                write_tree(&(node.children[i]), content);
            }
        }

        for pair in huffman_encoded_blocks {
            let (block, tree) = pair;

            // Writting number in binary
            content.push_str(" BEGIN_OF_N_WORDS ");
            let mut n_words = format!("{:b}", block.len() as u8);
            for _ in 0..(8 - n_words.len()) {n_words.insert_str(0, "0");}

            content.push_str(&format!("{n_words}"));
            content.push_str(" END_OF_N_WORDS ");

            for word in block {
                content.push_str(" ");
                content.push_str(" BEGIN_OF_WORD ");
                content.push_str(&word);
                content.push_str(" END_OF_WORD ");
            }
            content.push_str(" ");
            content.push_str(" BEGIN_OF_TREE ");
            write_tree(&tree, &mut content);
            content.push_str(" END_OF_TREE ");
            content.push_str(" ");
            
        }
        content.push_str(" END_OF_CHANNEL ");
        return content;
    }
    let mut content = String::new();
    
    content.push_str(&format!("WIDTH {}\n", width));
    content.push_str(&format!("HEIGHT {}\n", height));
    //content.push_str("\n");
    
    content.push_str(&write_channel(huffman_encoded_blocks.0));
    content.push_str(&write_channel(huffman_encoded_blocks.1));
    content.push_str(&write_channel(huffman_encoded_blocks.2));

    match std::fs::write(file_name, content) {
        Ok(_) => println!("Saved file {file_name} Successfully."),
        Err(_) => println!("Error saving {file_name}")
    }
}