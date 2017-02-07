

#[cfg(test)]
mod tests {

    mod pass {
        use std::path;

        pub fn src_path() -> path::PathBuf {
            if cfg!(any(target_os = "ios", target_os="android")) || ::std::env::var("DINGHY").is_ok() {
                ::std::env::current_exe().unwrap().parent().unwrap().join("src")
            } else {
                path::PathBuf::from(".")
            }
        }

        #[test]
        fn it_finds_source_files() {
            println!("pwd: {:?}", ::std::env::current_dir());
            println!("src_path: {:?}", src_path());
            assert!(src_path().join("src/lib.rs").exists());
        }

        #[test]
        fn it_works() {
        }
    }

    mod fails {
        #[test]
        fn it_fails() {
            panic!("Failing as expected");
        }
    }
}
