use std::{env, error::Error, fs, io, path::Path};

use moc2cmo::{Decompiler, Texture, can3::Can3Project};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let moc3_path = required_argument(&mut arguments, "model.moc3 path")?;
    let source_can3_path = required_argument(&mut arguments, "source CAN3 path")?;
    let cmo3_output = required_argument(&mut arguments, "output.cmo3 path")?;
    let can3_output = required_argument(&mut arguments, "output.can3 path")?;

    let moc3 = fs::read(moc3_path)?;
    let mut decompiler = Decompiler::new();
    for texture_path in arguments {
        decompiler.push_texture(Texture::from_png(fs::read(texture_path)?)?);
    }
    decompiler.decompile_to_file(&moc3, &cmo3_output)?;

    let mut animation = Can3Project::decode(&fs::read(source_can3_path)?)?;
    let model_name = Path::new(&cmo3_output).file_name().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "output model path has no file name",
        )
    })?;
    animation.relink_model(model_name)?;
    animation.write_to(can3_output)?;
    Ok(())
}

fn required_argument(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    name: &str,
) -> io::Result<std::ffi::OsString> {
    arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("missing {name} argument"),
        )
    })
}
