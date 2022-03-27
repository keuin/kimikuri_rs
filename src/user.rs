use std::fmt;
use std::fmt::Formatter;

pub struct User {
    pub id: u64,
    pub name: String,
    pub token: String,
    pub chat_id: u64,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "User {{ id={}, name={}, token={}, chat_id={} }}",
               self.id, self.name, self.token, self.chat_id)
    }
}