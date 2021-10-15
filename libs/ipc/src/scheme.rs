pub use crate::uri::*;
use crate::{
    syscall::{self, Syscall},
    Message,
};
use core::{
    intrinsics::transmute,
    sync::atomic::{AtomicUsize, Ordering},
};
use core::{slice, str};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[repr(usize)]
pub enum Error {
    NotFound,
    Other,
}

pub type Result<T> = core::result::Result<T, Error>;

#[repr(usize)]
pub enum SchemeRequest {
    Register = 0,
    Open,
    Close,
    FStat,
    LSeek,
    Read,
    Write,
    // Stat,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whence {
    Set,
    Cur,
    End,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

#[repr(C)]
pub struct Args<'a> {
    buf: &'a [u8],
    index: AtomicUsize,
}

impl<'a> Args<'a> {
    #[inline]
    pub fn new<T: Sized>(t: T) -> impl AsRef<[u8]> {
        struct X<T: Sized>(T);
        impl<T: Sized> AsRef<[u8]> for X<T> {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                let ptr = &self.0 as *const T as *const u8;
                let size = core::mem::size_of::<T>();
                unsafe { slice::from_raw_parts(ptr, size) }
            }
        }
        X(t)
    }

    #[inline]
    pub fn from(buf: &'a [u8]) -> Self {
        Self {
            buf,
            index: AtomicUsize::new(0),
        }
    }

    #[inline]
    pub fn get<T>(&self) -> T {
        let align = core::mem::align_of::<T>();
        let mut i = self.index.load(Ordering::SeqCst);
        i = (i + align - 1) & !(align - 1);
        self.index
            .store(i + core::mem::size_of::<T>(), Ordering::SeqCst);
        unsafe { core::ptr::read(self.buf.as_ptr().add(i) as *const T) }
    }

    #[inline]
    pub fn get_slice<T>(&self) -> &[T] {
        let ptr = self.get::<*const u8>() as *const T;
        let size = self.get::<usize>();
        let len = size / core::mem::size_of::<T>();
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    #[inline]
    pub fn get_mut_slice<T>(&self) -> &mut [T] {
        let ptr = self.get::<*mut u8>() as *mut T;
        let size = self.get::<usize>();
        let len = size / core::mem::size_of::<T>();
        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    #[inline]
    pub fn get_str(&self) -> Option<&str> {
        let ptr = self.get::<*const u8>();
        let size = self.get::<usize>();
        unsafe { str::from_utf8(slice::from_raw_parts(ptr, size)).ok() }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Resource(pub usize);

impl Resource {
    #[inline]
    pub fn open(uri: impl AsUri, flags: u32, mode: Mode) -> Result<Resource> {
        let uri = uri.as_str();
        let uri_ptr = uri.as_ptr() as *const u8;
        let uri_len = uri.len();
        let fd = unsafe {
            syscall::syscall(
                Syscall::SchemeRequest,
                &[
                    transmute(SchemeRequest::Open),
                    transmute(uri_ptr),
                    transmute(uri_len),
                    transmute(flags as usize),
                    transmute(mode),
                ],
            )
        };
        Ok(Resource(fd as _))
    }

    #[inline]
    pub fn close(self) -> Result<()> {
        unimplemented!()
    }

    #[inline]
    pub fn stat(&self) -> Result<()> {
        unimplemented!()
    }

    #[inline]
    pub fn lseek(&self, _offset: isize, _whence: Whence) -> Result<()> {
        unimplemented!()
    }

    #[inline]
    pub fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let r = unsafe {
            syscall::syscall(
                Syscall::SchemeRequest,
                &[
                    transmute(SchemeRequest::Read),
                    transmute(*self),
                    transmute(buf.as_mut_ptr()),
                    transmute(buf.len()),
                ],
            )
        };
        if r < 0 {
            return Err(Error::Other);
        }
        Ok(r as _)
    }

    #[inline]
    pub fn write(&self, buf: impl AsRef<[u8]>) -> Result<()> {
        let buf = buf.as_ref();
        let _ = unsafe {
            syscall::syscall(
                Syscall::SchemeRequest,
                &[
                    transmute(SchemeRequest::Write),
                    transmute(*self),
                    transmute(buf.as_ptr()),
                    transmute(buf.len()),
                ],
            )
        };
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SchemeId(pub usize);

pub trait SchemeServer {
    fn name(&self) -> &str;
    fn open(&self, uri: &Uri, flags: u32, mode: Mode) -> Result<Resource>;
    fn close(self, fd: Resource) -> Result<()>;
    fn stat(&self, _fd: Resource) -> Result<()> {
        unimplemented!()
    }
    fn lseek(&self, _fd: Resource, _offset: isize, _whence: Whence) -> Result<()> {
        unimplemented!()
    }
    fn read(&self, fd: Resource, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, fd: Resource, buf: &[u8]) -> Result<()>;

    // Helpers
    fn allocate_resource_id(&self) -> Resource {
        allocate_resource()
    }
}

fn allocate_resource() -> Resource {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    Resource(COUNTER.fetch_add(1, Ordering::SeqCst))
}

pub fn register_user_scheme(scheme: &'static impl SchemeServer) -> ! {
    let _ = unsafe {
        syscall::syscall(
            Syscall::SchemeRequest,
            &[
                transmute(SchemeRequest::Register),
                transmute(&scheme.name()),
            ],
        )
    };
    loop {
        let scheme_request = Message::receive(None);
        let args = scheme_request.get_data::<[usize; 5]>();
        let result = handle_user_scheme_request(scheme, args);
        scheme_request.reply(result);
    }
}

fn handle_user_scheme_request(scheme: &'static impl SchemeServer, args: &[usize; 5]) -> isize {
    match unsafe { transmute::<_, SchemeRequest>(args[0]) } {
        SchemeRequest::Register => -1,
        SchemeRequest::Open => {
            let uri = unsafe {
                let uri_ptr = transmute::<_, *const u8>(args[1]);
                let uri_len = transmute::<_, usize>(args[2]);
                let uri_str = str::from_utf8_unchecked(slice::from_raw_parts(uri_ptr, uri_len));
                Uri::new(uri_str).unwrap()
            };
            let resource = scheme
                .open(&uri, args[3] as _, unsafe { transmute(args[4]) })
                .unwrap();
            unsafe { transmute(resource) }
        }
        SchemeRequest::Close => {
            unimplemented!()
        }
        SchemeRequest::FStat => {
            unimplemented!()
        }
        SchemeRequest::LSeek => {
            unimplemented!()
        }
        SchemeRequest::Read => {
            let fd = unsafe { transmute::<_, Resource>(args[1]) };
            let buf = unsafe {
                let data = transmute::<_, *mut u8>(args[2]);
                let len = transmute::<_, usize>(args[3]);
                slice::from_raw_parts_mut(data, len)
            };
            let r = scheme.read(fd, buf).unwrap();
            unsafe { transmute(r) }
        }
        SchemeRequest::Write => {
            let fd = unsafe { transmute::<_, Resource>(args[1]) };
            let buf = unsafe {
                let data = transmute::<_, *const u8>(args[2]);
                let len = transmute::<_, usize>(args[3]);
                slice::from_raw_parts(data, len)
            };
            scheme.write(fd, buf).unwrap();
            0
        }
    }
}
