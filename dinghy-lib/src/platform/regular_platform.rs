use dinghy_build::build_env::set_all_env;
use overlay::Overlayer;
use platform;
use project::Project;
use std::fmt::{ Debug, Display, Formatter };
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use toolchain::ToolchainConfig;
use Build;
use BuildArgs;
use compiler::Compiler;
use config::PlatformConfiguration;
use Device;
use Platform;
use Result;

pub struct RegularPlatform {
    compiler: Arc<Compiler>,
    pub configuration: PlatformConfiguration,
    pub id: String,
    pub toolchain: ToolchainConfig,
}

impl Debug for RegularPlatform {
    fn fmt(&self, fmt: &mut Formatter) -> ::std::fmt::Result {
        write!(fmt, "{}", self.id)
    }
}

impl RegularPlatform {
    pub fn new<P: AsRef<Path>>(compiler: &Arc<Compiler>,
                               configuration: PlatformConfiguration,
                               id: String,
                               rustc_triple: String,
                               toolchain_path: P) -> Result<Box<Platform>> {
        if let Some(prefix) = configuration.deb_multiarch.clone() {
            return Ok(Box::new(RegularPlatform {
                compiler: compiler.clone(),
                configuration,
                id,
                toolchain: ToolchainConfig {
                    bin_dir: "/usr/bin".into(),
                    rustc_triple,
                    root: "/".into(),
                    sysroot: "/".into(),
                    cc: "gcc".to_string(),
                    binutils_prefix: prefix.clone(),
                    cc_prefix: prefix.clone(),
                },
            }))
        }
        let toolchain_path = toolchain_path.as_ref();
        let toolchain_bin_path = toolchain_path.join("bin");

        let mut bin: Option<PathBuf> = None;
        let mut prefix: Option<String> = None;
        for file in toolchain_bin_path.read_dir().map_err(|_| format!("Couldn't find toolchain directory {}", toolchain_path.display()))? {
            let file = file?;
            if file.file_name().to_string_lossy().ends_with("-gcc")
                || file.file_name().to_string_lossy().ends_with("-gcc.exe") {
                bin = Some(toolchain_bin_path);
                prefix = Some(
                    file.file_name()
                        .to_string_lossy()
                        .replace(".exe", "")
                        .replace("-gcc", ""),
                );
                break;
            }
        }
        let bin_dir = bin.ok_or("no bin/*-gcc found in toolchain")?;
        let tc_triple = prefix.ok_or("no gcc in toolchain")?.to_string();
        let sysroot = find_sysroot(&toolchain_path)?;

        let toolchain = ToolchainConfig {
                bin_dir,
                rustc_triple,
                root: toolchain_path.into(),
                sysroot,
                cc: "gcc".to_string(),
                binutils_prefix: tc_triple.clone(),
                cc_prefix: tc_triple,
            };
        Self::new_with_tc(compiler.clone(), configuration, id, toolchain)
    }

    pub fn new_with_tc(compiler: Arc<Compiler>, configuration: PlatformConfiguration, id: String, toolchain: ToolchainConfig) -> Result<Box<Platform>> {
        Ok(Box::new(RegularPlatform {
            compiler,
            configuration,
            id,
            toolchain,
        }))
    }
}

impl Display for RegularPlatform {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        write!(f, "{:?}", self.toolchain.root)
    }
}

impl Platform for RegularPlatform {
    fn build(&self, project: &Project, build_args: &BuildArgs) -> Result<Build> {
        // Cleanup environment
        set_all_env(&[
            ("LIBRARY_PATH", ""),
            ("LD_LIBRARY_PATH", ""),
        ]);
        // Set custom env variables specific to the platform
        set_all_env(&self.configuration.env());

        Overlayer::overlay(&self.configuration, self, project, &self.toolchain.sysroot)?;

        self.toolchain.setup_cc(&self.id, &self.toolchain.cc_executable(&self.toolchain.cc))?;

        if Path::new(&self.toolchain.binutils_executable("ar")).exists() {
            self.toolchain.setup_tool("AR", &self.toolchain.binutils_executable("ar"))?;
        }
        if Path::new(&self.toolchain.binutils_executable("as")).exists() {
            self.toolchain.setup_tool("AS", &self.toolchain.binutils_executable("as"))?;
        }
        if Path::new(&self.toolchain.binutils_executable("c++")).exists() {
            self.toolchain.setup_tool("CXX", &self.toolchain.cc_executable("c++"))?;
        }
        if Path::new(&self.toolchain.cc_executable("cpp")).exists() {
            self.toolchain.setup_tool("CPP", &self.toolchain.cc_executable("cpp"))?;
        }
        if Path::new(&self.toolchain.binutils_executable("gfortran")).exists() {
            self.toolchain.setup_tool("FC", &self.toolchain.binutils_executable("gfortran"))?;
        }
        trace!("Setup linker...");

        let mut linker_cmd = self.toolchain.cc_executable(&*self.toolchain.cc);
        linker_cmd.push_str(" ");
        if build_args.verbose { linker_cmd.push_str("-Wl,--verbose -v") }
        linker_cmd.push_str(&format!(" --sysroot {}", self.toolchain.sysroot.display()));
        for forced_overlay in &build_args.forced_overlays {
            linker_cmd.push_str(" -l");
            linker_cmd.push_str(&forced_overlay);
            // TODO Add -L
        }
        self.toolchain.setup_linker(&self.id, &linker_cmd)?;

        trace!("Setup pkg-config");
        self.toolchain.setup_pkg_config()?;
        trace!("Setup sysroot...");
        self.toolchain.setup_sysroot();
        trace!("Setup shims...");
        self.toolchain.shim_executables(&self.id)?;

        trace!("Internally invoke cargo");
        self.compiler.build(self.rustc_triple(), &build_args)
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn is_compatible_with(&self, device: &Device) -> bool {
        device.is_compatible_with_regular_platform(self)
    }

    fn rustc_triple(&self) -> Option<&str> {
        Some(&self.toolchain.rustc_triple)
    }

    fn strip(&self, build: &Build) -> Result<()> {
        for runnable in &build.runnables {
            platform::strip_runnable(runnable, Command::new(self.toolchain.binutils_executable("strip")))?;
        }
        Ok(())
    }
}

fn find_sysroot<P: AsRef<Path>>(toolchain_path: P) -> Result<PathBuf> {
    let toolchain = toolchain_path.as_ref();
    let immediate = toolchain.join("sysroot");
    if immediate.is_dir() {
        let sysroot = immediate.to_str().ok_or("sysroot is not utf-8")?;
        return Ok(sysroot.into());
    }
    for subdir in toolchain.read_dir()? {
        let subdir = subdir?;
        let maybe = subdir.path().join("sysroot");
        if maybe.is_dir() {
            let sysroot = maybe.to_str().ok_or("sysroot is not utf-8")?;
            return Ok(sysroot.into());
        }
    }
    Err(format!("no sysroot found in toolchain {:?}", toolchain))?
}
