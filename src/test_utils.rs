use std::collections::HashMap;
use std::fs::{create_dir_all, remove_file, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};

use crate::alias::ModuleAlias;
use crate::loader::{load_program, WholeProgram};

pub struct TestFilesCreator {
    temp_workdir: TempDir,
    main_path: PathBuf,
}

impl TestFilesCreator {
    #![allow(clippy::new_without_default)]
    pub fn new() -> TestFilesCreator {
        let workdir = tempdir().unwrap();
        let main_path = workdir.path().join("main.frisbee");
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
        assert!(
            name.clone().into().ends_with(".frisbee"),
            "File to create must end with .frisbee, but {} given",
            name.clone().into()
        );

        let mut file_path = self.temp_workdir.path().to_owned();
        file_path.extend(name.clone().into().split("/"));

        if file_path.exists() {
            remove_file(file_path.as_path()).unwrap();
        }

        create_dir_all(file_path.parent().unwrap()).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(contents.into().as_bytes())
            .expect("Error on writing test file");
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

fn split_to_files(s: &str) -> HashMap<String, String> {
    let mut res: HashMap<String, String> = HashMap::new();

    for group in s.split("===== file:") {
        if !group.contains(".frisbee") {
            continue;
        }
        let (file, contents) = group.split_once("\n").expect("Error unwrapping test file content");
        res.insert(file.trim().into(), contents.trim().into());
    }

    res
}

pub fn setup_and_load_program(s: &str) -> WholeProgram {
    // Note that temporary dir is removed together with TestFeildCreator, meaning
    // that after returning Whole
    let files = split_to_files(s);
    assert!(
        files.contains_key("main.frisbee"),
        "Please make sure main.frisbee is loaded!"
    );

    let mut t = TestFilesCreator::new();
    for (key, value) in files {
        t.set_file(key, value);
    }
    t.load_program()
}

pub fn new_alias(module: &str) -> ModuleAlias {
    // NOTE: this does not work for module.submodule right now
    ModuleAlias::new(&vec![module.to_owned()])
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

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

    #[test]
    #[should_panic]
    fn test_that_file_must_end_in_frisbee() {
        let mut t = TestFilesCreator::new();
        t.set_file("main.other", "");
    }

    #[test]
    fn test_setup_and_load_program() {
        let mut t = TestFilesCreator::new();
        t.set_file("main.frisbee", "from sub.mod import Type;\n\nclass Main {}");
        t.set_file("mod.frisbee", "fun Nil hello_world() {}");
        t.set_file("sub/mod.frisbee", "active Type {}");

        let wp = t.load_program();

        let main_prog = read_to_string(wp.workdir.join("main.frisbee"));
        let mod_prog = read_to_string(wp.workdir.join("mod.frisbee"));
        let sub_mod_prog = read_to_string(wp.workdir.join("sub/mod.frisbee"));

        assert_eq!(
            main_prog.unwrap(),
            "from sub.mod import Type;\n\nclass Main {}".trim()
        );
        assert_eq!(mod_prog.unwrap(), r#"fun Nil hello_world() {}"#.trim());
        assert_eq!(sub_mod_prog.unwrap(), r#"active Type {}"#.trim());
    }
}
