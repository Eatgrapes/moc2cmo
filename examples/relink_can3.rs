use std::{env, error::Error, ffi::OsString, fs, io, path::Path};

use moc2cmo::can3::Can3Project;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let source = required_argument(&mut arguments, "source CAN3")?;
    let model = required_argument(&mut arguments, "linked CMO3")?;
    let destination = required_argument(&mut arguments, "destination CAN3")?;

    let mut project = Can3Project::decode(&fs::read(source)?)?;
    project.relink_model(Path::new(&model))?;
    project.write_to(destination)?;
    Ok(())
}

fn required_argument(
    arguments: &mut impl Iterator<Item = OsString>,
    name: &str,
) -> io::Result<OsString> {
    arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("missing {name} argument"),
        )
    })
}
