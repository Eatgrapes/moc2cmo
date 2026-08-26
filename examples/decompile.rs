use std::{env, error::Error, io};

use moc2cmo::decompile_model3_to_files;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let model3_path = required_argument(&mut arguments, "model3.json path")?;
    let output_directory = required_argument(&mut arguments, "output directory")?;
    if arguments.next().is_some() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unexpected argument").into());
    }
    decompile_model3_to_files(model3_path, output_directory)?;
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
