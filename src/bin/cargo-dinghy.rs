extern crate cargo;
#[macro_use]
extern crate clap;
extern crate dinghy;
extern crate loggerv;
#[macro_use]
extern crate log;

use std::{env, path, thread, time};

use cargo::util::important_paths::find_root_manifest_for_wd;

use dinghy::errors::*;

fn main() {


    let filtered_env = ::std::env::args()
        .enumerate()
        .filter(|&(ix, ref s)| !(ix == 1 && s == "dinghy"))
        .map(|(_, s)| s);

    let matches = {
        ::clap::App::new("dinghy")
                .version(crate_version!())
                .arg(::clap::Arg::with_name("DEVICE")
                    .short("d")
                    .long("device")
                    .takes_value(true)
                    .help("device hint"))
                .arg(::clap::Arg::with_name("VERBOSE")
                    .short("v")
                    .long("verbose")
                    .multiple(true)
                    .help("Sets the level of verbosity"))
                .arg(::clap::Arg::with_name("TOOLCHAIN")
                    .short("t")
                    .long("toolchain")
                    .takes_value(true)
                    .help("Use a specific toolchain (build only)"))
                .subcommand(::clap::SubCommand::with_name("devices"))
                .subcommand(::clap::SubCommand::with_name("test")
                    .arg(::clap::Arg::with_name("SPEC")
                        .short("p")
                        .long("package")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .help("Package to run tests for"))
                    .arg(::clap::Arg::with_name("DEBUGGER")
                        .long("debugger")
                        .takes_value(false)
                        .help("just start debugger"))
                    .arg(::clap::Arg::with_name("CLEANUP")
                        .long("cleanup")
                        .takes_value(false)
                        .help("cleanup device after complete"))
                    .arg(::clap::Arg::with_name("TARGET")
                        .long("target")
                        .takes_value(true)
                        .help("target triple (rust conventions)"))
                    .arg(::clap::Arg::with_name("ALL")
                         .long("all")
                         .help("Test all packages in the workspace"))
                    .arg(::clap::Arg::with_name("EXCLUDE")
                        .long("exclude")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .help("Exclude package to from the test"))
                    .arg(::clap::Arg::with_name("VERBOSE")
                        .short("v")
                        .long("verbose")
                        .multiple(true)
                        .help("Use verbose output"))
                    .arg(::clap::Arg::with_name("LIB").long("lib").help("only the library"))
                    .arg(::clap::Arg::with_name("BIN")
                        .long("bin")
                        .takes_value(true)
                        .help("only the specified binary"))
                    .arg(::clap::Arg::with_name("EXAMPLE")
                        .long("example")
                        .takes_value(true)
                        .help("only the specified example"))
                    .arg(::clap::Arg::with_name("TEST")
                        .long("test")
                        .takes_value(true)
                        .help("only the specified integration test target"))
                    .arg(::clap::Arg::with_name("BENCH")
                        .long("bench")
                        .takes_value(true)
                        .help("only the specified benchmark target"))
                    .arg(::clap::Arg::with_name("RELEASE")
                        .long("release")
                        .help("Build artifacts in release mode, with optimizations"))
                    .arg(::clap::Arg::with_name("ENVS")
                        .long("env")
                        .takes_value(true)
                        .multiple(true)
                        .help("Space-separated list of env variables to set e.g. RUST_TRACE=trace"))
                    .arg(::clap::Arg::with_name("FEATURES")
                        .long("features")
                        .takes_value(true)
                        .help("Space-separated list of features to also build"))
                    .arg(::clap::Arg::with_name("ALL_FEATURES")
                        .long("all-features")
                        .help("Build all available features"))
                    .arg(::clap::Arg::with_name("NO_DEFAULT_FEATURES")
                        .long("no-default-features")
                        .help("Do not build the `default` feature"))
                    .arg(::clap::Arg::with_name("ARGS").multiple(true).help("test arguments")))
                .subcommand(::clap::SubCommand::with_name("run")
                    .arg(::clap::Arg::with_name("DEBUGGER")
                        .long("debugger")
                        .takes_value(false)
                        .help("just start debugger"))
                    .arg(::clap::Arg::with_name("CLEANUP")
                        .long("cleanup")
                        .takes_value(false)
                        .help("cleanup device after complete"))
                    .arg(::clap::Arg::with_name("TARGET")
                        .long("target")
                        .takes_value(true)
                        .help("target triple (rust conventions)"))
                    .arg(::clap::Arg::with_name("VERBOSE")
                        .short("v")
                        .long("verbose")
                        .multiple(true)
                        .help("Use verbose output"))
                    .arg(::clap::Arg::with_name("BIN")
                        .long("bin")
                        .takes_value(true)
                        .help("only the specified binary"))
                    .arg(::clap::Arg::with_name("EXAMPLE")
                        .long("example")
                        .takes_value(true)
                        .help("only the specified example"))
                    .arg(::clap::Arg::with_name("RELEASE")
                        .long("release")
                        .help("Build artifacts in release mode, with optimizations"))
                    .arg(::clap::Arg::with_name("ENVS")
                        .long("env")
                        .takes_value(true)
                        .multiple(true)
                        .help("Space-separated list of env variables to set e.g. RUST_TRACE=trace"))
                    .arg(::clap::Arg::with_name("FEATURES")
                        .long("features")
                        .takes_value(true)
                        .help("Space-separated list of features to also build"))
                    .arg(::clap::Arg::with_name("ALL_FEATURES")
                        .long("all-features")
                        .help("Build all available features"))
                    .arg(::clap::Arg::with_name("NO_DEFAULT_FEATURES")
                        .long("no")
                        .short("default")
                        .short("features")
                        .help("Do not build the `default` feature"))
                    .arg(::clap::Arg::with_name("ARGS").multiple(true).help("test arguments")))
                .subcommand(::clap::SubCommand::with_name("bench")
                    .arg(::clap::Arg::with_name("SPEC")
                        .short("p")
                        .long("package")
                        .multiple(true)
                        .number_of_values(1)
                        .takes_value(true)
                        .help("Package to run benchmarks for"))
                    .arg(::clap::Arg::with_name("DEBUGGER")
                        .long("debugger")
                        .takes_value(false)
                        .help("just start debugger"))
                    .arg(::clap::Arg::with_name("CLEANUP")
                        .long("cleanup")
                        .takes_value(false)
                        .help("cleanup device after complete"))
                    .arg(::clap::Arg::with_name("TARGET")
                        .long("target")
                        .takes_value(true)
                        .help("target triple (rust conventions)"))
                    .arg(::clap::Arg::with_name("ALL")
                         .long("all")
                         .help("Benchmark all packages in the workspace"))
                    .arg(::clap::Arg::with_name("EXCLUDE")
                        .long("exclude")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .help("Exclude package to from the benchmark"))
                    .arg(::clap::Arg::with_name("VERBOSE")
                        .short("v")
                        .long("verbose")
                        .multiple(true)
                        .help("Use verbose output"))
                    .arg(::clap::Arg::with_name("LIB").long("lib").help("only the library"))
                    .arg(::clap::Arg::with_name("BIN")
                        .long("bin")
                        .takes_value(true)
                        .help("only the specified binary"))
                    .arg(::clap::Arg::with_name("EXAMPLE")
                        .long("example")
                        .takes_value(true)
                        .help("only the specified example"))
                    .arg(::clap::Arg::with_name("TEST")
                        .long("test")
                        .takes_value(true)
                        .help("only the specified integration test target"))
                    .arg(::clap::Arg::with_name("BENCH")
                        .long("bench")
                        .takes_value(true)
                        .help("only the specified benchmark target"))
                    .arg(::clap::Arg::with_name("ENVS")
                        .long("env")
                        .takes_value(true)
                        .multiple(true)
                        .help("Space-separated list of env variables to set e.g. RUST_TRACE=trace"))
                    .arg(::clap::Arg::with_name("FEATURES")
                        .long("features")
                        .takes_value(true)
                        .help("Space-separated list of features to also build"))
                    .arg(::clap::Arg::with_name("ALL_FEATURES")
                        .long("all-features")
                        .help("Build all available features"))
                    .arg(::clap::Arg::with_name("NO_DEFAULT_FEATURES")
                        .long("no")
                        .short("default")
                        .short("features")
                        .help("Do not build the `default` feature"))
                    .arg(::clap::Arg::with_name("ARGS").multiple(true).help("test arguments")))
                .subcommand(::clap::SubCommand::with_name("build")
                    .arg(::clap::Arg::with_name("SPEC")
                        .short("p")
                        .long("package")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .help("Package to build"))
                    .arg(::clap::Arg::with_name("TARGET")
                        .long("target")
                        .takes_value(true)
                        .help("target triple (rust conventions)"))
                    .arg(::clap::Arg::with_name("ALL")
                         .long("all")
                         .help("Build all packages in the workspace"))
                    .arg(::clap::Arg::with_name("EXCLUDE")
                        .long("exclude")
                        .takes_value(true)
                        .multiple(true)
                        .number_of_values(1)
                        .help("Exclude package to from the build"))
                    .arg(::clap::Arg::with_name("VERBOSE")
                        .short("v")
                        .long("verbose")
                        .multiple(true)
                        .help("Use verbose output"))
                    .arg(::clap::Arg::with_name("BIN")
                        .long("bin")
                        .takes_value(true)
                        .help("only the specified binary"))
                    .arg(::clap::Arg::with_name("EXAMPLE")
                        .long("example")
                        .takes_value(true)
                        .help("only the specified example"))
                    .arg(::clap::Arg::with_name("TEST")
                        .long("test")
                        .takes_value(true)
                        .help("only the specified integration test target"))
                    .arg(::clap::Arg::with_name("BENCH")
                        .long("bench")
                        .takes_value(true)
                        .help("only the specified benchmark target"))
                    .arg(::clap::Arg::with_name("RELEASE")
                        .long("release")
                        .help("Build artifacts in release mode, with optimizations"))
                    .arg(::clap::Arg::with_name("FEATURES")
                        .long("features")
                        .takes_value(true)
                        .help("Space-separated list of features to also build"))
                    .arg(::clap::Arg::with_name("ALL_FEATURES")
                        .long("all-features")
                        .help("Build all available features"))
                    .arg(::clap::Arg::with_name("NO_DEFAULT_FEATURES")
                        .long("no")
                        .short("default")
                        .short("features")
                        .help("Do not build the `default` feature"))
                    .arg(::clap::Arg::with_name("ARGS").multiple(true).help("test arguments")))
                .subcommand(::clap::SubCommand::with_name("lldbproxy"))
    }.get_matches_from(filtered_env);

    loggerv::init_with_verbosity(matches.occurrences_of("VERBOSE")).unwrap();

    if let Err(e) = run(matches) {
        println!("{}", e);
        std::process::exit(1);
    }
}

fn maybe_device_from_cli(matches: &clap::ArgMatches) -> Result<Option<Box<dinghy::Device>>> {
    let dinghy = dinghy::Dinghy::probe()?;
    thread::sleep(time::Duration::from_millis(100));
    let devices = dinghy
        .devices()?
        .into_iter()
        .filter(|d| match matches.value_of("DEVICE") {
            Some(filter) => format!("{:?}", d)
                .to_lowercase()
                .contains(&filter.to_lowercase()),
            None => true,
        })
        .collect::<Vec<_>>();
    Ok(devices.into_iter().next())
}
fn device_from_cli(matches: &clap::ArgMatches) -> Result<Box<dinghy::Device>> {
    Ok(maybe_device_from_cli(matches)?.ok_or("No device found")?)
}

fn run(matches: clap::ArgMatches) -> Result<()> {
    match matches.subcommand() {
        ("devices", Some(_matches)) => {
            let dinghy = dinghy::Dinghy::probe()?;
            thread::sleep(time::Duration::from_millis(100));
            let devices = dinghy.devices()?;
            for d in devices {
                println!("{:?}", d);
            }
            Ok(())
        }
        ("run", Some(subs)) => prepare_and_run(&*device_from_cli(&matches)?, "run", subs),
        ("test", Some(subs)) => prepare_and_run(&*device_from_cli(&matches)?, "test", subs),
        ("bench", Some(subs)) => prepare_and_run(&*device_from_cli(&matches)?, "bench", subs),
        ("build", Some(subs)) => {
            let dev = maybe_device_from_cli(&matches)?;
            let target = if let Some(target) = subs.value_of("TARGET") {
                target.into()
            } else if let Some(ref d) = dev {
                d.target()
            } else {
                Err("no toolchain nor device could be determined")?
            };
            let toolchain = if let Some(tc) = matches.value_of("TOOLCHAIN") {
                dinghy::regular_toolchain::RegularToolchain::new(tc)?
            } else if let Some(d) = dev {
                d.toolchain(&target)?
            } else {
                Err("no toolchain nor device could be determined")?
            };
            build(&target, &*toolchain, cargo::ops::CompileMode::Build, subs)?;
            Ok(())
        }
        ("lldbproxy", Some(_matches)) => {
            let lldb = device_from_cli(&matches)?.start_remote_lldb()?;
            println!("lldb running at: {}", lldb);
            loop {
                thread::sleep(time::Duration::from_millis(100));
            }
        }
        (sub, _) => Err(format!("Unknown subcommand {}", sub))?,
    }
}

#[derive(Debug)]
struct Runnable {
    name: String,
    exe: path::PathBuf,
    source: path::PathBuf,
}

fn prepare_and_run(d: &dinghy::Device, subcommand: &str, matches: &clap::ArgMatches) -> Result<()> {
    let target = matches
        .value_of("TARGET")
        .map(|s| s.into())
        .unwrap_or(d.target());
    if !d.can_run(&*target) {
        Err(format!("device {:?} can not run target {}", d, target))?;
    }
    info!("Picked device `{}' [{}]", d.name(), target);
    let mode = match subcommand {
        "test" => cargo::ops::CompileMode::Test,
        "bench" => cargo::ops::CompileMode::Bench,
        _ => cargo::ops::CompileMode::Build,
    };
    let tc = d.toolchain(&target)?;
    debug!("Toolchain {:?}", tc);
    let runnable = build(&*target, &*tc, mode, matches)?;
    let args = matches
        .values_of("ARGS")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let envs = matches
        .values_of("ENVS")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    for t in runnable {
        let app = d.make_app(&t.source, &t.exe)?;
        if subcommand != "build" {
            d.install_app(&app.as_ref())?;
            if matches.is_present("DEBUGGER") {
                println!("DEBUGGER");
                d.debug_app(
                    app.as_ref(),
                    &*args.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                    &*envs.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                )?;
            } else {
                d.run_app(
                    app.as_ref(),
                    &*args.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                    &*envs.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                )?;
            }
            if matches.is_present("CLEANUP") {
                d.clean_app(&app.as_ref())?;
            }
        }
    }
    Ok(())
}


fn build(
    target: &str,
    toolchain: &dinghy::Toolchain,
    mode: cargo::ops::CompileMode,
    matches: &clap::ArgMatches,
) -> Result<Vec<Runnable>> {
    info!("Building for target {} using {}", target, toolchain);
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let cfg = cargo::util::config::Config::default()?;
    let features: Vec<String> = matches
        .value_of("FEATURES")
        .unwrap_or("")
        .split(" ")
        .map(|s| s.into())
        .collect();
    toolchain.setup_env(target)?;
    cfg.configure(
        matches.occurrences_of("VERBOSE") as u32,
        None,
        &None,
        false,
        false,
        &[],
    )?;
    let wd = cargo::core::Workspace::new(&wd_path, &cfg)?;
    let bins = matches
        .values_of("BIN")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let tests = matches
        .values_of("TEST")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let examples = matches
        .values_of("EXAMPLE")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let benches = matches
        .values_of("BENCH")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let filter = cargo::ops::CompileFilter::new(
        matches.is_present("LIB"),
        &bins,
        false,
        &tests,
        false,
        &examples,
        false,
        &benches,
        false,
        false,
    );
    let excludes = matches
        .values_of("EXCLUDE")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let packages = matches
        .values_of("SPEC")
        .map(|vs| vs.map(|s| s.to_string()).collect())
        .unwrap_or(vec![]);
    let spec = cargo::ops::Packages::from_flags(
        wd.is_virtual(),
        matches.is_present("ALL"),
        &excludes,
        &packages,
    )?;

    let options = cargo::ops::CompileOptions {
        config: &cfg,
        jobs: None,
        target: Some(&*target),
        features: &*features,
        all_features: matches.is_present("ALL_FEATURES"),
        no_default_features: matches.is_present("NO_DEFAULT_FEATURES"),
        spec: spec,
        filter: filter,
        release: mode == cargo::ops::CompileMode::Bench || matches.is_present("RELEASE"),
        mode: mode,
        message_format: cargo::ops::MessageFormat::Human,
        target_rustdoc_args: None,
        target_rustc_args: None,
    };
    let compilation = cargo::ops::compile(&wd, &options)?;
    if mode == cargo::ops::CompileMode::Build {
        Ok(
            compilation
                .binaries
                .into_iter()
                .take(1)
                .map(|t| {
                    Runnable {
                        name: "main".into(),
                        exe: t,
                        source: path::PathBuf::from("."),
                    }
                })
                .collect::<Vec<_>>(),
        )
    } else {
        Ok(
            compilation
                .tests
                .into_iter()
                .map(|(pkg, _, name, exe)| {
                    Runnable {
                        name: name,
                        source: pkg.root().to_path_buf(),
                        exe: exe,
                    }
                })
                .collect::<Vec<_>>(),
        )
    }
}
