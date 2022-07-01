#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ModuleAlias(String);

impl std::fmt::Display for ModuleAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ModuleAlias {
    pub fn new(path: &[String]) -> Self {
        for subpath in path {
            if subpath.contains(".") {
                panic!("Parsing went wrong leading to dot in ModuleAlias subpath")
            }
        }

        Self(path.join("."))
    }

    pub fn std() -> Self {
        Self("std".to_owned())
    }

    pub fn to_path(&self) -> Vec<&str> {
        self.0.split('.').collect()
    }
}
