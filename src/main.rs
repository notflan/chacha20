//#![cfg_attr(nightly, feature(asm))] 



#![allow(dead_code)]

#[macro_use] extern crate lazy_static;
//extern crate test;

#[macro_use] mod ext; #[allow(unused_imports)] use ext::*;

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
	Some('k') => Mode::Keygen,
	other => {
	    eprintln!("{} (v{}) - chacha20_poly1305 command line encryption tool",
		      env!("CARGO_PKG_NAME"),
		      env!("CARGO_PKG_VERSION"));
	    eprintln!(" by {} with <3 (licensed GPL v3.0 or later)", env!("CARGO_PKG_AUTHORS"));
	    eprintln!("\nStreams stdin to stdout through a chacha20_poly1305 cipher.");
	    eprintln!();
	    eprintln!("Usage: {} encrypt [<base64 key>] [<base64 iv>]", prog_name);
	    eprintln!("Usage: {} decrypt [<base64 key>] [<base64 iv>]", prog_name);
	    eprintln!("Usage: {} keygen [<base64 key>] [<base64 iv>]", prog_name);
	    eprintln!("Usage: {} help", prog_name);
	    eprintln!();
	    eprintln!("(Key size is {}, IV size is {})", cha::KEY_SIZE, cha::IV_SIZE);
	    eprintln!("(requires OpenSSL 1.1.0 or newer)");
	    eprintln!("\nencrypt/decrypt:\n\tIf a key and/or IV are not provided, they are generated randomly and printed to stderr in order on one line each.");
	    eprintln!("\tIf the key and/or IV provided's size is lower than the cipher's key/IV size, the rest of the key/IV is padded with 0s. If the size is higher, the extra bytes are ignored.");
	    eprintln!("\nkeygen:\n\tThe key/iv is printed in the same way as auto-generated keys for the en/decryption modes, but to stdout instead of stderr. If a key is given as parameter, the key is not printed. If the iv is given as a parameter also, nothing is printed.");
	    eprintln!("\nhelp:\n\tPrint this message to stderr then exit with code 0");
	    std::process::exit(if other == Some('h') {0} else {1})
	}
    };
    
    let key = match args.next() {
	Some(key) => key.parse()?,
	None => {
	    let key = Key::new();
	    if mode == Mode::Keygen {
		println!("{}", base64::encode(&key));
	    } else {
		eprintln!("{}", base64::encode(&key));
	    }
	    key
	},
    };
    let iv = match args.next() {
	Some(iv) => iv.parse()?,
	None => {
	    let iv = IV::new();
	    if mode == Mode::Keygen {
		println!("{}", base64::encode(&iv));
	    } else {
		eprintln!("{}", base64::encode(&iv));
	    }
	    iv
	},
    };

    Ok((mode, key, iv))
}

const USE_MMAP: bool = if cfg!(feature="mmap") {
    true
} else {
    false
};

#[cfg(feature="mmap")]
mod mapped;

#[allow(unreachable_code)]
fn try_mmap(decrypt: bool, key: Key, iv: IV) -> Result<i32, mapped::ProcessError>
{
    #[cfg(feature="mmap")] return mapped::try_process(if decrypt {
	cha::decrypter(key, iv).expect("Failed to create decrypter")
    } else {
	cha::encrypter(key, iv).expect("Failed to create encrypter")
    }).map(|_| 0i32);
    
    unreachable!("Built without feature `mmap`, but still tried to call into it. This is a bug")
}

fn main() {
    let (mode, key, iv) = keys().expect("Failed to read keys from argv (base64)");
    
    // Attempt a mapped solution
    if USE_MMAP && mode != Mode::Keygen {
	match try_mmap(mode == Mode::Decrypt, key, iv) {
	    Ok(0) => return,
	    Ok(n) => std::process::exit(n),
	    Err(err) => if cfg!(debug_assertions) {
		eprintln!("Failed to mmap input or output for processing, falling back to stream: {}", &err);
		eprintln!("\t{:?}", err);
	    }
	}
    }
    
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
	    let mut output = stream::Sink::decrypt(stdout.lock(), key, iv).expect("Failed to create decrypter");
	    std::io::copy(&mut input.lock(), &mut output).expect("Failed to decrypt");
	    output.flush().expect("Failed to flush stdout");
	},
	Mode::Keygen => {
	    //println!("{}", base64::encode(&key));
	    //println!("{}", base64::encode(&iv));
	},
    }
}
