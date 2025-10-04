use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::io::{self, prelude::*, BufReader, Cursor};
use std::path::Path;

use sha1::{Digest, Sha1};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) _expected_sizes: u64,
    pub(crate) reader: R,
}

// =======================
// Reading an object
// =======================
impl Object<()> {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let f = fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(f);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("read header from .git/objects")?;
        let header = CStr::from_bytes_with_nul(&buf).expect("exactly one nul at the end");
        let header = header
            .to_str()
            .context(".git/objects file header isn't valid UTF-8")?;
        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };
        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("unknown object type '{kind}'"),
        };
        let size = size.parse::<u64>().context("invalid size in header")?;
        let z = z.take(size);
        Ok(Object {
            kind,
            _expected_sizes: size,
            reader: z,
        })
    }
}

// =======================
// Writing an object
// =======================
impl<R: io::Read> Object<R> {
    /// Write the object to .git/objects and return raw SHA-1 bytes
    pub(crate) fn write_to_objects(mut self) -> anyhow::Result<[u8; 20]> {
        let mut data = Vec::new();
        self.reader.read_to_end(&mut data)?;

        // Build header
        let header = format!("{} {}\0", self.kind, data.len());
        let mut store = header.into_bytes();
        store.extend(data);

        // Compute SHA-1 hash
        let hash = Sha1::digest(&store);
        let hash_bytes: [u8; 20] = hash.into();

        // Write compressed object to .git/objects
        let dir = format!(".git/objects/{:02x}", hash_bytes[0]);
        fs::create_dir_all(&dir)?;
        let path = format!("{}/{}", dir, hex::encode(&hash_bytes[1..]));
        let file = fs::File::create(path)?;
        let mut encoder = ZlibEncoder::new(file, Compression::default());
        io::copy(&mut Cursor::new(store), &mut encoder)?;
        encoder.finish()?;

        Ok(hash_bytes)
    }
}

// =======================
// Convenience: create blob from file
// =======================
impl Object<BufReader<fs::File>> {
    pub(crate) fn blob_from_file(path: &Path) -> anyhow::Result<[u8; 20]> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);

        let obj = Object {
            kind: Kind::Blob,
            _expected_sizes: fs::metadata(path)?.len(),
            reader,
        };

        obj.write_to_objects()
    }
}
