use super::{MessageLevel, Messenger};
use heapless::Vec;

pub struct Broadcaster<'a> {
    messengers: Vec<&'a mut dyn Messenger, 8>,
}

impl<'a> Broadcaster<'a> {
    pub fn new() -> Self {
        Self {
            messengers: Vec::new(),
        }
    }

    pub fn add_messenger(&mut self, messenger: &'a mut dyn Messenger) -> Result<(), &'static str> {
        if self.messengers.push(messenger).is_err() {
            return Err("Broadcaster messenger capacity exceeded");
        }
        Ok(())
    }

    pub fn broadcast(&mut self, message: &str, level: MessageLevel) {
        for messenger in &mut self.messengers {
            if messenger.get_level() <= level {
                messenger.send_message(message);
            }
        }
    }
}
