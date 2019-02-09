use compiler::Compiler;
use config::PlatformConfiguration;
use dinghy_build::build_env::set_env;
use errors::*;
use overlay::Overlayer;
use project::Project;
use std::fmt::{ Debug, Display, Formatter };
use std::process;
use std::sync::Arc;
use toolchain::Toolchain;
use Build;
use BuildArgs;
use Device;
use Platform;

pub struct IosPlatform {
    id: String,
    pub sim: bool,
    pub toolchain: Toolchain,
    pub configuration: PlatformConfiguration,
    compiler: Arc<Compiler>,
}

impl Debug for IosPlatform {
    fn fmt(&self, fmt: &mut Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.id)
    }
}

impl IosPlatform {
    pub fn new(id: String, rustc_triple: &str, compiler: &Arc<Compiler>, configuration: PlatformConfiguration) -> Result<Box<Platform>> {
        Ok(Box::new(IosPlatform {
            id,
            sim: rustc_triple.contains("86"),
            toolchain: Toolchain {
                rustc_triple: rustc_triple.to_string()
            },
            compiler: Arc::clone(compiler),
            configuration,
        }))
    }

    fn sysroot_path(&self) -> Result<String> {
        let sdk_name = if self.sim {
            "iphonesimulator"
        } else {
            "iphoneos"
        };
        let xcrun = process::Command::new("xcrun")
            .args(&["--sdk", sdk_name, "--show-sdk-path"])
            .output()?;
        Ok(String::from_utf8(xcrun.stdout)?.trim_end().to_string())
    }
}

impl Platform for IosPlatform {
    fn build(&self, project: &Project, build_args: &BuildArgs) -> Result<Build> {
        let sysroot = self.sysroot_path()?;
        Overlayer::overlay(&self.configuration, self, project, &self.sysroot_path()?)?;
        self.toolchain.setup_cc(self.id().as_str(), "gcc")?;
        set_env("TARGET_SYSROOT", &sysroot);
        self.toolchain.setup_linker(&self.id(),
                                    &format!("cc -isysroot {}", sysroot))?;
        self.toolchain.setup_pkg_config()?;

        self.compiler.build(self.rustc_triple(), build_args)
    }

    fn id(&self) -> String {
        self.id.to_string()
    }

    fn is_compatible_with(&self, device: &Device) -> bool {
        device.is_compatible_with_ios_platform(self)
    }

    fn rustc_triple(&self) -> Option<&str> {
        Some(&self.toolchain.rustc_triple)
    }

    fn strip(&self, build: &Build) -> Result<()> {
        for runnable in &build.runnables {
            let mut command = ::std::process::Command::new("xcrun");
            command.arg("strip");
            ::platform::strip_runnable(runnable, command)?;
        }
        Ok(())
    }
}

impl Display for IosPlatform {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        if self.sim {
            write!(f, "XCode targetting Ios Simulator")
        } else {
            write!(f, "XCode targetting Ios Device")
        }
    }
}

