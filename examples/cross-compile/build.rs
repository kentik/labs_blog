use anyhow::Result;
use cc::Build;

fn main() -> Result<()> {
    Build::new().file("src/now.c").compile("now");
    Ok(())
}
