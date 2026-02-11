// original source: https://github.com/MalpenZibo/ashell/blob/main/src/services/tray/mod.rs
use std::{collections::BTreeSet, env, path::PathBuf, sync::LazyLock};

use freedesktop_icons::lookup;
use iced::{
    Element, Length,
    advanced::{image, svg},
    widget::{Image, Svg},
};
use tracing::debug;

static THEME: &str = "Papirus-Dark";
static SYSTEM_ICON_NAMES: LazyLock<BTreeSet<String>> = LazyLock::new(load_system_icon_names);
static SYSTEM_ICON_ENTRIES: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    SYSTEM_ICON_NAMES
        .iter()
        .map(|name| (name.clone(), normalize_icon_name(name)))
        .collect()
});

pub fn find(icon_name: &str) -> Option<FdIcon> {
    find_icon_path(icon_name)
        .or_else(|| find_similar_icon_path(icon_name))
        .or_else(|| find_prefix_icon_path(icon_name))
        .map(fd_icon_from_path)
}

fn fd_icon_from_path(path: PathBuf) -> FdIcon {
    if path.extension().is_some_and(|ext| ext == "svg") {
        debug!("svg icon found. Path: {path:?}");

        FdIcon::Svg(svg::Handle::from_path(path))
    } else {
        debug!("raster icon found. Path: {path:?}");

        FdIcon::Image(image::Handle::from_path(path))
    }
}

fn find_icon_path(icon_name: &str) -> Option<PathBuf> {
    lookup(icon_name)
        .with_cache()
        .with_theme(THEME)
        .find()
        .or_else(|| lookup(icon_name).with_cache().find())
}

fn similar_icon_names(icon_name: &str) -> Option<Vec<String>> {
    if SYSTEM_ICON_NAMES.is_empty() {
        return None;
    }

    let normalized = normalize_icon_name(icon_name);
    let mut matches = Vec::new();

    for candidate in SYSTEM_ICON_NAMES.iter() {
        let candidate_normalized = normalize_icon_name(candidate);

        if candidate_normalized == normalized {
            continue;
        }

        if candidate_normalized.contains(&normalized)
            || normalized.contains(&candidate_normalized)
            || candidate_normalized.contains(&normalized.replace('-', ""))
        {
            matches.push(candidate.clone());
            if matches.len() >= 5 {
                break;
            }
        }
    }

    if matches.is_empty() {
        None
    } else {
        Some(matches)
    }
}

fn find_similar_icon_path(icon_name: &str) -> Option<PathBuf> {
    similar_icon_names(icon_name).and_then(|candidates| {
        candidates
            .iter()
            .find_map(|candidate| find_icon_path(candidate))
    })
}

fn normalize_icon_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

fn prefix_match_icon(icon_name: &str) -> Option<String> {
    if SYSTEM_ICON_ENTRIES.is_empty() {
        return None;
    }

    let normalized = normalize_icon_name(icon_name);
    let mut candidates: Vec<&(String, String)> = SYSTEM_ICON_ENTRIES.iter().collect();
    let chars: Vec<char> = normalized.chars().collect();

    for (idx, ch) in chars.iter().enumerate() {
        candidates.retain(|(_, name)| name.chars().nth(idx) == Some(*ch));

        if candidates.len() == 1 {
            return Some(candidates[0].0.clone());
        }

        if candidates.is_empty() {
            break;
        }
    }

    candidates.first().map(|(name, _)| name.clone())
}

fn find_prefix_icon_path(icon_name: &str) -> Option<PathBuf> {
    prefix_match_icon(icon_name).and_then(|icon_name| find_icon_path(&icon_name))
}

fn load_system_icon_names() -> BTreeSet<String> {
    list_icon_directories()
        .iter()
        .filter(|maybe_dir| maybe_dir.is_dir())
        .flat_map(|dir| {
            walkdir::WalkDir::new(dir)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file())
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
        })
        .collect()
}

fn list_icon_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Ok(data_home) = env::var("XDG_DATA_HOME") {
        let base = PathBuf::from(data_home);
        dirs.push(base.join("icons"));
        dirs.push(base.join("pixmaps"));
    }

    if let Ok(home) = env::var("HOME") {
        let base = PathBuf::from(home);
        dirs.push(base.join(".local/share/icons"));
        dirs.push(base.join(".local/share/pixmaps"));
    }

    let data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());

    for dir in data_dirs.split(':') {
        if dir.is_empty() {
            continue;
        }
        let base = PathBuf::from(dir);
        dirs.push(base.join("icons"));
        dirs.push(base.join("pixmaps"));
    }

    dirs.push(PathBuf::from("/usr/share/icons"));
    dirs.push(PathBuf::from("/usr/share/pixmaps"));

    dirs.sort();
    dirs.dedup();
    dirs
}

#[derive(Debug, Clone)]
pub enum FdIcon {
    Image(image::Handle),
    Svg(svg::Handle),
}

impl FdIcon {
    pub fn into_elem<'a, Message: 'a>(self, size: impl Into<Length>) -> Element<'a, Message> {
        let size: Length = size.into();
        match self {
            FdIcon::Image(handle) => Element::from(Image::new(handle).height(size)),
            FdIcon::Svg(handle) => Element::from(Svg::new(handle).height(size).width(size)),
        }
    }
}
