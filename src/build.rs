use std::{env, fs, path, process};
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use cargo;

use errors::*;

use cargo::util::important_paths::find_root_manifest_for_wd;

pub fn setup_linker(device_target: &str) -> Result<()> {
    let cfg = cargo::util::config::Config::default()?;
    if let Some(linker) = cfg.get_string(&*format!("target.{}.linker", device_target))? {
        debug!("Config specifies linker {:?} in {}",
               linker.val,
               linker.definition);
        return Ok(());
    }
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let root = wd_path.parent().ok_or("building at / ?")?;
    let target_path = root.join("target").join(device_target);
    if let Some(linker) = guess_linker(device_target)? {
        let shim = create_shim(&root, device_target, &*linker)?;
        let var_name = format!("CARGO_TARGET_{}_LINKER",
                               device_target.replace("-", "_").to_uppercase());
        env::set_var(var_name, shim);
        return Ok(());
    }
    warn!("No linker set or guessed for target {}. See http://doc.crates.io/config.html .",
          device_target);
    Ok(())
}

#[cfg(not(target_os="windows"))]
fn create_shim<P: AsRef<path::Path>>(root: P,
                                     device_target: &str,
                                     shell: &str)
                                     -> Result<path::PathBuf> {
    let target_path = root.as_ref().join("target").join(device_target);
    fs::create_dir_all(&target_path)?;
    let shim = target_path.join("linker");
    if shim.exists() {
        return Ok(shim);
    }
    let mut linker_shim = fs::File::create(&shim)?;
    writeln!(linker_shim, "#!/bin/sh")?;
    linker_shim.write_all(shell.as_bytes())?;
    writeln!(linker_shim, "\n")?;
    fs::set_permissions(&shim, PermissionsExt::from_mode(0o777))?;
    Ok(shim)
}

#[cfg(target_os="windows")]
fn create_shim<P: AsRef<path::Path>>(root: P,
                                     device_target: &str,
                                     shell: &str)
                                     -> Result<path::PathBuf> {
    let target_path = root.as_ref().join("target").join(device_target);
    fs::create_dir_all(&target_path)?;
    let shim = target_path.join("linker.bat");
    let mut linker_shim = fs::File::create(&shim)?;
    linker_shim.write_all(shell.as_bytes())?;
    writeln!(linker_shim, "\n")?;
    Ok(shim)
}

#[cfg(not(target_os="windows"))]
fn guess_linker(device_target: &str) -> Result<Option<String>> {
    if device_target.ends_with("-apple-ios") {
        let xcrun = if device_target.starts_with("x86") {
            process::Command::new("xcrun").args(&["--sdk", "iphonesimulator", "--show-sdk-path"])
                .output()?
        } else {
            process::Command::new("xcrun").args(&["--sdk", "iphoneos", "--show-sdk-path"]).output()?
        };
        let sdk_path = String::from_utf8(xcrun.stdout)?;
        Ok(Some(format!(r#"cc -isysroot {} "$@""#, &*sdk_path.trim_right())))
    } else if device_target == "arm-linux-androideabi" {
        if let Err(_) = env::var("ANDROID_NDK_HOME") {
            if let Ok(home) = env::var("HOME") {
                let mac_place = format!("{}/Library/Android/sdk/ndk-bundle", home);
                if fs::metadata(&mac_place)?.is_dir() {
                    env::set_var("ANDROID_NDK_HOME", &mac_place)
                }
            } else {
                warn!("Android target detected, but could not find (or guess) ANDROID_NDK_HOME. \
                       You may need to set it up.");
                return Ok(None);
            }
        }
        let prebuild_android_toolchains_dir = path::PathBuf::from(env::var("ANDROID_NDK_HOME").unwrap())
            .join("toolchains/arm-linux-androideabi-4.9/prebuilt");
        let prebuilt = fs::read_dir(prebuild_android_toolchains_dir)?
            .next()
            .ok_or("No prebuilt toolchain in your android setup")??;
        Ok(Some(format!(r#"$ANDROID_NDK_HOME/toolchains/arm-linux-androideabi-4.9/prebuilt/{:?}/bin/arm-linux-androideabi-gcc \
                --sysroot $ANDROID_NDK_HOME/platforms/android-18/arch-arm \
                "$@" "#, prebuilt.file_name())))
    } else {
        Ok(None)
    }
}

#[cfg(target_os="windows")]
fn guess_linker(device_target: &str) -> Result<Option<String>> {
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let root = wd_path.parent().ok_or("building at / ?")?;
    let target_path = root.join("target").join(device_target);
    match device_target {
        "arm-linux-androideabi" => {
            let home = env::var("ANDROID_NDK_HOME")
                .map_err(|_| "environment variable ANDROID_NDK_HOME is required")?;

            Ok(Some(format!(r#"{home}\toolchains\arm-linux-androideabi-4.9\prebuilt\windows\bin\arm-linux-androideabi-gcc --sysroot {home}\platforms\android-18\arch-arm %* "#,
                home = home)))
        },
        _ => Ok(None)
    }
}

pub fn compile_tests(device_target: &str) -> Result<Vec<(String, path::PathBuf)>> {
    setup_linker(device_target)?;
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let cfg = cargo::util::config::Config::default()?;
    cfg.configure(0, None, &None, false, false)?;
    let wd = cargo::core::Workspace::new(&wd_path, &cfg)?;
    let options = cargo::ops::CompileOptions {
        config: &cfg,
        jobs: None,
        target: Some(&device_target),
        features: &[],
        all_features: false,
        no_default_features: false,
        spec: &[],
        filter: cargo::ops::CompileFilter::new(false, &[], &[], &[], &[]),
        release: false,
        mode: cargo::ops::CompileMode::Test,
        message_format: cargo::ops::MessageFormat::Human,
        target_rustdoc_args: None,
        target_rustc_args: None,
    };
    let compilation = cargo::ops::compile(&wd, &options)?;
    Ok(compilation.tests.iter().map(|t| (t.1.clone(), t.2.clone())).collect::<Vec<_>>())
}

pub fn compile_benches(device_target: &str) -> Result<Vec<(String, path::PathBuf)>> {
    setup_linker(device_target)?;
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let cfg = cargo::util::config::Config::default()?;
    cfg.configure(0, None, &None, false, false)?;
    let wd = cargo::core::Workspace::new(&wd_path, &cfg)?;
    let options = cargo::ops::CompileOptions {
        config: &cfg,
        jobs: None,
        target: Some(&device_target),
        features: &[],
        all_features: false,
        no_default_features: false,
        spec: &[],
        filter: cargo::ops::CompileFilter::new(false, &[], &[], &[], &[]),
        release: true,
        mode: cargo::ops::CompileMode::Bench,
        message_format: cargo::ops::MessageFormat::Human,
        target_rustdoc_args: None,
        target_rustc_args: None,
    };
    let compilation = cargo::ops::compile(&wd, &options)?;
    Ok(compilation.tests.iter().map(|t| (t.1.clone(), t.2.clone())).collect::<Vec<_>>())
}

pub fn compile_bin(device_target: &str) -> Result<Vec<path::PathBuf>> {
    setup_linker(device_target)?;
    let wd_path = find_root_manifest_for_wd(None, &env::current_dir()?)?;
    let cfg = cargo::util::config::Config::default()?;
    cfg.configure(0, None, &None, false, false)?;
    let wd = cargo::core::Workspace::new(&wd_path, &cfg)?;
    let options = cargo::ops::CompileOptions {
        config: &cfg,
        jobs: None,
        target: Some(&device_target),
        features: &[],
        all_features: false,
        no_default_features: false,
        spec: &[],
        filter: cargo::ops::CompileFilter::new(false, &[], &[], &[], &[]),
        release: false,
        mode: cargo::ops::CompileMode::Build,
        message_format: cargo::ops::MessageFormat::Human,
        target_rustdoc_args: None,
        target_rustc_args: None,
    };
    let compilation = cargo::ops::compile(&wd, &options)?;
    Ok(compilation.binaries)
}
