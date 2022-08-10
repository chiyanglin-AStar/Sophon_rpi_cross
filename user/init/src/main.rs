#![feature(default_alloc_error_handler)]
#![no_std]
#![no_main]

use user::sys::syscall::NetRequest;

#[macro_use]
extern crate user;

// static COUNTER: AtomicUsize = AtomicUsize::new(0);

// extern "C" fn thread_start() {
//     log!("thread_start");
//     for _ in 0..10 {
//         COUNTER.fetch_add(1, Ordering::SeqCst);
//         for _ in 0..100000 {
//             unsafe {
//                 asm!("");
//             }
//         }
//         log!(" - {}", COUNTER.load(Ordering::SeqCst));
//     }
//     exit_thread();
// }

// fn exit_thread() {
//     Resource::open("proc:/me/thread-exit", 0, Mode::ReadWrite)
//         .unwrap()
//         .write(&[])
//         .unwrap();
// }

// fn spawn_thread(f: *const extern "C" fn()) {
//     Resource::open("proc:/me/spawn-thread", 0, Mode::ReadWrite)
//         .unwrap()
//         .write(Args::new(f))
//         .unwrap();
// }

#[no_mangle]
pub extern "C" fn _start(_argc: isize, _argv: *const *const u8) -> isize {
    println!("Init process start...");
    // println!("Test fs read...");
    // let file = user::sys::open("/etc/hello.txt").unwrap();
    // let mut buf = [0u8; 32];
    // let len = user::sys::read(file, &mut buf).unwrap();
    // let s = core::str::from_utf8(&buf[0..len]);
    // println!("read: {:?}", s);
    // println!("Launch tty...");
    // user::sys::exec("/bin/tty", &[]);
    // user::sys::exit()
    user::sys::syscall::module_call("virtio-net", &NetRequest::Test);
    println!("Init loop");
    loop {}
}
