pub mod broadcaster;
pub mod lcd;
pub mod logger;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]

pub enum MessageLevel {
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

pub trait Messenger {
    fn send_message(&mut self, message: &str);

    fn get_level(&self) -> MessageLevel {
        MessageLevel::Info
    }
}
