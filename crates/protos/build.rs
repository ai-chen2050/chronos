use std::io::Result;
use prost_build::Config;

fn main() -> Result<()> {
    Config::new().out_dir("src").compile_protos(
        &[
            "src/zmessage.proto",
            "src/vlc.proto",
            "src/zp2p.proto",
            "src/bussiness.proto",
            ], 
        &["src"])?;
    Ok(())
}