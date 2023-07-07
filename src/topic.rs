use git2::Oid;

pub struct Topic {
    oid: Oid,
}

impl Topic {
    pub fn from_oid(oid: Oid) -> Topic {
        Topic { oid: oid }
    }
}
