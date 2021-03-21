
#[macro_use] extern crate hex_literal;

mod ext; #[macro_use] use ext::*;

mod key;
mod cha;
mod stream;

use key::{Key, IV};
/*
fn encrypt((key, iv): &(Key, IV), input: impl AsRef<[u8]>) -> Result<String, openssl::error::ErrorStack>
{
    let input = input.as_ref();
    let mut output = vec![0u8; input.len()];

    eprintln!("(enc) Key: {}, IV: {}, Input: ({}, {})", key, iv, input.len(), input.hex());
    
    let mut enc = cha::encrypter(key, iv)?;

    let n = enc.update(&input[..], &mut output[..])?;
    eprintln!("(enc) Written {} bytes", n);
    
    println!(">> {}", (&output[..n]).hex());
    assert!(enc.finalize(&mut output[..n])? == 0);
    println!(">> {}", (&output[..n]).hex());

    Ok(base64::encode(&output[..n]))
}

fn decrypt((key, iv): &(Key, IV), input: impl AsRef<str>) -> Result<Vec<u8>, openssl::error::ErrorStack>
{
    let input = base64::decode(input.as_ref()).expect("invalid base64");
    let mut output = vec![0u8; input.len()];

    eprintln!("(dec) Key: {}, IV: {}, Input: ({}, {})", key, iv, input.len(), input.hex());

    let mut dec = cha::decrypter(key, iv)?;

    let n = dec.update(&input[..], &mut output[..])?;
    eprintln!("(dec) Written {} bytes", n);

    println!(">> {}", (&output[..n]).hex());
    assert!(dec.finalize(&mut output[..n])? == 0);
    //    assert!(dec.finalize(&mut output[..n])? == 0);
    println!(">> {}", (&output[..n]).hex());

    output.truncate(n);
    Ok(output)
}*/

fn keys() -> Result<(Key, IV), base64::DecodeError>
{
    let mut args = std::env::args().skip(1);
    
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
	    let key = IV::new();
	    eprintln!("{}", base64::encode(&key));
	    key
	},
    };

    Ok((key, iv))
}

fn main() {

    let (key, iv) = keys().expect("Failed to read keys from argv (base64)");
    
    let stdout = std::io::stdout();
    let input = std::io::stdin();

    // Encryption
    {
	use std::io::Write;
	let mut output = stream::Sink::encrypt(stdout.lock(), key, iv).expect("Failed to create encrypter");
	std::io::copy(&mut input.lock(), &mut output).expect("Failed to encrypt");
	output.flush().expect("Failed to flush stdout");
    }
}
