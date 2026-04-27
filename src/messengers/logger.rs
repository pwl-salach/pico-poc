use super::Messenger;

pub struct Logger;

impl Messenger for Logger {
    fn send_message(&mut self, message: &str) {
        defmt::info!("{}", message);
    }

    fn get_level(&self) -> super::MessageLevel {
        super::MessageLevel::Debug
    }
}
