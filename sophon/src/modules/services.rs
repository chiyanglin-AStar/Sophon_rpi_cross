use alloc::boxed::Box;
use core::alloc::GlobalAlloc;
use core::iter::Step;
use core::ops::Range;
use device_tree::DeviceTree;
use kernel_module::ModuleCallHandler;
use log::Logger;
use memory::page::Frame;
use memory::page_table::PageFlags;
use memory::{
    address::Address,
    page::{Page, Size4K},
};
use proc::ProcId;

use crate::arch::{Arch, TargetArch};
use crate::memory::kernel::KERNEL_HEAP;
use crate::memory::kernel::KERNEL_MEMORY_MAPPER;
use crate::scheduler::monitor::SysMonitor;
use crate::scheduler::AbstractScheduler;
use crate::scheduler::SCHEDULER;
use crate::task::Proc;
use crate::task::Task;
use crate::utils::testing::Tests;

use super::raw_module_call;
use super::MODULES;
pub struct KernelService(pub usize);

impl kernel_module::KernelService for KernelService {
    fn log(&self, s: &str) {
        print!("{}", s);
    }

    fn set_sys_logger(&self, logger: &'static dyn Logger) {
        log::init(logger)
    }

    fn register_tests(&self, tests: Tests) {
        crate::utils::testing::register_kernel_tests(tests);
    }

    fn alloc(&self, layout: core::alloc::Layout) -> Option<Address> {
        let ptr = unsafe { crate::ALLOCATOR.alloc(layout) };
        if ptr.is_null() {
            None
        } else {
            Some(ptr.into())
        }
    }

    fn dealloc(&self, ptr: Address, layout: core::alloc::Layout) {
        unsafe { crate::ALLOCATOR.dealloc(ptr.as_mut_ptr(), layout) }
    }

    fn register_module_call_handler(&self, handler: &'static dyn ModuleCallHandler) {
        // log!("register module call");
        MODULES.write()[self.0].as_mut().map(|m| {
            m.call = Some(handler);
        });
    }

    fn module_call<'a>(&self, module: &str, request: syscall::RawModuleRequest<'a>) -> isize {
        raw_module_call(module, true, request.as_buf())
    }

    fn current_process(&self) -> Option<ProcId> {
        Some(Proc::current().id)
    }

    fn current_task(&self) -> Option<proc::TaskId> {
        Some(Task::current().id)
    }

    fn handle_panic(&self) -> ! {
        if cfg!(sophon_test) {
            TargetArch::halt(-1)
        }
        syscall::exit();
    }

    fn get_device_tree(&self) -> Option<&'static DeviceTree<'static, 'static>> {
        unsafe { crate::DEV_TREE.as_ref() }
    }

    fn map_device_page(&self, frame: Frame) -> Page {
        let page = KERNEL_HEAP.virtual_allocate::<Size4K>(1).start;
        KERNEL_MEMORY_MAPPER.map(page, frame, PageFlags::device());
        page
    }

    fn map_device_pages(&self, frames: Range<Frame>) -> Range<Page> {
        let num_pages = Step::steps_between(&frames.start, &frames.end).unwrap();
        let pages = KERNEL_HEAP.virtual_allocate::<Size4K>(num_pages);
        for i in 0..num_pages {
            let frame = Step::forward(frames.start, i);
            let page = Step::forward(pages.start, i);
            KERNEL_MEMORY_MAPPER.map(page, frame, PageFlags::device());
        }
        pages
    }

    fn set_irq_handler(&self, irq: usize, handler: Box<dyn Fn() -> isize>) {
        TargetArch::interrupt().set_irq_handler(irq, handler);
    }

    fn enable_irq(&self, irq: usize) {
        TargetArch::interrupt().enable_irq(irq);
    }

    fn disable_irq(&self, irq: usize) {
        TargetArch::interrupt().disable_irq(irq);
    }

    fn set_interrupt_controller(&self, controller: &'static dyn interrupt::InterruptController) {
        TargetArch::set_interrupt_controller(controller);
    }

    fn schedule(&self) -> ! {
        TargetArch::interrupt().notify_end_of_interrupt();
        SCHEDULER.timer_tick();
        unreachable!()
    }

    fn new_monitor(&self) -> mutex::Monitor {
        mutex::Monitor::new(SysMonitor::new())
    }
}
