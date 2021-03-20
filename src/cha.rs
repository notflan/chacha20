
use getrandom::getrandom;
use openssl::{
    symm::{
	Cipher, Crypter, Mode,
    },
    error::ErrorStack,
};

pub const KEY_SIZE: usize = 32;
pub const IV_SIZE: usize = 12;

static NEW_CIPHER: fn() -> Cipher = Cipher::chacha20_poly1305;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
#[repr(transparent)]
pub struct Key([u8; KEY_SIZE]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
#[repr(transparent)]
pub struct IV([u8; IV_SIZE]);

#[inline] pub fn decrypter(key: impl AsRef<Key>, iv: impl AsRef<IV>) -> Result<Crypter, ErrorStack>
{
    Crypter::new(
	NEW_CIPHER(),
	Mode::Decrypt,
	key.as_ref().as_ref(),
	Some(iv.as_ref().as_ref())
    )
}
#[inline] pub fn encrypter(key: impl AsRef<Key>, iv: impl AsRef<IV>) -> Result<Crypter, ErrorStack>
{
    Crypter::new(
	NEW_CIPHER(),
	Mode::Encrypt,
	key.as_ref().as_ref(),
	Some(iv.as_ref().as_ref())
    )
}

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

#[inline(always)] pub fn keygen() -> (Key, IV)
{
    (Key::new(), IV::new())
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

