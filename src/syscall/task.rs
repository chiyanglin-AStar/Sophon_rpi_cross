use crate::exception::ExceptionFrame;
use crate::task::*;

pub fn fork(exception_frame: &mut ExceptionFrame) -> TaskId {
    unimplemented!()
    // let parent_task = crate::task::Task::current().unwrap();
    // println!("Fork task start");
    // let child_task = parent_task.fork(exception_frame as *const ExceptionFrame as usize);
    // println!("Fork task");
    // child_task.id()
}

pub fn exit(exception_frame: &mut ExceptionFrame) -> isize {
    GLOBAL_TASK_SCHEDULER.remove_task(Task::current().unwrap().id());
    0
}
