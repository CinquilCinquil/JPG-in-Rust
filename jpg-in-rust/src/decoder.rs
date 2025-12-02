use image::{ImageBuffer, RgbImage};
use std::collections::HashMap;

pub type ImageBlock<T> = Vec<T>;
pub type ImageInBlocks<T> = (Vec<ImageBlock<T>>, Vec<ImageBlock<T>>, Vec<ImageBlock<T>>);

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub symbol: Option<(i8, i8)>,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

pub fn decode(filepath: &str, output_path: &str) -> Result<(), String> {
    println!("Starting decoder...");
    
    //decode_grayscale_only(filepath, output_path)
    match pre_processing(filepath) {
        Ok(compressed_data) => {
            println!("Parsing compressed data...");
            
            let (width, height, huffman_blocks) = statistical_decoding(&compressed_data)?;
            
            println!("Inverse quantization and DCT...");
            let ycbcr_blocks = inverse_quantization_and_dct(huffman_blocks)?;
            
            println!("Merging blocks...");
            let ycbcr_image = merge_blocks(ycbcr_blocks, width, height)?;
            
            println!("Converting to RGB...");
            let rgb_image = ycbcr_to_rgb(ycbcr_image, width, height)?;
            
            println!("Saving image...");
            save_image(&rgb_image, output_path)?;
            
            println!("Decoding completed successfully!");
            Ok(())
        }
        Err(error) => {
            println!("Error: {}", error);
            Err(error)
        }
    }
}

// Step 0: Load compressed file
fn pre_processing(filepath: &str) -> Result<String, String> {
    std::fs::read_to_string(filepath)
        .map_err(|e| format!("Failed to read compressed file: {}", e))
}

// Step 1: Decode Huffman and RLE
fn statistical_decoding(content: &str) -> Result<(u32, u32, ImageInBlocks<i8>), String> {
    // Parse the file structure
    let ((width, height), y_blocks_data, cb_blocks_data, cr_blocks_data) = parse_file(content)?;
    
    // Decode each channel
    let y_blocks = decode_channel(y_blocks_data)?;
    let cb_blocks = decode_channel(cb_blocks_data)?;
    let cr_blocks = decode_channel(cr_blocks_data)?;
    
    Ok((width, height, (y_blocks, cb_blocks, cr_blocks)))
}

// Step 2: Inverse quantization and inverse DCT
fn inverse_quantization_and_dct(blocks: ImageInBlocks<i8>) -> Result<ImageInBlocks<u8>, String> {
    let (y_blocks, cb_blocks, cr_blocks) = blocks;
    
    let y_decoded = process_channel_blocks(y_blocks, true)?;
    let cb_decoded = process_channel_blocks(cb_blocks, false)?;
    let cr_decoded = process_channel_blocks(cr_blocks, false)?;
    
    Ok((y_decoded, cb_decoded, cr_decoded))
}

fn upsample_chroma(small_pixels: &[u8], small_width: u32, small_height: u32, 
                   target_width: u32, target_height: u32) -> Vec<u8> {
    let mut result = vec![0u8; (target_width * target_height) as usize];
    
    for y in 0..target_height as usize {
        for x in 0..target_width as usize {
            // Mapear para a resolução menor
            let src_x = x / 2;
            let src_y = y / 2;
            
            let src_idx = src_y * small_width as usize + src_x;
            
            // Clamp para não ultrapassar bounds
            let src_idx = src_idx.min(small_pixels.len() - 1);
            
            result[y * target_width as usize + x] = small_pixels[src_idx];
        }
    }
    
    result
}

// Step 3: Convert from blocks to full image arrays
fn merge_blocks(blocks: ImageInBlocks<u8>, width: u32, height: u32) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
    let (y_blocks, cb_blocks, cr_blocks) = blocks;
    
    println!("📊 Merge blocks: Y={}, Cb={}, Cr={}", 
             y_blocks.len(), cb_blocks.len(), cr_blocks.len());
    
    println!("=== DEBUG MERGE BLOCKS ===");
    println!("Dimensões alvo: {}x{}", width, height);
    println!("Número de blocos - Y: {}, Cb: {}, Cr: {}", 
             y_blocks.len(), cb_blocks.len(), cr_blocks.len());
    
    // Calcular quantos blocos deveríamos ter
    let y_blocks_expected = ((width + 7) / 8) * ((height + 7) / 8);
    let chroma_width = (width + 1) / 2;
    let chroma_height = (height + 1) / 2;
    let cbcr_blocks_expected = ((chroma_width + 7) / 8) * ((chroma_height + 7) / 8);
    
    println!("Blocos esperados - Y: {}, Cb/Cr: {}", 
             y_blocks_expected, cbcr_blocks_expected);
    println!("Blocos encontrados - Y: {}, Cb: {}, Cr: {}", 
             y_blocks.len(), cb_blocks.len(), cr_blocks.len());
    
    // 1. Processar Y (resolução completa)
    let y_pixels = blocks_to_pixels(y_blocks, width, height, true);
    
    /*// 2. Processar Cb/Cr (METADE da resolução)
    let chroma_width = (width + 1) / 2;
    let chroma_height = (height + 1) / 2;*/
    
    println!("🔍 Chroma resolution: {}x{} (original: {}x{})", 
             chroma_width, chroma_height, width, height);
    
    /*let cb_small = blocks_to_pixels(cb_blocks, chroma_width, chroma_height, false);
    let cr_small = blocks_to_pixels(cr_blocks, chroma_width, chroma_height, false);*/
    
    // 3. Upsample 2x
    let cb_pixels = blocks_to_pixels(cb_blocks, width, height, false);
    let cr_pixels = blocks_to_pixels(cr_blocks, width, height, false);
    
    Ok((y_pixels, cb_pixels, cr_pixels))
}

// Step 4: Convert YCbCr to RGB
fn ycbcr_to_rgb(ycbcr: (Vec<u8>, Vec<u8>, Vec<u8>), width: u32, height: u32) -> Result<RgbImage, String> {
    let (y_pixels, cb_pixels, cr_pixels) = ycbcr;
    
    println!("Y pixels: {}, Cb pixels: {}, Cr pixels: {}", y_pixels.len(), cb_pixels.len(), cr_pixels.len());
    if y_pixels.len() != cb_pixels.len() || y_pixels.len() != cr_pixels.len() {
        return Err("Channel sizes don't match".to_string());
    }
    
    let pixel_count = y_pixels.len();
    /*let width = (pixel_count as f64).sqrt().ceil() as u32;
    let height = ((pixel_count + width as usize - 1) / width as usize) as u32;*/
    
    let mut img = RgbImage::new(width, height);
    
    for i in 0..pixel_count {
        let x = (i % width as usize) as u32;
        let y = (i / width as usize) as u32;
        
        if x < width && y < height {
            let y_val = y_pixels[i] as f64;
            let cb_val = cb_pixels[i] as f64;
            let cr_val = cr_pixels[i] as f64;
            
            // YCbCr to RGB conversion
            let r = (y_val + 1.402 * (cr_val - 128.0)).clamp(0.0, 255.0) as u8;
            let g = (y_val - 0.344136 * (cb_val - 128.0) - 0.714136 * (cr_val - 128.0))
                .clamp(0.0, 255.0) as u8;
            let b = (y_val + 1.772 * (cb_val - 128.0)).clamp(0.0, 255.0) as u8;
            
            img.put_pixel(x, y, image::Rgb([r, g, b]));
        }
    }
    
    Ok(img)
}

// Step 5: Save the final image
fn save_image(img: &RgbImage, output_path: &str) -> Result<(), String> {
    img.save(output_path)
        .map_err(|e| format!("Failed to save image: {}", e))
}

// Helper functions

fn parse_file(content: &str) -> Result<((u32, u32), 
                                        Vec<(Vec<String>, TreeNode)>, 
                                        Vec<(Vec<String>, TreeNode)>, 
                                        Vec<(Vec<String>, TreeNode)>), String> {
    
    // Procurar dimensões no início do arquivo
    let mut width = 0;
    let mut height = 0;
    
    // Verificar se tem cabeçalho com dimensões
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        if line.starts_with("WIDTH ") {
            width = line[6..].trim().parse()
                .map_err(|e| format!("Invalid width: {}", e))?;
        } else if line.starts_with("HEIGHT ") {
            height = line[7..].trim().parse()
                .map_err(|e| format!("Invalid height: {}", e))?;
        }
    }
    
    // Se não encontrou dimensões, tentar estimar
    if width == 0 || height == 0 {
        println!("⚠️  Aviso: Dimensões não encontradas no arquivo. Estimando...");
        // Vamos parsear primeiro e estimar depois
    }
    
    let mut channels = Vec::new();
    let mut remaining = content;
    
    // Pular linhas do cabeçalho até encontrar o primeiro canal
    if let Some(first_channel) = remaining.find("BEGIN_OF_CHANNEL") {
        remaining = &remaining[first_channel..];
    }
    
    while let Some(ch_start) = remaining.find("BEGIN_OF_CHANNEL") {
        let ch_end = remaining[ch_start..].find("END_OF_CHANNEL")
            .ok_or("Missing END_OF_CHANNEL")? + ch_start + "END_OF_CHANNEL".len();
        
        let channel_str = &remaining[ch_start..ch_end];
        let blocks = parse_channel(channel_str)?;
        channels.push(blocks);
        
        remaining = &remaining[ch_end..];
    }
    
    if channels.len() != 3 {
        return Err(format!("Expected 3 channels, found {}", channels.len()));
    }
    
    Ok(((width, height), channels[0].clone(), channels[1].clone(), channels[2].clone()))
}

fn parse_channel(content: &str) -> Result<Vec<(Vec<String>, TreeNode)>, String> {
    let mut blocks = Vec::new();
    let mut pos = 0;
    
    while pos < content.len() {
        // Find N_WORDS
        if let Some(nw_start) = content[pos..].find("BEGIN_OF_N_WORDS") {
            let nw_end = content[pos + nw_start..].find("END_OF_N_WORDS")
                .ok_or("Missing END_OF_N_WORDS")?;
            
            let nw_str = content[pos + nw_start + "BEGIN_OF_N_WORDS".len()..
                                pos + nw_start + nw_end].trim();
            
            let n_words = u8::from_str_radix(nw_str, 2)
                .map_err(|e| format!("Invalid binary: {}", e))? as usize;
            
            pos += nw_start + nw_end + "END_OF_N_WORDS".len();
            
            // Read words
            let mut words = Vec::new();
            for _ in 0..n_words {
                if let Some(w_start) = content[pos..].find("BEGIN_OF_WORD") {
                    let w_end = content[pos + w_start..].find("END_OF_WORD")
                        .ok_or("Missing END_OF_WORD")?;
                    
                    let word = content[pos + w_start + "BEGIN_OF_WORD".len()..
                                      pos + w_start + w_end].trim();
                    
                    words.push(word.to_string());
                    pos += w_start + w_end + "END_OF_WORD".len();
                } else {
                    return Err("Not enough words".to_string());
                }
            }
            
            // Read tree
            if let Some(t_start) = content[pos..].find("BEGIN_OF_TREE") {
                let t_end = content[pos + t_start..].find("END_OF_TREE")
                    .ok_or("Missing END_OF_TREE")?;
                
                let tree_str = content[pos + t_start + "BEGIN_OF_TREE".len()..
                                      pos + t_start + t_end].trim();
                
                let tree = parse_tree_string(tree_str)?;
                blocks.push((words, tree));
                
                pos += t_start + t_end + "END_OF_TREE".len();
            } else {
                return Err("Missing tree".to_string());
            }
        } else {
            break;
        }
    }
    
    Ok(blocks)
}

fn parse_tree_string(s: &str) -> Result<TreeNode, String> {
    //println!("DEBUG: Parsing tree string: '{}'", s);
    
    // Caso 1: Apenas um par - "0(0, 64)"
    if !s.contains('1') {
        // Formato: "0(x,y)"
        if !s.starts_with('0') || !s.contains('(') || !s.contains(')') {
            return Err(format!("Formato inválido para árvore com um nó: '{}'", s));
        }
        
        let content_start = s.find('(').ok_or("Não encontrou '('")? + 1;
        let content_end = s.find(')').ok_or("Não encontrou ')'")?;
        let content = &s[content_start..content_end];
        
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
        if parts.len() != 2 {
            return Err(format!("Conteúdo deve ter 2 partes: '{}'", content));
        }
        
        let first_val: i8 = parts[0].parse()
            .map_err(|e| format!("Número inválido '{}': {}", parts[0], e))?;
        
        let second_val: i8 = parts[1].parse()
            .map_err(|e| format!("Número inválido '{}': {}", parts[1], e))?;
        
        //println!("DEBUG: Árvore com um nó: ({}, {})", first_val, second_val);
        
        // Árvore com apenas uma folha
        // Neste caso, ambos os bits (0 e 1) levam ao mesmo símbolo
        return Ok(TreeNode {
            symbol: None,
            left: Some(Box::new(TreeNode {
                symbol: Some((first_val, second_val)),
                left: None,
                right: None,
            })),
            right: Some(Box::new(TreeNode {
                symbol: Some((first_val, second_val)),
                left: None,
                right: None,
            })),
        });
    }
    
    // Caso 2: Dois pares - "0(-128, 1)1(0, 63)"
    // ... seu código existente para dois pares ...
    let chars: Vec<char> = s.chars().collect();
    let mut idx = 0;
    
    // Primeiro child (bit 0)
    if chars[idx] != '0' {
        return Err(format!("Expected '0' for first child, got '{}'", chars[idx]));
    }
    idx += 1;
    
    if chars[idx] != '(' {
        return Err(format!("Expected '(' after '0', got '{}'", chars[idx]));
    }
    idx += 1;
    
    // Parse primeiro número
    let mut num1_str = String::new();
    while idx < chars.len() && chars[idx] != ',' {
        num1_str.push(chars[idx]);
        idx += 1;
    }
    
    if chars[idx] != ',' {
        return Err(format!("Expected ',', got '{}'", chars[idx]));
    }
    idx += 1; // Pula a vírgula
    
    // Pular espaços APÓS a vírgula
    while idx < chars.len() && chars[idx].is_whitespace() {
        idx += 1;
    }
    
    // Parse segundo número
    let mut num2_str = String::new();
    while idx < chars.len() && chars[idx] != ')' {
        num2_str.push(chars[idx]);
        idx += 1;
    }
    
    if chars[idx] != ')' {
        return Err(format!("Expected ')', got '{}'", chars[idx]));
    }
    idx += 1; // Pula o ')'
    
    // Parse os números
    let first_val: i8 = num1_str.parse()
        .map_err(|e| format!("Invalid first number '{}': {}", num1_str, e))?;
    
    let second_val: i8 = num2_str.parse()
        .map_err(|e| format!("Invalid second number '{}': {}", num2_str, e))?;
    
    //println!("DEBUG: First child: ({}, {})", first_val, second_val);
    
    // Segundo child (bit 1)
    if idx >= chars.len() {
        return Err("Unexpected end after first child".to_string());
    }
    
    // Pular qualquer espaço entre os dois pares
    while idx < chars.len() && chars[idx].is_whitespace() {
        idx += 1;
    }
    
    if chars[idx] != '1' {
        return Err(format!("Expected '1' for second child, got '{}'", chars[idx]));
    }
    idx += 1;
    
    if idx >= chars.len() || chars[idx] != '(' {
        return Err("Expected '(' after '1'".to_string());
    }
    idx += 1;
    
    // Parse terceiro número
    let mut num3_str = String::new();
    while idx < chars.len() && chars[idx] != ',' {
        num3_str.push(chars[idx]);
        idx += 1;
    }
    
    if idx >= chars.len() || chars[idx] != ',' {
        return Err("Expected ',' after third number".to_string());
    }
    idx += 1; // Pula a vírgula
    
    // Pular espaços APÓS a vírgula
    while idx < chars.len() && chars[idx].is_whitespace() {
        idx += 1;
    }
    
    // Parse quarto número
    let mut num4_str = String::new();
    while idx < chars.len() && chars[idx] != ')' {
        num4_str.push(chars[idx]);
        idx += 1;
    }
    
    if idx >= chars.len() || chars[idx] != ')' {
        return Err("Expected ')' after fourth number".to_string());
    }
    
    let third_val: i8 = num3_str.parse()
        .map_err(|e| format!("Invalid third number '{}': {}", num3_str, e))?;
    
    let fourth_val: i8 = num4_str.parse()
        .map_err(|e| format!("Invalid fourth number '{}': {}", num4_str, e))?;
    
    //println!("DEBUG: Second child: ({}, {})", third_val, fourth_val);
    
    // Build tree with two leaf nodes
    Ok(TreeNode {
        symbol: None,
        left: Some(Box::new(TreeNode {
            symbol: Some((first_val, second_val)),
            left: None,
            right: None,
        })),
        right: Some(Box::new(TreeNode {
            symbol: Some((third_val, fourth_val)),
            left: None,
            right: None,
        })),
    })
}

fn decode_channel(blocks_data: Vec<(Vec<String>, TreeNode)>) -> Result<Vec<ImageBlock<i8>>, String> {
    let mut decoded_blocks = Vec::new();
    
    for (codes, tree) in blocks_data {
        // Decode Huffman symbols
        let rle_symbols = decode_huffman_symbols(&codes, &tree);
        
        // Expand RLE
        let quantized_values = expand_rle(rle_symbols)?;
        
        // Inverse zigzag
        let block = inverse_zigzag(quantized_values)?;
        
        decoded_blocks.push(block);
    }
    
    Ok(decoded_blocks)
}

fn decode_huffman_symbols(codes: &[String], tree: &TreeNode) -> Vec<(i8, i8)> {
    let mut symbols = Vec::new();
    
    for code in codes {
        let mut node = tree;
        for ch in code.chars() {
            match ch {
                '0' => {
                    if let Some(ref left) = node.left {
                        node = left;
                    } else {
                        panic!("Invalid Huffman tree");
                    }
                }
                '1' => {
                    if let Some(ref right) = node.right {
                        node = right;
                    } else {
                        panic!("Invalid Huffman tree");
                    }
                }
                _ => panic!("Invalid code character"),
            }
        }
        
        if let Some(symbol) = node.symbol {
            symbols.push(symbol);
        } else {
            panic!("Code didn't reach a leaf");
        }
    }
    
    symbols
}

fn expand_rle(rle_symbols: Vec<(i8, i8)>) -> Result<Vec<i8>, String> {
    let mut values = Vec::new();
    
    for (value, count) in rle_symbols {
        for _ in 0..count {
            values.push(value);
        }
    }
    
    if values.len() != 64 {
        return Err(format!("Expected 64 values after RLE, got {}", values.len()));
    }
    
    Ok(values)
}

fn inverse_zigzag(values: Vec<i8>) -> Result<ImageBlock<i8>, String> {
    if values.len() != 64 {
        return Err("Zigzag values must be 64".to_string());
    }
    
    let mut block = vec![0; 64];
    let zigzag_order = [
        0, 1, 8, 16, 9, 2, 3, 10,
        17, 24, 32, 25, 18, 11, 4, 5,
        12, 19, 26, 33, 40, 48, 41, 34,
        27, 20, 13, 6, 7, 14, 21, 28,
        35, 42, 49, 56, 57, 50, 43, 36,
        29, 22, 15, 23, 30, 37, 44, 51,
        58, 59, 52, 45, 38, 31, 39, 46,
        53, 60, 61, 54, 47, 55, 62, 63
    ];
    
    for (i, &pos) in zigzag_order.iter().enumerate() {
        block[pos] = values[i];
    }
    
    Ok(block)
}

fn process_channel_blocks(blocks: Vec<ImageBlock<i8>>, is_luminance: bool) -> Result<Vec<ImageBlock<u8>>, String> {
    let mut processed_blocks = Vec::new();
    
    for block in blocks {
        // Dequantize
        let dequantized = dequantize_block(&block, is_luminance);
        
        // Inverse DCT
        let idct_block = inverse_dct_block(&dequantized);
        
        processed_blocks.push(idct_block);
    }
    
    Ok(processed_blocks)
}

fn dequantize_block(block: &[i8], is_luminance: bool) -> Vec<f64> {
    let luminance_qtable = [
        4.0, 3.0, 4.0, 4.0, 4.0, 6.0, 11.0, 15.0,
        3.0, 3.0, 3.0, 4.0, 5.0, 8.0, 14.0, 19.0,
        3.0, 4.0, 4.0, 5.0, 8.0, 12.0, 16.0, 20.0,
        4.0, 5.0, 6.0, 7.0, 12.0, 14.0, 18.0, 20.0,
        6.0, 6.0, 9.0, 11.0, 14.0, 17.0, 21.0, 23.0,
        9.0, 12.0, 12.0, 18.0, 23.0, 22.0, 25.0, 21.0,
        11.0, 13.0, 15.0, 17.0, 21.0, 23.0, 25.0, 21.0,
        13.0, 12.0, 12.0, 13.0, 16.0, 19.0, 21.0, 21.0,
    ];
    
    let chrominance_qtable = [
        4.0, 4.0, 6.0, 10.0, 21.0, 21.0, 21.0, 21.0,
        4.0, 5.0, 6.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        6.0, 6.0, 12.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        10.0, 14.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
        21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0, 21.0,
    ];
    
    let qtable = if is_luminance { &luminance_qtable } else { &chrominance_qtable };
    
    let mut result = Vec::with_capacity(64);
    for i in 0..64 {
        result.push(block[i] as f64 * qtable[i]);
    }
    
    result
}

fn inverse_dct_block(block: &[f64]) -> Vec<u8> {
    let mut result = vec![0.0; 64];
    let pi = std::f64::consts::PI;
    
    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0.0;
            
            for u in 0..8 {
                let cu = if u == 0 { 1.0 / 2.0_f64.sqrt() } else { 1.0 };
                for v in 0..8 {
                    let cv = if v == 0 { 1.0 / 2.0_f64.sqrt() } else { 1.0 };
                    
                    let cos1 = ((2.0 * x as f64 + 1.0) * u as f64 * pi / 16.0).cos();
                    let cos2 = ((2.0 * y as f64 + 1.0) * v as f64 * pi / 16.0).cos();
                    
                    sum += cu * cv * block[v * 8 + u] * cos1 * cos2;
                }
            }
            
            result[y * 8 + x] = sum / 4.0;
        }
    }
    
    // Para Y: level shift +128, clamp 0-255
    // Para Cb/Cr: JÁ estão centrados em 128, apenas clamp
    result.iter()
        .map(|&v| {
            let shifted = v + 128.0; // Para Y, sempre shift
            shifted.clamp(0.0, 255.0).round() as u8
        })
        .collect()
}

fn blocks_to_pixels(blocks: Vec<ImageBlock<u8>>, width: u32, height: u32, is_luminance: bool) -> Vec<u8> {
    let blocks_per_row = ((width + 7) / 8) as usize;
    let total_rows = ((height + 7) / 8) as usize;
    
    let mut pixels = vec![0u8; (width * height) as usize];
    
    for (block_idx, block) in blocks.iter().enumerate() {
        let row = block_idx / blocks_per_row;
        let col = block_idx % blocks_per_row;
        
        for i in 0..8 {
            for j in 0..8 {
                let x = col * 8 + j;
                let y = row * 8 + i;
                
                if (x as u32) < width && (y as u32) < height {
                    let idx = (y as u32 * width + x as u32) as usize;
                    pixels[idx] = block[i * 8 + j];
                }
            }
        }
    }
    
    pixels
}