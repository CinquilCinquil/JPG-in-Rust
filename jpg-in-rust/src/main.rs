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
    let decoder_filepath = &args[2];

    encoder::encode(filepath);

    let image = "image_decoder.png";

    match decoder::decode(decoder_filepath, image) {
        Ok(_) => println!("✅ Sucesso! Imagem salva em {}", image),
        Err(e) => println!("❌ Erro: {}", e),
    }
    
}
