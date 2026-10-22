use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let out_dir = env::var("OUT_DIR").unwrap();
        let icon_src = Path::new("parvan.ico").canonicalize().unwrap();
        let rc_content = format!("id ICON \"{}\"", icon_src.display().to_string().replace('\\', "/"));
        let rc_file = Path::new(&out_dir).join("resource.rc");
        let res_file = Path::new(&out_dir).join("resource.o");

        fs::write(&rc_file, rc_content).unwrap();

        // Compile with windres directly using cat/type as dummy preprocessor to bypass GCC path splitting with spaces
        let status = Command::new("C:\\MinGW\\bin\\windres.exe")
            .arg("--preprocessor=cmd.exe")
            .arg("--preprocessor-arg=/c")
            .arg("--preprocessor-arg=type")
            .arg("-i")
            .arg(&rc_file)
            .arg("-o")
            .arg(&res_file)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("cargo:rustc-link-arg={}", res_file.display().to_string().replace('\\', "/"));
            }
            err => {
                panic!("windres failed: {:?}", err);
            }
        }
    }
}
