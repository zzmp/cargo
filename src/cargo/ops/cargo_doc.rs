use std::collections::HashSet;

use core::source::Source;
use ops;
use sources::PathSource;
use util::{CargoResult, human};

pub struct DocOptions<'a> {
    pub all: bool,
    pub compile_opts: ops::CompileOptions<'a>,
}

pub fn doc(manifest_path: &Path,
           options: &mut DocOptions) -> CargoResult<()> {
    let mut source = try!(PathSource::for_path(&manifest_path.dir_path()));
    try!(source.update());
    let package = try!(source.get_root_package());

    let mut lib_names = HashSet::new();
    let mut bin_names = HashSet::new();
    for target in package.get_targets().iter().filter(|t| t.get_profile().is_doc()) {
        if target.is_lib() {
            assert!(lib_names.insert(target.get_name()));
        } else {
            assert!(bin_names.insert(target.get_name()));
        }
    }
    for bin in bin_names.iter() {
        if lib_names.contains(bin) {
            return Err(human("Cannot document a package where a library and a \
                              binary have the same name. Consider renaming one \
                              or marking the target as `doc = false`"))
        }
    }

    try!(ops::compile(manifest_path, &mut options.compile_opts));
    Ok(())
}
