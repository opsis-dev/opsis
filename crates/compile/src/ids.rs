const ID_ALPHABET: [char; 36] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub fn random_id(prefix: &str) -> String {
    format!("{prefix}_{}", nanoid::nanoid!(8, &ID_ALPHABET))
}

pub fn node_id_prefix(kind: &str) -> &str {
    kind.split('.').next().unwrap_or(kind)
}

pub trait Ids {
    fn id(&mut self, prefix: &str) -> String;
}

#[derive(Debug)]
pub struct RandomIds;

impl Ids for RandomIds {
    fn id(&mut self, prefix: &str) -> String {
        random_id(prefix)
    }
}

#[derive(Debug)]
pub struct SaltedIds {
    salt: String,
    count: u32,
}

impl SaltedIds {
    pub fn new(salt: &str) -> Self {
        Self {
            salt: salt.to_owned(),
            count: 0,
        }
    }
}

impl Ids for SaltedIds {
    fn id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}_{}{}", self.salt, self.count);
        self.count += 1;
        id
    }
}
