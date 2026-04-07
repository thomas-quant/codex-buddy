use std::{io::Write, os::unix::net::UnixStream, path::Path};

use anyhow::Result;

pub fn relay_hook_payload(socket_path: impl AsRef<Path>, payload: impl AsRef<[u8]>) -> Result<()> {
    let mut stream = UnixStream::connect(socket_path.as_ref())?;
    stream.write_all(payload.as_ref())?;
    stream.flush()?;
    Ok(())
}
