use std::env;
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
}
