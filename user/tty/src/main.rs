#![feature(format_args_nl)]
#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::{borrow::ToOwned, string::String, vec};
// use core::arch::asm;
// use core::sync::atomic::{AtomicUsize, Ordering};
use heap::UserHeap;
use syscall::UserLogger;

#[global_allocator]
static ALLOCATOR: UserHeap = UserHeap::new();

struct TTY {
    fd: usize,
}

impl TTY {
    fn new() -> Self {
        let fd = vfs::open("/dev/tty.serial");
        assert!(fd != -1);
        Self { fd: fd as _ }
    }

    fn prompt(&self) -> String {
        print!("> ");
        self.readline()
    }

    fn read_byte(&self) -> u8 {
        let mut buf = [0u8; 1];
        let len = vfs::read(self.fd, &mut buf);
        assert!(len > 0);
        let mut c = buf[0];
        if c == 127 {
            c = 8;
            buf[0] = c;
            let s = core::str::from_utf8(&buf).unwrap();
            print!("{} {}", s, s);
        } else {
            let s = core::str::from_utf8(&buf).unwrap();
            print!("{}", s);
        }
        c
    }

    fn readline(&self) -> String {
        let mut buf = vec![];
        loop {
            let c = self.read_byte();
            if c == 8 {
                buf.pop();
            } else if c == b'\n' {
                break;
            } else {
                buf.push(c);
            }
        }
        core::str::from_utf8(&buf).unwrap().to_owned()
    }

    pub fn run(&self) {
        log!("[[Sophon TTY]]");
        loop {
            let cmd = self.prompt();
            // println!("{:?}", cmd);
            if cmd == "exit" {
                break;
            }
            syscall::exec(&cmd);
        }
        log!("Sophon TTY exited.");
    }
}

#[no_mangle]
pub extern "C" fn _start(_argc: isize, _argv: *const *const u8) -> isize {
    UserLogger::init();
    ALLOCATOR.init();
    let tty = TTY::new();
    tty.run();
    syscall::exit()
}

#[panic_handler]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    log!("{}", info);
    syscall::exit();
}
