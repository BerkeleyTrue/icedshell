use std::path::{Path, PathBuf};

use derive_more::Deref;
use iced::futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use tokio::fs;

async fn find_coretemp_path() -> anyhow::Result<Option<PathBuf>> {
    let mut dirs = fs::read_dir("/sys/bus/platform/devices/").await?;

    let mut paths = Vec::new();

    while let Some(entry) = dirs.next_entry().await? {
        let file_type = entry.file_type().await?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("coretemp.") && file_type.is_dir() {
            paths.push(entry.path());
        }
    }

    Ok(paths.first().cloned())
}

async fn get_hwmon_paths() -> anyhow::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if let Some(path) = find_coretemp_path().await? {
        let mut dirs = fs::read_dir(path.join("hwmon")).await?;

        while let Some(entry) = dirs.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("hwmon") {
                paths.push(entry.path());
            }
        }
    } else {
        // fallback to /sys/class/hwmon
        let mut dirs = fs::read_dir("/sys/class/hwmon/").await?;

        while let Some(entry) = dirs.next_entry().await? {
            let file_type = entry.file_type().await?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with("hwmon") && file_type.is_dir() {
                paths.push(entry.path());
            }
        }
    }

    Ok(paths)
}

async fn is_core_label(path: &Path) -> anyhow::Result<bool> {
    let content = fs::read_to_string(path).await?;
    let label = &content[..4.min(content.len())];
    Ok(matches!(label, "Core" | "Tdie" | "Tctl"))
}

async fn get_core_paths() -> anyhow::Result<Vec<PathBuf>> {
    let hwmon_paths = get_hwmon_paths().await?;
    let mut paths = Vec::new();

    for path in hwmon_paths {
        let mut dirs = fs::read_dir(path).await?;

        while let Some(entry) = dirs.next_entry().await? {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let path = entry.path();
            if name.starts_with("temp")
                && name.ends_with("_label")
                && fs::try_exists(&path).await?
                && is_core_label(&path).await?
            {
                let input_path = PathBuf::from(path.to_string_lossy().replace("_label", "_input"));

                if fs::try_exists(&input_path).await? {
                    paths.push(input_path);
                }
            }
        }
    }

    Ok(paths)
}

async fn read_core_temps() -> anyhow::Result<Vec<f32>> {
    let core_paths = get_core_paths().await?;
    let mut temps = Vec::new();

    for core_path in core_paths {
        let content = fs::read_to_string(&core_path).await?;
        let temp: f32 = content.trim().parse().unwrap_or(0.);
        temps.push(temp);
    }
    Ok(temps)
}

async fn average_core_temp() -> anyhow::Result<f32> {
    let temps = read_core_temps().await?;

    let avg_milli = if temps.is_empty() {
        0.
    } else {
        let added: f32 = temps.iter().sum();
        added / temps.len() as f32
    };

    Ok((avg_milli / 1000.).round())
}

#[derive(Hash, Deref)]
pub struct ListenData(pub u64);

pub fn listen<'a>(delay: &ListenData) -> BoxStream<'a, anyhow::Result<f32>> {
    let delay = delay.0;
    stream::repeat(())
        .then(move |_| async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
            average_core_temp().await
        })
        .boxed()
}
