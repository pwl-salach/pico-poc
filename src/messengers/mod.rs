pub mod lcd;

pub trait Messenger {
    fn send_message(&mut self, message: &str);
}
