use image::GenericImageView;

type Image = i32; // Temporary!!

pub fn encode(filepath : &str) {
    let mut img = pre_processing(filepath);

    colorspace_conversion(&mut img);

    split_into_blocks(&mut img);

    discrete_cosine_transform(&mut img);

    quantization(&mut img);

    statistical_enconding(&mut img);

    save_image(&mut img);
}

// Step 0
pub fn pre_processing(filepath : &str) -> Image {
    return 0;
}

// Step 1
pub fn colorspace_conversion(img : &mut Image) {
    
}

// Step 2
pub fn split_into_blocks(img : &mut Image) {
    
}

// Step 4
pub fn discrete_cosine_transform(img : &mut Image) {
    
}

// Step 5
pub fn quantization(img : &mut Image) {
    
}

// Step 6
pub fn statistical_enconding(img : &mut Image) {
    
}

// Step 7
pub fn save_image(img : &mut Image) {
    
}