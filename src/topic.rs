#[derive(Debug)]
#[allow(dead_code)]
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
