use prost_build;
use std::error::Error;

// fn main() -> std::io::Result<()> {
// 	// compile protocol buffer using protoc
// 	// protobuf_codegen_pure::Codegen::new()
// 	// 	.out_dir("src/")
// 	// 	.inputs(&["protos/start_match.proto"])
// 	// 	.include("protos")
// 	// 	.run()
// 	// 	.expect("Codegen failed.");
// 	// println!("Hi from build");
// 	let res = prost_build::compile_protos(&["protos/start_match.proto"], &["protos/"]);
// 	match res {
// 		Err(e) => {
// 			println!("{:?}", e);
// 		}
// 		_ => {
// 			println!("Ok");
// 		}
// 	}
// 	println!("Compiled protos");
// 	Ok(())
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
	tonic_build::compile_protos("protos/start_match.proto")?;
	Ok(())
}