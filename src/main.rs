
#![allow(dead_code)]

mod ext; #[macro_use] use ext::*;

mod key;
mod cha;
mod stream;

use key::{Key, IV};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Mode
{
    Encrypt, Decrypt, Keygen
}

fn keys() -> Result<(Mode, Key, IV), base64::DecodeError>
{
    let mut args = std::env::args();
    let prog_name = args.next().unwrap();

    let mode = match args.next()
	.map(|x| x.chars().next().map(|x| x.to_ascii_lowercase()))
	.flatten()
    {
	Some('e') => Mode::Encrypt,
	Some('d') => Mode::Decrypt,
	Some('k') => {
	    let (key, iv) = cha::keygen();
	    return Ok((Mode::Keygen, key, iv));
	},
	_ => {
	    eprintln!("{} (v{}) - chacha20_poly1305 command line encryption tool",
		      env!("CARGO_PKG_NAME"),
		      env!("CARGO_PKG_VERSION"));
	    eprintln!(" by {} with <3 (licensed GPL v3.0 or later)", env!("CARGO_PKG_AUTHORS"));
	    eprintln!("\nStreams stdin to stdout through a chacha20_poly1305 cipher.");
	    eprintln!();
	    eprintln!("Usage: {} encrypt [<base64 key>] [<base64 iv>]", prog_name);
	    eprintln!("Usage: {} decrypt [<base64 key>] [<base64 iv>]", prog_name);
	    eprintln!("Usage: {} keygen", prog_name);
	    eprintln!();
	    eprintln!("(Key size is {}, IV size is {})", cha::KEY_SIZE, cha::IV_SIZE);
	    eprintln!("\nencrypt/decrypt:\n\tIf key and/or IV are not provided, they are generated randomly and printed to stderr in order on one line each");
	    eprintln!("\tIf the key and/or IV provided's size is lower than the cipher's key/IV size, the rest of the key/IV padded with 0s. If the size is higher, the extra bytes are ignored.");
	    eprintln!("\nkeygen:\n\tThe key/iv is printed in the same way as auto-generated keys for the en/decryption modes, but to stdout instead of stderr");
	    std::process::exit(1)
	}
    };
    
    let key = match args.next() {
	Some(key) => key.parse()?,
	None => {
	    let key = Key::new();
	    eprintln!("{}", base64::encode(&key));
	    key
	},
    };
    let iv = match args.next() {
	Some(iv) => iv.parse()?,
	None => {
	    let iv = IV::new();
	    eprintln!("{}", base64::encode(&iv));
	    iv
	},
    };

    Ok((mode, key, iv))
}

fn main() {

    let (mode, key, iv) = keys().expect("Failed to read keys from argv (base64)");
    
    let stdout = std::io::stdout();
    let input = std::io::stdin();

    // Streaming
    use std::io::Write;
    match mode 
    {
	Mode::Encrypt => {
	    let mut output = stream::Sink::encrypt(stdout.lock(), key, iv).expect("Failed to create encrypter");
	    std::io::copy(&mut input.lock(), &mut output).expect("Failed to encrypt");
	    output.flush().expect("Failed to flush stdout");
	},
	Mode::Decrypt => {
	    let mut output = stream::Sink::decrypt(stdout.lock(), key, iv).expect("Failed to create encrypter");
	    std::io::copy(&mut input.lock(), &mut output).expect("Failed to encrypt");
	    output.flush().expect("Failed to flush stdout");
	},
	Mode::Keygen => {
	    println!("{}", base64::encode(&key));
	    println!("{}", base64::encode(&iv));
	},
    }
}
