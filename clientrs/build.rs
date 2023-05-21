use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .message_attribute(".", "#[serde_with::serde_as]\n#[derive(serde::Serialize)]")
        .enum_attribute(".", "#[derive(serde::Serialize)]")
        .field_attribute(
            "PoolUnused.sha256",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        .field_attribute(
            "PoolRefCount.sha256",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        .field_attribute(
            "FileManifest.sha256",
            "#[serde_as(as = \"serde_with::hex::Hex\")]",
        )
        .field_attribute(
            "FileManifest.chunks",
            "#[serde_as(as = \"Vec<serde_with::hex::Hex>\")]",
        )
        .field_attribute(
            "FileManifest.path",
            "#[serde_as(as = \"serde_with::base64::Base64\")]",
        )
        .field_attribute(
            "FileManifest.symlink",
            "#[serde_as(as = \"serde_with::base64::Base64\")]",
        )
        .compile(&["./woodstock.proto"], &["./"])?;

    Ok(())
}
