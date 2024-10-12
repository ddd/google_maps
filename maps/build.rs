use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = [
        "proto/mapsjs.proto",
    ];

    let proto_paths = [
        PathBuf::from("../proto"),
    ];

    tonic_build::configure()
        .build_server(false)
        .compile(&proto_files, &proto_paths)?;

    Ok(())
}