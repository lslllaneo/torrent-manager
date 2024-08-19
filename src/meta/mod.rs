use std::{
    collections::HashMap,
    fs::{self, metadata, File, Metadata},
    io::Result,
    os::linux::fs::MetadataExt,
    path::Path,
};

struct Similar {
    inode: u64,
    source_dir: String,
    dest_dir: String,
    link_type: LinkType,
}

#[derive(Debug)]
enum LinkType {
    FILE,
    DIRECTORY,
}

/// get the each file's inode and absolute path of dir recursively, keep them in param map
/// if there is a symlink, it will read the linked path
fn get_inodes(dir: &Path, map: &mut HashMap<u64, String>) -> Result<()> {
    if dir.is_file() {
        let metadata = metadata(dir)?;
        if let Some(ab_path) = fs::canonicalize(dir)?.to_str() {
            map.insert(metadata.st_ino(), ab_path.to_string());
        }
    } else if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            get_inodes(&path, map)?;
        }
    }
    Ok(())
}


/// Read all files in the source path and destination directory to obtain the relationship between all hard-connected files.
/// The key is the inode of the file, and the value is the tuple composed of the absolute path of the source file and the destination file   
pub fn get_linked_files(source_dir: &Path, dest_dir: &Path) -> Result<HashMap<u64, Similar>> {
    let mut source_map = HashMap::new();
    get_inodes(source_dir, &mut source_map)?;

    let mut dest_map = HashMap::new();
    get_inodes(dest_dir, &mut dest_map)?;

    let mut map = HashMap::new();

    for (ino, source_path) in source_map.into_iter() {
        if let Some(dest_path) = dest_map.remove(&ino) {
            let simalir = Similar {
                inode: ino,
                source_dir: source_path,
                dest_dir: dest_path,
                link_type: LinkType::FILE,
            };
            map.insert(ino, simalir);
        }
    }

    Ok(map)
}

#[allow(unused)]
fn read_metadata(dir: &str) -> Result<Metadata> {
    let path = Path::new(dir);
    let f = File::open(path)?;
    let metadata = f.metadata()?;
    Ok(metadata)
}
