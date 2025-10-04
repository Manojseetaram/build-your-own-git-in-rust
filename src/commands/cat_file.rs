use crate::objects::Object;
use anyhow::Context;
use std::io::Read;

pub fn invoke(_pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    // Read the object from .git/objects
    let mut obj =
        Object::read(object_hash).with_context(|| format!("read object {}", object_hash))?;

    // Read all content
    let mut buf = Vec::new();
    obj.reader.read_to_end(&mut buf)?;

    // Print as string (for tree/commit/blob)
    let content = String::from_utf8_lossy(&buf);
    println!("{}", content);

    Ok(())
}
