#![feature(custom_derive)]
extern crate toml;
extern crate rustc_serialize;

use std::env;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use toml::Value;

#[derive(Debug, RustcDecodable)]
 struct Package {
     name: String,
     source: String,
     version: String,
     dependencies: Option<Vec<String>>
}

#[derive(Debug, RustcDecodable)]
 struct Root {
     name: String,
     version: String,
     dependencies: Option<Vec<String>>
}

#[derive(Debug, RustcDecodable)]
 struct CargoLock {
     root: Root,
     package: Option<Vec<Package>>
}

fn parse_cargo_lock() -> io::Result<toml::Value> {
    let mut f = try!(File::open("Cargo.lock"));
    let mut buf = String::new();
    try!(f.read_to_string(&mut buf));

    Ok(try!(buf.parse().ok().ok_or(io::Error::new(io::ErrorKind::Other, "Unale to parse cargo.toml"))))
}

fn get_dependency_list() -> Vec<Package> {
    let value = parse_cargo_lock().ok().expect("Unable to open cargo lock");
    let lock: Option<CargoLock> = toml::decode(value);

    match lock {
        Some(lock) => lock.package.unwrap_or(Vec::new()),
        None => Vec::new()
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
    let dependencies = get_dependency_list();

    let root = env::home_dir().unwrap_or(Path::new("/").to_path_buf());
    let cargo_dir = root.join(".cargo/registry/src");

    let mut cmd = Command::new("ctags");

    cmd.arg("-R");
    cmd.arg(".");

    for dep in dependencies {
        match find_dependency_dir(&cargo_dir, &dep) {
            Ok(dir) => { cmd.arg(dir.as_os_str()); },
            Err(_) => {}
        }
    }

    cmd.output().ok().expect("Couldn't run ctags");
}
