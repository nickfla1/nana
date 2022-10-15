use crate::result::NanaResult;

pub fn exec() -> NanaResult<()> {
    println!(
        "{} - v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}
