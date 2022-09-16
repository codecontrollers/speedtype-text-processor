use walkdir::{DirEntry, WalkDir};

pub fn walk<'a>(root: &'a str, extension: &'a str) -> Vec<DirEntry> {
    let dirs = WalkDir::new(root)
        .into_iter()
        .flat_map(|e| e)
        .filter(move |e| {
            e.file_name()
                .to_str()
                .map_or(false, |n| n.ends_with(extension))
        })
        .into_iter();

    let dirvec: Vec<DirEntry> = dirs.collect();

    dirvec
}
