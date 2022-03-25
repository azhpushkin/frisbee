use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

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

    pub fn add_mainfile<S>(&mut self, contents: S)
    where
        S: Into<String> + Clone,
    {
        self.add_file("main.frisbee", contents);
    }

    pub fn add_file<N, C>(&mut self, name: N, contents: C)
    where
        N: Into<String> + Clone,
        C: Into<String> + Clone,
    {
        let mut file = File::create(self.temp_workdir.path().join(name.into())).unwrap();
        file.write(contents.into().as_bytes()).unwrap();
    }

    pub fn get_main_path(&self) -> &Path {
        self.main_path.as_path()
    }
}
