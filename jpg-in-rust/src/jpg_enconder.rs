use image::GenericImageView;
use image::DynamicImage;
use image::Rgba;

type Image = DynamicImage;
type YCCColorSpace = (u8, u8, u8);
type Pixel = (u32, u32, Rgba<u8>);

pub fn encode(filepath : &str) {
    match pre_processing(filepath) {
        Ok(img) => {
            let crominance_values = colorspace_conversion(&img);

            split_into_blocks(&crominance_values);

            discrete_cosine_transform(&mut img);

            quantization(&mut img);

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

// Step 1
pub fn colorspace_conversion(img : &Image) -> Vec<YCCColorSpace> {
    let pixels = img.pixels();
    let (w, h) = img.dimensions();

    let mut crominance_values : Vec<YCCColorSpace> = vec![(0, 0, 0); (w * h) as usize];
    let red = |pixel : Pixel| {pixel.2.0[0] as f64};
    let blue = |pixel : Pixel| {pixel.2.0[1] as f64};
    let green = |pixel : Pixel| {pixel.2.0[2] as f64};

    for pixel in pixels {
        let i = (pixel.0 + pixel.1 * h) as usize;
        crominance_values[i] = (
            (0.299 * red(pixel) + 0.587 * green(pixel) + 0.114 * blue(pixel)) as u8,
            (-0.1687 * red(pixel) - 0.3313 * green(pixel) + 0.5 * blue(pixel) + 128.0) as u8,
            (0.5 * red(pixel) - 0.4187 * green(pixel) - 0.0813 * blue(pixel) + 128.0) as u8
        );
    }

    return crominance_values;
}

// Step 2
pub fn split_into_blocks(img : &Vec<YCCColorSpace>) {
    
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