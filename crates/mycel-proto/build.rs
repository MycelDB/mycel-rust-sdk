use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    env::set_var("PROTOC", protoc);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir.join("../..");
    let api_root = env::var_os("MYCEL_API_ROOT")
        .map(PathBuf::from)
        .or_else(|| existing_dir(repo_root.join("third_party/mycel-api")))
        .or_else(|| existing_dir(repo_root.join("../mycel-api")));
    let proto_root = api_root
        .ok_or_else(|| {
            format!(
                "mycel-api checkout not found (set MYCEL_API_ROOT, initialize third_party/mycel-api submodule, or place mycel-api beside this repo)"
            )
        })?
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

fn existing_dir(path: PathBuf) -> Option<PathBuf> {
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
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
