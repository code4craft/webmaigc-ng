use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDescriptor {
    pub name: &'static str,
    pub responsibility: &'static str,
}

impl ModuleDescriptor {
    pub const fn new(name: &'static str, responsibility: &'static str) -> Self {
        Self {
            name,
            responsibility,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "module={} responsibility={}",
            self.name, self.responsibility
        )
    }
}
