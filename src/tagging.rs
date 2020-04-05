use anyhow::*;
use metaflac::Tag;
use std::path::PathBuf;

pub fn tag_file(target_file: &PathBuf) -> Result<()> {
    let parent = parent_directory(&target_file)?;
    let parent_parent = parent_directory(&parent)?;
    let album = extract_file_name(&parent)?;
    let artist = extract_file_name(&parent_parent)?;
    let (track_number, title) = extract_track_info(target_file)?;

    let mut tag = Tag::new();
    let comment = tag.vorbis_comments_mut();
    comment.set_album(vec![album]);
    comment.set_artist(vec![artist]);
    comment.set_title(vec![title]);
    comment.set_track(track_number);
    tag.write_to_path(target_file)?;

    Ok(())
}

fn extract_track_info(path: &PathBuf) -> Result<(u32, String)> {
    let file_name = extract_file_name(path)?;
    let space_idx = file_name
        .find(" ")
        .ok_or(anyhow!("Failed to find a space character in {:?}", path))?;
    let (track_number, title) = file_name.split_at(space_idx);
    let track_number: u32 = track_number
        .parse()
        .map_err(|err| anyhow!("Failed to extract track number from {:?}: {}", path, err))?;
    let title = title.trim().to_string();
    Ok((track_number, title))
}

fn extract_file_name(path: &PathBuf) -> Result<String> {
    path.file_stem()
        .and_then(|name| name.to_str().map(|name| name.to_string()))
        .ok_or(anyhow!("Failed to extract filename from {:?}", path))
}

fn parent_directory(path: &PathBuf) -> Result<PathBuf> {
    path.parent()
        .map(|path| path.to_path_buf())
        .ok_or(anyhow!("Failed to find parent directory for {:?}", path))
}
