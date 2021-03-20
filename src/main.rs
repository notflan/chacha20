
#[macro_use] extern crate hex_literal;

mod cha;

use cha::{Key, IV};

fn encrypt((key, iv): &(Key, IV), input: impl Into<Vec<u8>>) -> Result<String, openssl::error::ErrorStack>
{
    let input = input.into();//.into();
    let mut output = vec![0u8; input.len()];

    eprintln!("(enc) Key: {:?}, IV: {:?}, Input: ({}, {:?})", key, iv, input.len(), input);
    
    let mut enc = cha::encrypter(key, iv)?;

    let n = enc.update(&input[..], &mut output[..])?;
    eprintln!("(enc) Written {} bytes", n);
    enc.finalize(&mut output[..n])?;

    Ok(base64::encode(&output[..n]))
}

fn decrypt((key, iv): &(Key, IV), input: impl Into<String>) -> Result<Vec<u8>, openssl::error::ErrorStack>
{
    let input = base64::decode(input.into()).expect("invalid base64");
    let mut output = vec![0u8; input.len()];

    eprintln!("(dec) Key: {:?}, IV: {:?}, Input: ({}, {:?})", key, iv, input.len(), input);

    let mut dec = cha::decrypter(key, iv)?;

    let n = dec.update(&input[..], &mut output[..])?;
    eprintln!("(dec) Written {} bytes", n);
    dec.finalize(&mut output[..n])?;

    output.truncate(n);
    Ok(output)
}

fn main() {
    let key = cha::keygen();
    let enc = encrypt(&key, std::env::args().nth(1).unwrap()).expect("encrypt");
    println!("{}", enc);
    let dec = decrypt(&key, enc).expect("decrypt");

    println!("{:?}", std::str::from_utf8(&dec[..]).unwrap());
}
