use super::IsrStackFrame;

pub fn handler(_data: *const IsrStackFrame) {
    panic!("Division by zero!!");
}