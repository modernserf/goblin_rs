use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScopeType {
    Let,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScopeRecord {
    pub index: usize,
    pub typ: ScopeType,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    index: usize,
    map: HashMap<String, ScopeRecord>,
}

impl Scope {
    pub fn root() -> Self {
        Scope {
            index: 0,
            map: HashMap::new(),
        }
    }
    pub fn get(&self, key: &str) -> Option<ScopeRecord> {
        self.map.get(key).map(|r| r.to_owned())
    }
    pub fn add(&mut self, key: String, typ: ScopeType) -> ScopeRecord {
        let record = ScopeRecord {
            index: self.index,
            typ,
        };
        self.index += 1;
        self.map.insert(key, record);
        record
    }
}
