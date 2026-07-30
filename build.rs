use std::env;
use std::fs;
use std::path::{PathBuf, Path};
use std::process::Command;

fn main()
{
    let shader_dir = Path::new("shaders");

    println!("cargo:rerun-if-changed=shaders");

    if !shader_dir.exists()
    {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let profile_dir = out_dir
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap();

    let out_path = profile_dir.join("shaders");

    let entries = fs::read_dir(shader_dir).expect("Failed to read shaders directory");

    fs::create_dir_all(&out_path).expect("Failed to create directory: OUT_DIR/shaders");

    for entry in entries
    {
        let entry = entry.unwrap();
        let src_path = entry.path();

        if src_path.is_file()
        {
            let file_name = src_path.file_name().unwrap().to_str().unwrap();

            let dest_file_name = format!("{}.spv", file_name);
            let dst_path = out_path.join(dest_file_name);

            let status = Command::new("glslc")
                .arg(&src_path)
                .arg("-o")
                .arg(&dst_path)
                .status()
                .expect("Failed to run glslc. Check PATH.");

            if !status.success()
            {
                panic!("Fauked to compile shader: {:?}", src_path);
            }
        }
    }
}