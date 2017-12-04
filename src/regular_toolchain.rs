use std::{env, path};

use {Result, Toolchain};

#[derive(Debug)]
pub struct RegularToolchain {
    root: path::PathBuf,
    bin: path::PathBuf,
    bin_prefix: String,
    sysroot: String,
}

impl RegularToolchain {
    pub fn new<P: AsRef<path::Path>>(toolchain: P) -> Result<Box<Toolchain>> {
        let mut bin: Option<path::PathBuf> = None;
        let mut prefix: Option<String> = None;
        for file in toolchain.as_ref().join("bin").read_dir()? {
            let file = file?;
            if file.file_name().to_string_lossy().ends_with("-gcc")
                || file.file_name().to_string_lossy().ends_with("-gcc.exe")
            {
                bin = Some(toolchain.as_ref().join("bin"));
                prefix = Some(
                    file.file_name()
                        .to_string_lossy()
                        .replace(".exe", "")
                        .replace("-gcc", ""),
                );
                break;
            }
        }
        let bin = bin.ok_or("no bin/*-gcc found in toolchain")?;
        let bin_prefix = prefix.ok_or("no gcc in toolchain")?.to_string();
        let sysroot = sysroot_in_toolchain(&toolchain)?;
        Ok(Box::new(RegularToolchain {
            root: toolchain.as_ref().into(),
            bin,
            bin_prefix,
            sysroot,
        }))
    }

    fn binary(&self, name: &str) -> String {
        self.bin
            .join(format!("{}-{}", self.bin_prefix, name))
            .to_string_lossy()
            .into()
    }
}

impl ::std::fmt::Display for RegularToolchain {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(f, "{:?}", self.root)
    }
}

impl Toolchain for RegularToolchain {
    fn cc_command(&self, _target: &str) -> Result<String> {
        Ok(format!("{} {}", self.binary("gcc"), ::shim::GLOB_ARGS))
    }
    fn linker_command(&self, _target: &str) -> Result<String> {
        Ok(format!(
            "{} --sysroot {} {}",
            self.binary("gcc"),
            self.sysroot,
            ::shim::GLOB_ARGS
        ))
    }
    fn setup_more_env(&self, _target: &str) -> Result<()> {
        env::set_var("TARGET_SYSROOT", &self.sysroot);
        env::set_var("TARGET_AR", &self.binary("ar"));
        Ok(())
    }
}

fn sysroot_in_toolchain<P: AsRef<path::Path>>(p: P) -> Result<String> {
    let immediate = p.as_ref().join("sysroot");
    if immediate.is_dir() {
        let sysroot = immediate.to_str().ok_or("sysroot is not utf-8")?;
        return Ok(sysroot.into());
    }
    for subdir in p.as_ref().read_dir()? {
        let subdir = subdir?;
        let maybe = subdir.path().join("sysroot");
        if maybe.is_dir() {
            let sysroot = maybe.to_str().ok_or("sysroot is not utf-8")?;
            return Ok(sysroot.into());
        }
    }
    Err(format!("no sysroot found in toolchain {:?}", p.as_ref()))?
}
