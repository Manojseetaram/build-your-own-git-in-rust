use crate::objects::{Kind, Object};
use anyhow::Context;
use std::cmp::Ordering;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Recursively construct a tree object for the given directory.
/// Returns `None` for empty directories.
fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;

    let mut entries = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("bad directory entry in {}", path.display()))?;
        let name = entry.file_name();
        let meta = entry.metadata().context("metadata for directory entry")?;
        entries.push((entry, name, meta));
    }

    // Sort according to Git tree rules
    entries.sort_unstable_by(|a, b| {
        let afn = a.1.as_encoded_bytes();
        let bfn = b.1.as_encoded_bytes();
        let common_len = std::cmp::min(afn.len(), bfn.len());

        match afn[..common_len].cmp(&bfn[..common_len]) {
            Ordering::Equal => {}
            o => return o,
        }

        if afn.len() == bfn.len() {
            return Ordering::Equal;
        }

        let c1 = if let Some(c) = afn.get(common_len).copied() {
            Some(c)
        } else if a.2.is_dir() {
            Some(b'/')
        } else {
            None
        };
        let c2 = if let Some(c) = bfn.get(common_len).copied() {
            Some(c)
        } else if b.2.is_dir() {
            Some(b'/')
        } else {
            None
        };

        c1.cmp(&c2)
    });

    let mut tree_object = Vec::new();

    for (entry, file_name, meta) in entries {
        if file_name == ".git" {
            continue;
        }

        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if (meta.permissions().mode() & 0o111) != 0 {
            "100755"
        } else {
            "100644"
        };

        let path = entry.path();

        // Recursively write directories or blobs
        let hash: [u8; 20] = if meta.is_dir() {
            match write_tree_for(&path)? {
                Some(h) => h,
                None => continue, // skip empty directories
            }
        } else {
            Object::blob_from_file(&path).context("create blob object")?
        };

        // Build tree entry
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(hash);
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        let hash = Object {
            kind: Kind::Tree,
            expected_size: tree_object.len() as u64,
            reader: Cursor::new(tree_object),
        }
        .write_to_objects()
        .context("write tree object")?;

        Ok(Some(hash))
    }
}

/// Entry point for writing the root tree object
pub(crate) fn invoke() -> anyhow::Result<()> {
    let Some(hash) = write_tree_for(Path::new(".")).context("construct root tree object")? else {
        anyhow::bail!("asked to make tree object for empty tree");
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
