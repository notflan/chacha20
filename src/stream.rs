#![allow(dead_code)]

use super::*;
use key::*;

use std::io::{self, Write};
use std::fmt;
use openssl::{
    symm::Crypter,
    error::ErrorStack,
};
use smallvec::SmallVec;

pub const BUFFER_SIZE: usize = 1024;
pub type Error = ErrorStack;

/// ChaCha Sink
//#[derive(Debug)]
pub struct Sink<W>
{
    stream: W,
    crypter: Crypter, // for chacha, finalize does nothing it seems. we can also call it multiple times.

    buffer: SmallVec<[u8; BUFFER_SIZE]> // used to buffer the operation
}

impl<W: fmt::Debug> fmt::Debug for Sink<W>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "Sink({:?}, ({} buffer cap))", self.stream, self.buffer.capacity())
    }
}

impl<W> Sink<W>
where W: Write
{
    /// Create a new Chacha Sink stream wrapper
    pub fn new(stream: W, crypter: Crypter) -> Self
    {
	Self{stream, crypter, buffer: SmallVec::new()}
    }

    /// Create an encrypting Chacha Sink stream wrapper
    pub fn encrypt(stream: W, key: Key, iv: IV) -> Result<Self, Error>
    {
	Ok(Self::new(stream, cha::encrypter(key, iv)?))
    }
    
    /// Create a decrypting Chacha Sink stream wrapper
    pub fn decrypt(stream: W, key: Key, iv: IV) -> Result<Self, Error>
    {
	Ok(Self::new(stream, cha::decrypter(key, iv)?))
    }

    /// Consume into the inner stream
    pub fn into_inner(self) -> W
    {
	self.stream
    }

    /// Consume into the inner stream and crypter
    pub fn into_parts(self) -> (W, Crypter)
    {
	(self.stream, self.crypter)
    }
    
    /// The crypter of this instance
    pub fn crypter(&self) -> &Crypter
    {
	&self.crypter
    }
    
    /// The crypter of this instance
    pub fn crypter_mut(&mut self) -> &mut Crypter
    {
	&mut self.crypter
    }

    /// The inner stream
    pub fn inner(&self) -> &W
    {
	&self.stream
    }
    
    /// The inner stream
    pub fn inner_mut(&mut self) -> &mut W
    {
	&mut self.stream
    }
}

impl<W: Write> Write for Sink<W>
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
	prog1!{
	    {
		self.buffer.write_all(buf).unwrap();
		let n = self.crypter.update(&buf[..], &mut self.buffer[..])?;
		self.crypter.finalize(&mut self.buffer[..n])?; // I don't think this is needed

		self.stream.write(&self.buffer[..n])
	    },
	    self.buffer.clear();
	}
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
	prog1!{
	    {
		self.buffer.write_all(buf).unwrap();
		let n = self.crypter.update(&buf[..], &mut self.buffer[..])?;
		self.crypter.finalize(&mut self.buffer[..n])?;

		self.stream.write_all(&self.buffer[..n])
	    },
	    self.buffer.clear();
	}
    }
    #[inline] fn flush(&mut self) -> io::Result<()> {
	self.stream.flush()
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    const INPUT: &'static str = "Hello world!";

    fn enc_stream(input: impl AsRef<[u8]>, key: Key, iv: IV) -> Sink<Vec<u8>>
    {
	let enc_buffer = Vec::new();
	let input = input.as_ref();
	
	eprintln!("(enc) Key: {}, IV: {}, Input: ({}, {})", key, iv, input.len(), input.hex());
	
	let mut stream = Sink::encrypt(enc_buffer, key, iv).expect("sink::enc");
	assert_eq!(stream.write(input).unwrap(), input.len());
	stream.flush().unwrap();
	
	eprintln!("Output encrypted: {}", stream.inner().hex());

	stream
    }

    #[test]
    fn enc()
    {
	let (key, iv) = cha::keygen();

	eprintln!("Sink ends: {:?}", enc_stream(INPUT.as_bytes(), key, iv));
    }

    #[test]
    fn dec()
    {
	let (key, iv) = cha::keygen();
	eprintln!("Input unencrypted: {}", INPUT.hex());

	let input = enc_stream(INPUT.as_bytes(), key.clone(), iv.clone()).into_inner();

	let mut dec_buffer = Vec::new();
	{
	    let mut stream = Sink::decrypt(&mut dec_buffer, key, iv).expect("sink::dec");

	    stream.write_all(&input[..]).unwrap();
	    stream.flush().unwrap();
	    
	    eprintln!("Output decrypted: {}", stream.inner().hex());
	}
	assert_eq!(&dec_buffer[..], INPUT.as_bytes());
    }
}

