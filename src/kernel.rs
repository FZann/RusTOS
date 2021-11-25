mod assembly;
mod vectors;

pub mod processes;
pub mod scheduler;

pub enum SysCallType {
    ContextSwitch
}