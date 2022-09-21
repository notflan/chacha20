//! Use memory mapping to process the entire stream at once
use super::*;

use std::os::unix::prelude::*;
use std::{
    io,
    fs,
    ptr,
    ops,
    mem::{
	self,
	MaybeUninit,
    },
    borrow::BorrowMut,
    convert::{TryFrom, TryInto,},
};
use libc::{
    mmap, munmap, madvise, MAP_FAILED,
};
use openssl::symm::Crypter;

#[derive(Debug)]
struct MapInner
{
    mem: ptr::NonNull<u8>,
    size: usize
}

impl ops::Drop for MapInner
{
    #[inline]
    fn drop(&mut self)
    {
	unsafe {
	    munmap(self.mem.as_ptr() as *mut _, self.size);
	}
    }
}

#[inline] 
pub fn raw_file_size(fd: &(impl AsRawFd + ?Sized)) -> io::Result<u64>
{
    use libc::fstat;
    let mut stat = MaybeUninit::uninit();
    match unsafe { fstat(fd.as_raw_fd(), stat.as_mut_ptr()) } {
	0 => match unsafe {stat.assume_init()}.st_size {
	    x if x < 0 => Err(io::Error::new(io::ErrorKind::InvalidInput, format!("File from {} is too large", fd.as_raw_fd()))),
	    x => Ok(x as u64),
	},
	_ => Err(io::Error::last_os_error()),
    }
}

#[derive(Debug)]
pub struct Mapped<T: ?Sized>
{
    mem: MapInner,
    file: T
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub enum MapMode
{
    ReadOnly,
    ReadWrite,
    WriteOnly,
}
//TODO: impl MapMode -> fn as_prot() -> c_int, fn as_flags() -> c_int

impl<T: AsRawFd> Mapped<T>
{
    pub fn try_new_sized(file: T, size: u64, rw: MapMode) -> io::Result<Self>
    {
	todo!("mem: mmap(0, size, rw.prot(), MAP_SHARED (For rw: *Write*), MAP_PRIVATE [TODO: add support for| MAP_HUGE_] (For rw: ReadOnly), file.as_raw_fd(), 0) != MAP_FAILED (TODO: maybe madvise?)")
    }
    #[inline(always)] 
    pub fn try_new(file: T, rw: MapMode) -> io::Result<Self>
    {
	let size = raw_file_size(&file)?;
	Self::try_new_sized(file, size, rw)
    }
}
/*
impl<T: AsRawFd> TryFrom<T> for Mapped<T>
{
    type Error = io::Error;

    fn try_from(from: T) -> Result<Self, Self::Error>
    {
	Self::try_new(from, raw_file_size(&from)?)
    }
}*/


fn process_mapped_files<T, U>(mode: &mut Crypter, input: &mut Mapped<T>, output: &mut Mapped<U>) -> io::Result<()>
where T: AsRawFd + ?Sized,
U: AsRawFd + ?Sized
{
    todo!("mode.update(input.slice(), output.slice()); /* unneeded: mode.finalize(input.slice(), output.slice()) */ ");
}

#[inline] 
pub fn try_process(mut mode: impl BorrowMut<Crypter>) -> io::Result<io::Result<()>>
{
    let mode = mode.borrow_mut();
    /*let mstdin = Mapped::try_new(input)?;
    let mstdout = Mapped::try_new(output)?;*/ //TODO: if failed to map output, but input is successful (TODO: XXX: implement other way around), we can fall back to wrapping output in `Sink`.
    todo!("Try to map the stdin and stdout streams, if that fails, return Err(last_os_err).");
    todo!("return Ok(process_mapped_files(mode, mstdin, mstdout, key, iv))")
}
