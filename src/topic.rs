#[derive(Debug)]
pub struct Topic {
    name: String,
}

impl Topic {
    pub fn from_name(name: &str) -> Topic {
        Topic {
            name: name.to_string(),
        }
    }
}
