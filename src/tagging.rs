use std::fs;
use std::path::PathBuf;

use anyhow::*;
use audiotags::{Album, MimeType, Picture, Tag};

pub fn tag_file(target_path: &PathBuf, cover_path: Option<PathBuf>) -> Result<()> {
    let parent = parent_directory(&target_path)?;
    let parent_parent = parent_directory(&parent)?;
    let album = extract_file_name(&parent)?;
    let artist = extract_file_name(&parent_parent)?;
    let (track_number, title) = extract_track_info(target_path)?;

    let mut tag = Tag::new().read_from_path(target_path)?;
    tag.set_album(Album::with_title(&album).and_artist(&artist));
    tag.set_title(&title);
    tag.set_track_number(track_number);

    if let Some(cover_path) = cover_path {
        let mime_type = cover_mime_type(&cover_path)?;
        let cover_bytes = fs::read(&cover_path)?;
        let picture = Picture {
            data: &cover_bytes,
            mime_type,
        };
        tag.set_album_cover(picture)
    }

    let path_as_str = target_path.to_str().ok_or_else(|| anyhow!("Failed to convert {:?} to a string"))?;
    tag.write_to_path(path_as_str)?;

    Ok(())
}

fn extract_track_info(path: &PathBuf) -> Result<(u16, String)> {
    let file_name = extract_file_name(path)?;
    let space_idx = file_name
        .find(' ')
        .ok_or_else(|| anyhow!("Failed to find a space character in {:?}", path))?;
    let (track_number, title) = file_name.split_at(space_idx);
    let track_number: u16 = track_number
        .parse()
        .map_err(|err| anyhow!("Failed to extract track number from {:?}: {}", path, err))?;
    let title = title.trim().to_string();
    Ok((track_number, title))
}

fn extract_file_name(path: &PathBuf) -> Result<String> {
    path.file_stem()
        .and_then(|name| name.to_str().map(|name| name.to_string()))
        .ok_or_else(|| anyhow!("Failed to extract filename from {:?}", path))
}

fn parent_directory(path: &PathBuf) -> Result<PathBuf> {
    path.parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| anyhow!("Failed to find parent directory for {:?}", path))
}

fn cover_mime_type(cover_path: &PathBuf) -> Result<MimeType> {
    let extension = cover_path
        .extension()
        .and_then(|os_string| os_string.to_str())
        .ok_or_else(|| anyhow!("Could not extract extension from {:?}", cover_path))?;

    match extension {
        "jpg" => Ok(MimeType::Jpeg),
        "bmp" => Ok(MimeType::Bmp),
        "gif" => Ok(MimeType::Gif),
        "png" => Ok(MimeType::Png),
        "tiff" => Ok(MimeType::Tiff),
        _ => Err(anyhow!("Unsupported extension: {}", extension)),
    }
}
