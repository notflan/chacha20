
#[macro_use] extern crate hex_literal;

mod ext; use ext::*;

mod key;
mod cha;

use key::{Key, IV};

fn encrypt((key, iv): &(Key, IV), input: impl AsRef<[u8]>) -> Result<String, openssl::error::ErrorStack>
{
    let input = input.as_ref();
    let mut output = vec![0u8; input.len()];

    eprintln!("(enc) Key: {}, IV: {}, Input: ({}, {})", key, iv, input.len(), input.hex());
    
    let mut enc = cha::encrypter(key, iv)?;

    let n = enc.update(&input[..], &mut output[..])?;
    eprintln!("(enc) Written {} bytes", n);
    enc.finalize(&mut output[..n])?;

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
    dec.finalize(&mut output[..n])?;

    output.truncate(n);
    Ok(output)
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or({
	let mut input = [0u8; 16];
	getrandom::getrandom(&mut input[..]).expect("rng fatal");
	khash::generate(&Default::default(), input).expect("kana-hash fatal")
	//input.hex().into()
    });
    
    let key = cha::keygen();
    let enc = encrypt(&key, &input).expect("encrypt");
    println!("{}", enc);
    let dec = decrypt(&key, enc).expect("decrypt");

    let output = std::str::from_utf8(&dec[..]).unwrap();
    println!("{:?}", output);
    assert_eq!(output, &input[..]);
}
