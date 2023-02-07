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
    borrow::{BorrowMut, Cow},
    convert::{TryFrom, TryInto,},
};
use libc::{
    mmap, munmap, madvise, MAP_FAILED,
};
use mapped_file::{
    MappedFile,
    Perm,
    Flags,
    file::memory::{
	MemoryFile,
    },
};
use openssl::symm::Crypter;
/*
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
}*/

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

#[inline]
fn try_truncate(fd: &(impl AsRawFd + ?Sized), to: u64) -> io::Result<()>
{
    //use libc::ftruncate;
    let to = to.try_into().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    match unsafe { ftruncate(fd.as_raw_fd(), to) } {
	0 => Ok(()),
	_ => Err(io::Error::last_os_error()),
    }
}
/*
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
}*/
/*
impl<T: AsRawFd> TryFrom<T> for Mapped<T>
{
    type Error = io::Error;

    fn try_from(from: T) -> Result<Self, Self::Error>
    {
	Self::try_new(from, raw_file_size(&from)?)
    }
}*/


fn process_mapped_files<T, U>(mode: &mut Crypter, input: &mut MappedFile<T>, output: &mut MappedFile<U>) -> io::Result<()>
where T: AsRawFd,
U: AsRawFd,
{
    mode.update(&input[..], &mut output[..])?;
    Ok(())
}

fn try_map_sized<T: AsRawFd>(file: T, perm: mapped_file::Perm, flags: impl mapped_file::MapFlags) -> Result<MappedFile<T>, T>
{
    macro_rules! unwrap {
	($res:expr) => {
            match $res {
		Ok(v) => v,
		_ => return Err(file)
	    }
	};
    }
    if let Ok(sz) = raw_file_size(&file) {
	let sz = unwrap!(sz.try_into());
	MappedFile::try_new(file, sz, perm, flags).map_err(|err| err.into_inner())
    } else {
	Err(file)
    }
}

#[inline] 
fn try_map_to<T: AsRawFd + ?Sized>(file: &T, file_size: usize) -> bool
{
    let file_size = match file_size.try_into() {
	Ok(v) => v,
	_ => return false
    };
    if let Ok(stdout_size) = raw_file_size(file) {
	if stdout_size >= file_size {
	    // Size already ok
	    return true;
	}
    }
    // Grow file
    unsafe {
	libc::ftruncate(file.as_raw_fd(), file_size) == 0
    }
}

#[inline] 
fn translate_hugetlb_size(size: usize) -> mapped_file::hugetlb::HugePage
{
    #[inline] 
    fn check_func(sizes_kb: &[usize]) -> Option<&usize>
    {
	sizes_kb.iter().skip_while(|&&kb| kb < 1024 * 1024).next()
	    .or_else(|| sizes_kb.iter().nth(1)
		     .or_else(|| sizes_kb.first()))
    }
    const MB: usize = 1024*1024;
    const GB: usize = 1024 * MB;
    match size {
	0..MB => mapped_file::hugetlb::HugePage::Smallest,
	MB..GB => mapped_file::hugetlb::HugePage::Selected(check_func),
	very_high => mapped_file::hugetlb::HugePage::Largest,
    }
}

/// Create and map a temporary memory file of this size.
///
/// This function may optionally choose to huge hugepages if `size` is large enough
fn create_sized_temp_mapping(size: usize) -> io::Result<(MappedFile<MemoryFile>, bool)>
{
    const HUGETLB_THRESH: usize = 1024 * 1024; // 1MB
    let file = match size {
	0 => MemoryFile::new(),
	0..HUGETLB_THRESH => MemoryFile::with_size(size),
	size => {
	    let hugetlb = translate_hugetlb_size(size);
	    let file = MemoryFile::with_size_hugetlb(size, hugetlb)?;
	    return MappedFile::new(file, size, Perm::ReadWrite, Flags::Shared.with_hugetlb(hugetlb)).map(|x| (x, true));
	},  
    }?;
    MappedFile::new(file, size, Perm::ReadWrite, Flags::Shared).map(|x| (x, false))
}

type MappedMemoryFile = MappedFile<mapped_file::file::memory::MemoryFile>;

/// How a mapped operation should optimally take place
//TODO: Make optimised mapping operations for each one, like `fn process(self, enc: &mut Crypter) -> io::Result<usize>`
pub enum OpTable<T, U>
{
    /// Both input and output are mapped
    Both(MappedFile<T>, MappedFile<U>),
    /// Input is mapped, but output is not. Work is done through a temporary map, then copied to `U`.
    Input(MappedFile<T>, MappedMemoryFile, U),
    /// Output is mapped, but input is not. Work is done from a temporary map
    Output(T, MappedMemoryFile, MappedFile<U>),
    /// Streaming mode (`read()`+`write()` only)
    Neither(T, U),
}

impl<T, U> OpTable<T, U>
{
    /// Consume into either a mapping line, or the in/out tuple if neither are mapped.
    #[inline] 
    pub fn only_mapped(self) -> Result<Self, (T, U)>
    {
	match self {
	    Self::Neither(t, u) => Err((t, u)),
	    this => Ok(this),
	}
    }
}

impl<T: io::Read+AsRawFd, U: io::Write+AsRawFd> OpTable<T, U>
{
    fn pre_process(&mut self) -> io::Result<()>
    {
	match self {
	    Self::Both(input, output)
		=> {
		    let _ = input.advise(mapped_file::Advice::Sequential, Some(true));
		    let _ = output.advise(mapped_file::Advice::Sequential, None);
		},
	    Self::Input(input, mem)
		=> {
		    let _ = input.advise(mapped_file::Advice::Sequential, Some(true));
		    let _ = mem.advise(mapped_file::Advice::Sequential, None);
		},
	    Self::Output(input, mem, _)
		=> {
		    let _ = mem.advise(mapped_file::Advice::Sequential, Some(true));
		    std::io::copy(&mut input, &mut mem)?;
		},
	    _ => (),
	}
	Ok(())
    }
    fn post_process(&mut self) -> io::Result<()>
    {
	match self {
	    Self::Both(_, output) => drop(output.flush(mapped_file::Flush::Wait)?),
	    Self::Input(_, mut mem, mut output) => drop(std::io::copy(&mut mem, &mut output)?),
	    Self::Output(_, _, mut output) => drop(output.flush(mapped_file::Flush::Wait)?),
	    Self::Neither(_, stream) => stream.flush()?,
	    _ => (),
	}
	Ok(())
    }
    /// Execute this en/decryption in an optimised function
    pub fn execute(mut self, mut mode: impl BorrowMut<Crypter>) -> io::Result<usize>
    {
	self.pre_process()?;
	let mode: &mut Crypter = mode.borrow_mut();
	match self {
	    Self::Both(mut input, mut output) => {
		let len = std::cmp::min(input.len(), output.len());
		process_mapped_files(mode, &mut input, &mut output)?;
		
		self.post_process()?;
		Ok(len)
	    },
	    Self::Input(mut input, mut output, _) => {
		let len = std::cmp::min(input.len(), output.len());
		process_mapped_files(mode, &mut input, &mut output)?;

		self.post_process()?;
		Ok(len)
	    },
	    Self::Output(_, mut input, mut output) => {
		let len = std::cmp::min(input.len(), output.len());
		process_mapped_files(mode, &mut input, &mut output)?;

		self.post_process()?;
		Ok(len)
	    },
	    Self::Neither(sin, sout) => {
		const BUFFER_SIZE: usize = 1024*1024;
		macro_rules! try_allocmem {
		    ($size:expr) => {
			{
			    let size = usize::from($size);
			    let hsz = mapped_file::hugetlb::HugePage::Dynamic { kilobytes: size/1024 }; // 1MB buffer 
			    hsz.compute_huge()
				.and_then(|huge| mapped_file::file::memory::MemoryFile::with_size_hugetlb(size, huge).ok())
				.or_else(|| mapped_file::file::memory::MemoryFile::with_size(size).ok())
				.and_then(|file| MappedFile::new(file, size, Perm::ReadWrite, Flags::Private).ok())
			}
		    };
		    ($mem:expr, $size:expr) => {
			$mem.map(|x| Cow::Borrowed(&mut x[..])).unwrap_or_else(|| Cow::Owned(vec![0u8; $size]));
		    }
		}
		let mut _mem = try_allocmem!(BUFFER_SIZE);
		let mut _memo = try_allocmem!(BUFFER_SIZE);
		let mut buffer = try_allocmem!(_mem, BUFFER_SIZE);
		let mut buffero = try_allocmem!(_memo, BUFFER_SIZE);
		let mut read =0;
		let mut cur;
		while { cur = sin.read(&mut buffer[..])?; cur > 0 } {
		    /*let cur =*/ mode.update(&buffer[..cur], &mut buffero[..cur])?;
		    sout.write_all(&buffero[..cur])?;
		    read += cur;
		}

		self.post_process()?;
		Ok(read)
	    },
	}
    }
}

#[inline] 
fn sized_then_or<T: AsRawFd, U, F>(stream: T, trans: F) -> Result<U, T>
    where F: FnOnce(T, usize) -> U
{
    let Some(size): usize = raw_file_size(&stream).and_then(u64::into).ok() else { return Err(stream); };
    Some(trans(stream, trans))
}

#[inline] 
fn map_size_or<T: AsRawFd, U, F>(stream: T, size: usize, trans: F) -> Result<U, T>
    where F: FnOnce(MappedFile<T>, usize) -> U
{
    if try_map_to(&stream, size) {
	// Sized
	match MappedFile::try_new(stream, size, Perm::ReadWrite, Flags::Shared) {
	    Ok(map) => Ok(trans(map, size)),
	    Err(e) => Err(e.into_inner()),
	}
    } else {
	// Unsized
	Err(stream)
    }
}

/// Create an optimised call table for the cryptographic transformation from `from` to `to`.
pub fn create_process<T: AsRawFd, U: AsRawFd>(from: T, to: U) -> OpTable<T, U>
{
    let (input, buffsz) = match sized_then_or(from, |input, input_size| {
	(match MappedFile::try_new(input, input_size, Perm::Readonly, Flags::Private) {
	    Ok(m) => Ok(m),
	    Err(e) => Err(e.into_inner()),
	}, input_size)
    }) {
	Ok((i, bs)) => (i, Some(bs)),
	Err(e) => (Err(e), None),
    };
    
    let (output, outsz) = {
	if let Some(buffsz) = buffsz.or_else(|| raw_file_size(&to).ok()) {
	    match map_size_or(to, buffsz, |mmap, size| {
		(mmap, size)
	    }) {
		Ok((m, s)) => (Ok(m), Some(s)),
		Err(e) => (Err(e), if buffsz == 0 { None } else { Some(buffsz) }),
	    }
	} else {
	    Err((to, None))
	}
    };

    match ((input, buffsz), (output, outsz)) {
	// Check for all combinations of mapping successes or failures
	((Ok(min), isz), (Ok(mout), osz)) => OpTable::Both(min, mout),
	((Ok(min), isz), (Err(sout), osz)) => OpTable::Input(min, create_sized_temp_mapping(isz.or(osz)), sout),
	((Err(sin), isz), (Ok(mout), osz)) => OpTable::Output(sin, create_sized_temp_mapping(osz.or(isz)), mout),
	((Err(sin, isz), (Err(sout), osz))) => OpTable::Neither(sin, sout),
    }
}

    #[cfg(feature="try_process-old")] 
    const _:() = {
pub fn try_process(mut mode: impl BorrowMut<Crypter>) -> io::Result<io::Result<()>>
{
    let mode = mode.borrow_mut();
    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();

    let file_size = raw_file_size(&stdin)?;

    macro_rules! attempt_mapping {
	($file:expr, $perm:expr, $flags:expr) => {
	    {
		let file = $file;
		if try_map_to(&file, file_size) {
		    MappedFile::try_new(file, file_size, $perm, $flags).map_err(|e| e.into_inner())
		} else {
		    return Err(io::Error::new(io::ErrorKind::InvalidData, concat!("Failed to truncate ", stringify!($file), " to size")));
		}
	    }
	};
	($file:expr, $perm:expr) => {
	    attempt_mapping($file, $perm, Flags::default())
	};
    }
    // Try map stdout
    Ok(match attempt_mapping!(stdout, Perm::ReadWrite, Flags::Shared) {
	Ok(mut mstdout) => {
	    let res = match try_map_sized(stdin, mapped_file::Perm::Readonly, mapped_file::Flags::Private)
	    {
		Ok(mut mstdin) => {
		    // Stdin and stdout are mapped. (3)
		    process_mapped_files(&mut mode, &mut mstdin, &mut mstdout)
		},
		Err(_) => {
		    // Only stdout is mapped. (1)
		    let size = file_size as usize;
		    let is_huge = size >= 1024*1024;
		    if is_huge {
			let (mapped_memfd, _) = create_sized_temp_mapping(size)?;
			    io::copy(&mut stdin, &mut &mapped_memfd[..]).and_then(move |_| mapped_memfd)
		    } else {
			MemoryFile::with_size(size).or_else(|_| MemoryFile::new()).and_then(|mut memfd| {
			    let size = io::copy(&mut stdin, &mut memfd)?;
			    MappedFile::new(memfd, size as usize, Perm::ReadWrite, Flags::Shared)
			})
		    }.and_then(move |mut mapped_memfd| {
			process_mapped_files(mode, &mut mapped_memfd, &mut mstdout)
		    })
		    //todo!("Copy stdin into a memory-file, and then map that and process it to stdout (XXX: What if the file is too large, but we cannot tell the size of stdin?)")
		}
	    };
	    if res.is_ok() {
		mstdout.flush(mapped_file::Flush::Wait)
	    } else {
		res
	    }
	},
	Err(mut stdout) => {
	    match try_map_sized(stdin, mapped_file::Perm::Readonly, mapped_file::Flags::Private)
	    {
		Ok(mut mstdin) => {
		    // Only stdin is mapped. (2)
		    if cfg!(feature="sodiumoxide") {
			todo!("XXX: When we switch to `sodiumoxide`, we won't *need* this, we can mutate the *private* stdin mapping directly.")
		    } else {
			//TODO: XXX: When we switch to `sodiumoxide`, we won't *need* this, we can mutate the *private* stdin mapping directly.

			
			//todo!("Create a memory file (possibly with hugetlb, depending on `file_size`), map that, process_mapped_files(...) into that memory file, then `io::copy(&mut &memfile[..], &stdout)`")
			let (mapped_memfd, is_huge) = create_sized_temp_mapping(file_size as usize)?;
			process_mapped_files(&mut mode, &mut mstdin, &mut mapped_memfd).and_then(move |_| {
			    if is_huge {
				// Cannot use fd syscalls on `MFD_HUGELB`, copy into stdout from the mapped buffer itself.
				io::copy(&mut &mapped_file[..], &mut stdout)
			    } else {
				// Sync the contents into the memory file then use fd syscalls to copy into stdout.
				mapped_memfd.flush(mapped_file::Flush::Wait).and_then(move |_| {
				    let mut memfd = mapped_memfd.into_inner();
				    io::copy(&mut memfd, &mut stdout)
				})
			    }
			}).map(|_| ())
		    }
		},
		Err(_) => {
		    // Neither are mapped. (0)
		    return Err(io::Error::new(io::ErrorKind::InvalidInput, "cannot map stdin or stdout"));
		}
	    }
	},
    })
    /*let mstdin = Mapped::try_new(input)?;
	let mstdout = Mapped::try_new(output)?;*/ //TODO: if failed to map output, but input is successful (TODO: XXX: implement other way around), we can fall back to wrapping output in `Sink`.
    //todo!("Try to map the stdin and stdout streams, if that fails, return Err(last_os_err).");
    //todo!("return Ok(process_mapped_files(mode, mstdin, mstdout, key, iv))")
}
    };
