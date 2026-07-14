use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    env::set_var("PROTOC", protoc);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_root = env::var_os("MYCEL_API_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("../../../mycel-api"))
        .join("api/proto");

    if !proto_root.is_dir() {
        return Err(format!(
            "mycel-api proto root not found: {} (set MYCEL_API_ROOT to a mycel-api checkout)",
            proto_root.display()
        )
        .into());
    }

    println!("cargo:rerun-if-env-changed=MYCEL_API_ROOT");
    println!("cargo:rerun-if-changed={}", proto_root.display());

    let mut proto_paths = Vec::new();
    collect_proto_files(&proto_root, &mut proto_paths)?;
    proto_paths.sort();

    if proto_paths.is_empty() {
        return Err(format!("no .proto files found under {}", proto_root.display()).into());
    }

    for proto in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&proto_paths, &[proto_root])?;

    Ok(())
}

fn collect_proto_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_proto_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "proto") {
            out.push(path);
        }
    }
    Ok(())
}
