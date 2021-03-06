use std::os;
use std::io;
use std::io::{fs, File};

use util::{CargoResult, human, ChainError, process};
use core::shell::MultiShell;

macro_rules! git( ($($a:expr),*) => ({
    process("git") $(.arg($a))* .exec_with_output()
}) )

pub struct NewOptions<'a> {
    pub git: bool,
    pub bin: bool,
    pub path: &'a str,
}

pub fn new(opts: NewOptions, _shell: &mut MultiShell) -> CargoResult<()> {
    let path = os::getcwd().join(opts.path);
    if path.exists() {
        return Err(human(format!("Destination `{}` already exists",
                                 path.display())))
    }
    let name = path.filename_str().unwrap();
    mk(&path, name, &opts).chain_error(|| {
        human(format!("Failed to create project `{}` at `{}`",
                      name, path.display()))
    })
}

fn mk(path: &Path, name: &str, opts: &NewOptions) -> CargoResult<()> {

    if opts.git {
        try!(git!("init", path));
        let mut gitignore = "/target\n".to_string();
        if !opts.bin {
            gitignore.push_str("/Cargo.lock\n");
        }
        try!(File::create(&path.join(".gitignore")).write(gitignore.as_bytes()));
    } else {
        try!(fs::mkdir(path, io::UserRWX));
    }

    let author = try!(discover_author());
    try!(File::create(&path.join("Cargo.toml")).write_str(format!(
r#"[package]

name = "{}"
version = "0.0.1"
authors = ["{}"]
"#, name, author).as_slice()));

    try!(fs::mkdir(&path.join("src"), io::UserRWX));

    if opts.bin {
        try!(File::create(&path.join("src/main.rs")).write_str("\
fn main() {
    println!(\"Hello, world!\")
}
"));
    } else {
        try!(File::create(&path.join("src/lib.rs")).write_str("\
#[test]
fn it_works() {
}
"));
    }

    Ok(())
}

fn discover_author() -> CargoResult<String> {
    let name = match git!("config", "user.name") {
        Ok(out) => String::from_utf8_lossy(out.output.as_slice()).into_string(),
        Err(..) => match os::getenv("USER") {
            Some(user) => user,
            None => return Err(human("could not determine the current user, \
                                      please set $USER"))
        }
    };

    let email = match git!("config", "user.email") {
        Ok(out) => Some(String::from_utf8_lossy(out.output.as_slice()).into_string()),
        Err(..) => None,
    };

    let name = name.as_slice().trim().to_string();
    let email = email.map(|s| s.as_slice().trim().to_string());

    Ok(match (name, email) {
        (name, Some(email)) => format!("{} <{}>", name, email),
        (name, None) => name,
    })
}
