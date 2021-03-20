use getrandom::getrandom;
use std::fmt;
use crate::cha::{
    KEY_SIZE,
    IV_SIZE,
};
use crate::ext::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
#[repr(transparent)]
pub struct Key([u8; KEY_SIZE]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
#[repr(transparent)]
pub struct IV([u8; IV_SIZE]);


impl Key
{
    pub fn new() -> Self
    {
	let mut output = [0u8; KEY_SIZE];
	getrandom(&mut output[..]).expect("rng fatal");
	Self(output)
    }
}

impl IV
{
    pub fn new() -> Self
    {
	let mut output = [0u8; IV_SIZE];
	getrandom(&mut output[..]).expect("rng fatal");
	Self(output)
    }
}

impl AsRef<[u8]> for Key
{
    fn as_ref(&self) -> &[u8]
    {
	&self.0[..]
    }
}
impl AsRef<[u8]> for IV
{
    fn as_ref(&self) -> &[u8]
    {
	&self.0[..]
    }
}

impl AsMut<[u8]> for Key
{
    fn as_mut(&mut self) -> &mut [u8]
    {
	&mut self.0[..]
    }
}

impl AsMut<[u8]> for IV
{
    fn as_mut(&mut self) -> &mut [u8]
    {
	&mut self.0[..]
    }
}

impl AsRef<Key> for Key
{
    #[inline] fn as_ref(&self) -> &Key
    {
	self
    }
}
impl AsRef<IV> for IV
{
    #[inline] fn as_ref(&self) -> &IV
    {
	self
    }
}

impl fmt::Display for Key
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "Key({})", self.0.iter().copied().into_hex())
    }
}

impl fmt::Display for IV
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
	write!(f, "Key({})", self.0.iter().copied().into_hex())
    }
}
