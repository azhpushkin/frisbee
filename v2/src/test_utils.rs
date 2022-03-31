use std::collections::HashMap;
use std::fs::{remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

use crate::ast::{ModulePath, ModulePathAlias};
use crate::loader::{load_program, WholeProgram};

pub struct TestFilesCreator {
    temp_workdir: TempDir,
    main_path: PathBuf,
}

impl TestFilesCreator {
    pub fn new() -> TestFilesCreator {
        let workdir = tempdir().unwrap();
        let main_path = workdir.path().clone().join("main.frisbee");
        TestFilesCreator { temp_workdir: workdir, main_path }
    }

    pub fn set_mainfile<S>(&mut self, contents: S)
    where
        S: Into<String> + Clone,
    {
        self.set_file("main.frisbee", contents);
    }

    pub fn set_file<N, C>(&mut self, name: N, contents: C)
    where
        N: Into<String> + Clone,
        C: Into<String> + Clone,
    {
        let file_path = self.temp_workdir.path().join(name.into());
        if file_path.exists() {
            remove_file(file_path.as_path()).unwrap();
        }

        let mut file = File::create(file_path).unwrap();
        file.write(contents.into().as_bytes()).unwrap();
    }

    pub fn get_main_path(&self) -> &Path {
        self.main_path.as_path()
    }

    pub fn load_program(&self) -> WholeProgram {
        let p = load_program(self.get_main_path());
        assert!(p.is_some(), "Loading error!");
        p.unwrap()
    }
}

pub fn split_to_files(s: &str) -> HashMap<String, String> {
    let mut res: HashMap<String, String> = HashMap::new();

    for group in s.split("===== file:") {
        if !group.contains(".frisbee") {
            continue;
        }
        let (file, contents) = group
            .split_once("\n")
            .expect("Error unwrapping test file content");
        res.insert(file.trim().into(), contents.trim().into());
    }

    res
}

pub fn setup_and_load_program(s: &str) -> WholeProgram {
    let files = split_to_files(s);
    if !files.contains_key("main.frisbee") {
        panic!("Please make sure main.frisbee is loaded!");
    }

    let mut t = TestFilesCreator::new();
    for (key, value) in files {
        t.set_file(key, value);
    }
    return t.load_program();
}

pub fn new_alias(module: &str) -> ModulePathAlias {
    // NOTE: this does not work for module.submodule right now
    ModulePath(vec![module.to_string()]).alias()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_to_files() {
        let res = split_to_files(
            r#"
            ===== file: main.frisbee
            from sub.mod import Type;

            class Main {}
            ===== file: sub/mod.frisbee
            active Type {}
        "#,
        );

        assert_eq!(res.len(), 2);
        assert_eq!(
            res.get("main.frisbee").unwrap(),
            r#"
            from sub.mod import Type;

            class Main {}
            "#
            .trim()
        );
        assert_eq!(
            res.get("sub/mod.frisbee").unwrap(),
            r#"
            active Type {}
            "#
            .trim()
        );
    }
}
