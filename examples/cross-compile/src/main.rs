use std::ffi::CStr;
use std::os::raw::c_char;
use anyhow::{anyhow, Result};
use libc::size_t;

extern "C" {
    fn now(ptr: *mut c_char, len: size_t) -> size_t;
}

fn main() -> Result<()> {
    let mut buffer = [0u8; 1024];

    let timestamp = unsafe {
        let ptr = buffer.as_mut_ptr() as *mut c_char;
        let len = buffer.len()        as size_t;

        match now(ptr, len) {
            n if n > 0 => Ok(CStr::from_bytes_with_nul(&buffer[..n+1])?),
            _          => Err(anyhow!("error: strftime returned zero")),
        }
    }?.to_string_lossy();

    println!("current time: {timestamp}");

    Ok(())

}
