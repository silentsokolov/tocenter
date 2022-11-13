// use zip;

use std::io::prelude::*;
use std::io::{Seek, Write};
use std::iter::Iterator;
use zip::result::ZipError;
use zip::write::FileOptions;

use std::fs::{self, File};
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

const METHOD_STORED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);
const SRC: &str = "resources";
const DST: &str = "resources.zip";

fn main() {
    match make_zip(SRC, DST, METHOD_STORED.unwrap()) {
        Ok(_) => println!("build-resources={}={}", SRC, DST),
        Err(e) => panic!("build-resources=error={:?}", e),
    }
}

fn zip_dir<T>(
    items: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip_writer = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in items {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        if path.is_file() {
            println!("build-resources={:?}={:?}", path, name);
            zip_writer.start_file(String::from(name.to_string_lossy()), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip_writer.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            println!("build-resources={:?}={:?}", path, name);
            zip_writer.start_file(String::from(name.to_string_lossy()), options)?;
        }
    }
    zip_writer.finish()?;
    Result::Ok(())
}

fn make_zip(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    fs::remove_file(&path).ok();

    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}
