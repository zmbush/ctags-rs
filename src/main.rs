#![feature(custom_derive)]
extern crate toml;
extern crate rustc_serialize;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, RustcDecodable, Default)]
 struct Package {
     name: String,
     source: String,
     version: String,
     dependencies: Option<Vec<String>>
}

#[derive(Debug, RustcDecodable, Default)]
 struct Root {
     name: String,
     version: String,
     dependencies: Option<Vec<String>>
}

#[derive(Debug, RustcDecodable, Default)]
 struct CargoLock {
     root: Root,
     package: Option<Vec<Package>>
}

fn parse_cargo_lock() -> io::Result<CargoLock> {
    let mut f = try!(File::open("Cargo.lock"));
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));

    match toml::decode_str(&buf) {
        Some(v) => Ok(v),
        None => Ok(CargoLock::default())
    }
}

fn get_dependency_list() -> Vec<Package> {
    match parse_cargo_lock() {
        Ok(lock) => lock.package.unwrap_or(Vec::new()),
        Err(_) => {
            println!("No Cargo.lock found");
            Vec::new()
        }
    }
}

fn find_dependency_dir(dir: &Path, package: &Package) -> io::Result<PathBuf> {
    let mut fringe = vec![dir.to_path_buf()];
    while fringe.len() > 0 {
        let dir = fringe.remove(0);
        let meta = try!(fs::metadata(&dir));
        if meta.is_dir() {
            let f = dir.file_name()
                .expect(&format!("{} has no file_name", dir.display()))
                .to_string_lossy();

            if f == format!("{}-{}", package.name, package.version) {
                return Ok(dir.to_path_buf().clone())
            } else {
                for entry in try!(fs::read_dir(&dir)) {
                    let entry = try!(entry);
                    fringe.push(entry.path());
                }
            }
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Not found"))
}

fn main() {
    let mut cmd = Command::new("ctags");

    cmd.arg("-R");
    cmd.arg(".");

    let rust_ctag_config = include_str!("rust.ctags");
    for config in rust_ctag_config.split("\n") {
        cmd.arg(config);
    }

    let dependencies = get_dependency_list();

    if dependencies.len() > 0 {
        let root = env::home_dir().unwrap_or(Path::new("/").to_path_buf());
        let cargo_dir = root.join(".cargo/registry/src");

        for dep in dependencies {
            match find_dependency_dir(&cargo_dir, &dep) {
                Ok(dir) => {
                    println!("Found cargo dependency: {}-{}", dep.name, dep.version);
                    cmd.arg(dir.as_os_str());
                },
                Err(_) => {}
            }
        }
    }

    cmd.output().ok().expect("Couldn't run ctags");
}
