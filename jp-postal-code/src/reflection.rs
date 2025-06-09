use tonic_reflection::server::v1alpha::{ServerReflection, ServerReflectionServer};

pub fn reflection_service() -> anyhow::Result<ServerReflectionServer<impl ServerReflection>> {
    Ok(tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(jp_postal_code_proto::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?)
}
