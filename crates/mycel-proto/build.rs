use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    env::set_var("PROTOC", protoc);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_root = manifest_dir.join("../../../mycel-api/api/proto");

    let protos = [
        "mycel/common/v1/access.proto",
        "mycel/client/v1/auth.proto",
        "mycel/client/v1/blob.proto",
        "mycel/client/v1/change_stream.proto",
        "mycel/client/v1/domain.proto",
        "mycel/client/v1/graph.proto",
        "mycel/client/v1/import_export.proto",
        "mycel/client/v1/metadata_catalog.proto",
        "mycel/client/v1/query.proto",
        "mycel/client/v1/semantic.proto",
        "mycel/client/v1/session.proto",
        "mycel/client/v1/space.proto",
        "mycel/client/v1/template.proto",
        "mycel/admin/v1/auth.proto",
        "mycel/admin/v1/backup.proto",
        "mycel/admin/v1/domain.proto",
        "mycel/admin/v1/inference.proto",
        "mycel/admin/v1/operator.proto",
        "mycel/admin/v1/semantic.proto",
        "mycel/admin/v1/semantic_maintenance.proto",
        "mycel/admin/v1/semantic_migration.proto",
        "mycel/admin/v1/space.proto",
        "mycel/admin/v1/user.proto",
    ];

    let proto_paths: Vec<PathBuf> = protos.iter().map(|p| proto_root.join(p)).collect();

    for proto in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto.display());
    }

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&proto_paths, &[proto_root])?;

    Ok(())
}
