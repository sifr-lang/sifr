use prost_types::field_descriptor_proto::{Label, Type};
use prost_types::{
    DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
};
use std::error::Error;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    if manifest_dir.join("ARM_BUILD_SCRIPT_SENTINEL").is_file() {
        std::fs::write(
            manifest_dir.join("BUILD_SCRIPT_EXECUTED"),
            "prost-build=0.14.4;build-script=executed",
        )?;
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor = FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("sifr_probe.proto".to_string()),
            package: Some("sifr.probe".to_string()),
            message_type: vec![DescriptorProto {
                name: Some("Probe".to_string()),
                field: vec![
                    field("id", 1, Type::Uint64),
                    field("payload", 2, Type::Bytes),
                ],
                ..DescriptorProto::default()
            }],
            syntax: Some("proto3".to_string()),
            ..FileDescriptorProto::default()
        }],
    };
    prost_build_upstream::Config::new().compile_fds(descriptor)?;

    let generated_path = out_dir.join("sifr.probe.rs");
    let generated = std::fs::read_to_string(&generated_path)?;
    if !generated.contains("pub struct Probe")
        || !generated.contains("pub id: u64")
        || !generated.contains("pub payload: ::prost::alloc::vec::Vec<u8>")
    {
        return Err(io::Error::other("prost-build output omitted the probe schema").into());
    }
    std::fs::write(
        out_dir.join("sifr-prost-build-evidence.txt"),
        "prost-build=0.14.4;message=sifr.probe.Probe",
    )?;
    Ok(())
}

fn field(name: &str, number: i32, kind: Type) -> FieldDescriptorProto {
    FieldDescriptorProto {
        name: Some(name.to_string()),
        number: Some(number),
        label: Some(Label::Optional as i32),
        r#type: Some(kind as i32),
        ..FieldDescriptorProto::default()
    }
}
