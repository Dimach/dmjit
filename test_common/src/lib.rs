use std::process::{Command, ExitStatus};
use std::fs::File;
use std::io::Write;
use std::{io, fs};
use std::path::{Path, PathBuf};

pub struct DMTest<'l> {
    name: &'l str,
    work_dir: &'l str,
    test_data_search_path: Vec<&'l str>,
}


impl <'l> DMTest<'l> {
    pub fn new(name: &'l str, work_dir: &'l str, test_data_search_path: Vec<&'l str>) -> Self {
        Self {
            name,
            work_dir,
            test_data_search_path
        }
    }

    fn work_dir(&self) -> PathBuf {
        Path::new(self.work_dir).join(self.name)
    }

    pub fn run_hook_test(&self, files: Vec<&str>) {
        fs::remove_file(self.work_dir().join("result.txt")).ok();
        fs::remove_file(self.work_dir().join("world.txt")).ok();
        let mut all_files = vec!("common.dm", "hook.dm", "bench.dm");
        all_files.extend(files);
        assert!(self.compile_files(all_files).unwrap().success());
        assert_eq!(self.link_hook_lib().unwrap(), ());
        assert!(self.run_dm().unwrap().success());
    }

    pub fn run_hook_and_assert_result(&self, files: Vec<&str>) {
        self.run_hook_test(files.clone());

        let mut expected_lines: Vec<String> = Vec::new();
        for file in files.iter() {
            let str = fs::read_to_string(self.work_dir().join(file)).unwrap();
            let mut do_test_found = false;
            for line in str.lines() {
                if line.starts_with("/proc/do_test") {
                    do_test_found = true;
                }

                if do_test_found {
                    let pos = line.rfind(RES_PREFIX);
                    if let Some(pos) = pos {
                        let text = &line[(pos + RES_PREFIX.len())..];
                        expected_lines.push(text.trim().to_string());
                    }
                }

            }

            if do_test_found {
                break;
            }
        }
        assert_eq!(self.test_result().trim(), expected_lines.join("\n"));
    }

    pub fn test_result(&self) -> String {
        let path = self.work_dir().join("result.txt");
        let res = fs::read_to_string(path).unwrap();
        res.replace("\r\n","\n")
    }

    fn compile_files(&self, files: Vec<&str>) -> io::Result<ExitStatus> {
        let test_dme_path = self.work_dir().join("test.dme");
        std::fs::create_dir_all(test_dme_path.parent().unwrap())?;
        let mut dme = File::create(test_dme_path.clone())?;

        writeln!(dme, "#define DMJIT_LIB \"{}\"", lib_path().file_name().unwrap().to_str().unwrap())?;

        for file_name in files {
            let src = self.test_data_search_path.iter()
                .filter_map(|search_path| {
                    let file = Path::new(search_path).join(file_name);
                    if file.exists() {
                        Option::Some(file)
                    } else {
                        Option::None
                    }
                }).next().unwrap();
            let target = self.work_dir().join(file_name);
            std::fs::create_dir_all(target.parent().unwrap())?;
            std::fs::copy(src, target)?;
            writeln!(dme, "#include \"{}\"", file_name)?;
        }


        cmd_dm()
            .arg(test_dme_path.to_str().unwrap())
            .current_dir(self.work_dir())
            .spawn()
            .unwrap()
            .wait()
    }

    #[cfg(unix)]
    fn link_hook_lib(&self) -> io::Result<()> {
        io::Result::Ok(()) // On linux, shared library is always on path
    }

    #[cfg(windows)]
    fn link_hook_lib(&self)  -> io::Result<()> {
        let lib_path = lib_path();
        std::fs::copy(lib_path, self.work_dir().join(lib_path.file_name().unwrap().to_str().unwrap()))?; // TODO: avoid copying whole DLL if possible
        io::Result::Ok(())
    }

    fn run_dm(&self) -> io::Result<ExitStatus> {
        cmd_dreamdaemon()
            .current_dir(self.work_dir())
            .arg(self.work_dir().join("test.dmb").to_str().unwrap())
            .arg("-trusted")
            .spawn()?
            .wait()
    }

}

const RES_PREFIX: &str = "// RES:";

fn lib_path() -> &'static Path {
    if cfg!(target_os = "windows") {
        Path::new("target/i686-pc-windows-msvc/debug/dmjit.dll")
    } else {
        Path::new("target/i686-unknown-linux-gnu/debug/libdmjit.so")
    }
}

fn cmd_dm() -> Command {
    if cfg!(target_os = "windows") {
        Command::new(format!("{}\\bin\\dm.exe", byond_path()))
    } else {
        let mut cmd = Command::new(format!("{}/bin/byondexec", byond_path()));
        cmd.arg(format!("{}/bin/DreamMaker", byond_path()));
        cmd
    }
}

fn byond_path() -> String {
    let byond_path = std::env::var("BYOND_PATH");
    if cfg!(target_os = "windows") {
        byond_path.unwrap_or("C:\\Program Files (x86)\\BYOND".to_string())
    } else {
        byond_path.unwrap()
    }
}

fn cmd_dreamdaemon() -> Command {
    if cfg!(target_os = "windows") {
        Command::new(format!("{}\\bin\\dreamdaemon.exe", byond_path()))
    } else {
        let mut cmd = Command::new(format!("{}/bin/byondexec", byond_path()));
        cmd.arg(format!("{}/bin/DreamDaemon", byond_path()));
        cmd
    }
}