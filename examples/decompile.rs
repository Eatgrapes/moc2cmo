use std::{env, error::Error, fs, io};

use moc2cmo::{Decompiler, Texture};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let moc3_path = arguments
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing model.moc3 path"))?;
    let output_path = arguments
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "missing output.cmo3 path"))?;

    let moc3 = fs::read(moc3_path)?;
    let mut decompiler = Decompiler::new();
    for texture_path in arguments {
        decompiler.push_texture(Texture::from_png(fs::read(texture_path)?)?);
    }
    decompiler.decompile_to_file(&moc3, output_path)?;
    Ok(())
}
