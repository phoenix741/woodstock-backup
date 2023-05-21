use std::io::Result;

extern crate napi_build;

fn main() -> Result<()> {
  let mut config = prost_build::Config::new();
  //config.message_attribute(".", "#[napi]");
  config.protoc_arg("--experimental_allow_proto3_optional");
  config.compile_protos(&["../nestjs/woodstock.proto"], &["../nestjs/"])?;

  napi_build::setup();

  Ok(())
}
