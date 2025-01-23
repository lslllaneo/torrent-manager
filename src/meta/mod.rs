use std::{
    collections::HashMap,
    fs::{self, metadata, File, Metadata},
    io::Result,
    os::linux::fs::MetadataExt,
    path::Path,
};

use serde::Serialize;

use crate::req::{self, Torrent};

#[derive(Debug, Serialize)]
pub(crate) struct Similar {
    inode: u64,
    source_dir: String,
    dest_dir: String,
    link_type: LinkType,
}

#[derive(Debug, Serialize)]
pub(crate) enum LinkType {
    File,
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

#[derive(Debug)]
pub(crate) struct MappringReleation {
    manager_dir: String,
    client_dir: String,
}

/// deal releations between the torrent client's save_dir and the manager's dir, and save the
/// releation of save_dir and torrent
fn deal_torrent_path(
    torrents: Vec<Torrent>,
    map_dirs: HashMap<String, Vec<MappringReleation>>,
) -> HashMap<String, Torrent> {
    let mut dir_torrent = HashMap::new();
    for mut torrent in torrents {
        if let Some(torrent_save_dir) = &torrent.file {
            if let Some(mapping_relations) = map_dirs.get(&torrent.source) {
                let releation = mapping_relations
                    .iter()
                    .filter(|releation| releation.client_dir.starts_with(torrent_save_dir))
                    .last();

                if let Some(releation) = releation {
                    torrent.file = Some(
                        torrent_save_dir.replace(&releation.client_dir, &releation.manager_dir),
                    );
                }
            }

            let file = torrent.file.as_ref().unwrap().clone();

            dir_torrent.insert(file, torrent);
        }
    }

    dir_torrent
}

#[derive(Debug)]
enum SeedDir {
    ALL,
    PARTIRAL,
    NONE
}

/// recursively traverses the specfied folder to get all files and folders, and judge the file
/// or folder is seeding.
fn get_unseed_files(
    torrent_dir_map: &mut HashMap<String, Torrent>,
    torrent_dir: &Path,
    unseeding_dirs: &mut Vec<&Path>,
) -> anyhow::Result<SeedDir> {
    let base_dir = torrent_dir.to_str().unwrap();


    if let Some(torrent) = torrent_dir_map.remove(base_dir) {
        return Ok(SeedDir::NONE);
    }
    // 遍历:
    if torrent_dir.is_dir() {
        let has_seeding = false;
        for enrty in fs::read_dir(torrent_dir)? {
           has_seeding = get_unseed_files(torrent_dir_map, &enrty?.path(), &mut unseeding_dirs)?;  
        }
        if get_unseed_files(, torrent_folders, unseeding_torrents) {
            unseeding_dirs.push(torrent_dir);
        }
    }
    Ok(SeedDir::NONE)
}

#[allow(unused)]
pub async fn list_unseed_files(
    torrent_folders: Vec<&Path>,
    map_dirs: HashMap<String, Vec<MappringReleation>>,
) -> anyhow::Result<Vec<Torrent>> {
    let mut torrents = req::get_torrents().await?;

    // it has more than one torrent in the same dir, but it's no problem

    let mut torrent_dir_map = deal_torrent_path(torrents, map_dirs);

    let mut unseeding_torrents = Vec::new();

    get_unseed_files(
        &mut torrent_dir_map,
        &torrent_folders,
        &mut unseeding_torrents,
    );

    Ok(unseeding_torrents)
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
                link_type: LinkType::File,
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
