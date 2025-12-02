use std::env;
use std::path::Path;
mod types;
mod encoder;
mod decoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Image path not supplied!");
    }

    let filepath = &args[1];
    

    encoder::encode(filepath);
    if args.len() == 3 {
        let decoder_filepath = &args[2];
        let image = "image_decoder.png";

        match decoder::decode(decoder_filepath, image) {
            Ok(_) => println!("✅ Sucess! Image saved in {}", image),
            Err(e) => println!("❌ Error: {}", e),
        }
    }
    
}
