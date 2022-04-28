#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ModuleAlias(String);

impl std::fmt::Display for ModuleAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ModuleAlias {
    pub fn new(path: &[String]) -> Self {
        Self(path.join("."))
    }

    pub fn std() -> Self {
        Self("std".to_owned())
    }
}
