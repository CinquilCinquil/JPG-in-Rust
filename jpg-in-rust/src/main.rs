use std::env;
mod jpg_enconder;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Image path not supplied!");
    }

    let filepath = &args[1];

    jpg_enconder::encode(filepath);
}
