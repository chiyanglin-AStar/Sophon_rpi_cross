#[allow(unused)]
use core::arch::asm;
use core::intrinsics::transmute;

use crate::ModuleRequest;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Syscall {
    Log,
    ModuleCall,
    Wait,
    Sbrk,
    Exec,
    Exit,
    ThreadExit,
    Halt,
    MutexCreate,
    MutexLock,
    MutexUnlock,
    MutexDestroy,
    CondvarCreate,
    CondvarWait,
    CondvarNotifyAll,
    CondvarDestroy,
}

#[inline]
#[cfg(target_arch = "x86_64")]
pub fn syscall(_syscall: Syscall, _args: &[usize]) -> isize {
    unimplemented!()
}

#[inline]
#[cfg(target_arch = "aarch64")]
pub fn syscall(syscall: Syscall, args: &[usize]) -> isize {
    debug_assert!(args.len() <= 6);
    let a: usize = args.get(0).cloned().unwrap_or(0);
    let b: usize = args.get(1).cloned().unwrap_or(0);
    let c: usize = args.get(2).cloned().unwrap_or(0);
    let d: usize = args.get(3).cloned().unwrap_or(0);
    let e: usize = args.get(4).cloned().unwrap_or(0);
    let ret: isize;
    unsafe {
        asm!("svc #0",
            inout("x0") syscall as usize => ret,
            in("x1") a, in("x2") b, in("x3") c, in("x4") d, in("x5") e,
        );
    }
    ret
}

#[inline]
pub fn log(message: &str) {
    syscall(Syscall::Log, &[&message as *const &str as usize]);
}

#[inline]
pub fn module_call<'a>(module: &str, request: &'a impl ModuleRequest<'a>) -> isize {
    unsafe {
        let name = &module as *const &str;
        let args = request.as_raw().as_buf();
        syscall(
            Syscall::ModuleCall,
            &[transmute(name), args[0], args[1], args[2], args[3]],
        )
    }
}

#[inline]
pub fn wait() -> isize {
    syscall(Syscall::Wait, &[])
}

#[inline]
pub fn exec(path: &str, args: &[&str]) -> isize {
    let path = &path as *const &str;
    let args = &args as *const &[&str];
    unsafe { syscall(Syscall::Exec, &[transmute(path), transmute(args)]) }
}

#[inline]
pub fn exit() -> ! {
    syscall(Syscall::Exit, &[]);
    unreachable!()
}

#[inline]
pub fn thread_exit() -> ! {
    syscall(Syscall::ThreadExit, &[]);
    unreachable!()
}

#[inline]
pub fn halt(code: usize) -> ! {
    syscall(Syscall::Halt, &[code]);
    unreachable!()
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpaqueMutexPointer(*mut ());

#[inline]
pub fn mutex_create() -> OpaqueMutexPointer {
    let r = syscall(Syscall::MutexCreate, &[]);
    OpaqueMutexPointer(r as _)
}

#[inline]
pub fn mutex_lock(mutex: OpaqueMutexPointer) -> isize {
    syscall(Syscall::MutexLock, &[mutex.0 as _])
}

#[inline]
pub fn mutex_unlock(mutex: OpaqueMutexPointer) -> isize {
    syscall(Syscall::MutexUnlock, &[mutex.0 as _])
}

#[inline]
pub fn mutex_destroy(mutex: OpaqueMutexPointer) -> isize {
    syscall(Syscall::MutexDestroy, &[mutex.0 as _])
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpaqueCondvarPointer(*mut ());

#[inline]
pub fn condvar_create() -> OpaqueCondvarPointer {
    let r = syscall(Syscall::CondvarCreate, &[]);
    OpaqueCondvarPointer(r as _)
}

#[inline]
pub fn condvar_wait(cvar: OpaqueCondvarPointer, mutex: OpaqueMutexPointer) -> isize {
    syscall(Syscall::CondvarWait, &[cvar.0 as _, mutex.0 as _])
}

#[inline]
pub fn condvar_notify_all(cvar: OpaqueCondvarPointer) -> isize {
    syscall(Syscall::CondvarNotifyAll, &[cvar.0 as _])
}

#[inline]
pub fn condvar_destory(cvar: OpaqueCondvarPointer) -> isize {
    syscall(Syscall::CondvarDestroy, &[cvar.0 as _])
}
