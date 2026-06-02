use iced::futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use tokio::fs;

#[derive(Debug, Clone)]
pub enum BatteryState {
    Full,
    Charging(f64),
    Discharging(f64),
    Low(f64),
    None,
}

async fn has_battery(bat: &str) -> anyhow::Result<bool> {
    let bat_dir = format!("/sys/class/power_supply/{}", bat);
    let mut entries = fs::read_dir(bat_dir).await?;
    Ok(entries.next_entry().await?.is_some())
}

async fn get_cap(bat: &str) -> f64 {
    let charge_now_path = format!("/sys/class/power_supply/{}/charge_now", bat);
    let charge_full_path = format!("/sys/class/power_supply/{}/charge_full", bat);
    let charge_now = fs::read_to_string(&charge_now_path)
        .await
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok());

    let charge_full = fs::read_to_string(&charge_full_path)
        .await
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok());

    match (charge_now, charge_full) {
        (Some(now), Some(full)) if full > 0.0 => ((now / full) * 100.0).round(),
        _ => 100.,
    }
}

async fn get_status(bat: &str) -> BatteryState {
    let cap = get_cap(bat).await;
    let status_path = format!("/sys/class/power_supply/{}/status", bat);
    let status = fs::read_to_string(&status_path)
        .await
        .unwrap_or_else(|_| "Unknown".to_string());

    match status.trim() {
        "Full" => BatteryState::Full,
        "Charging" => BatteryState::Charging(cap),
        "Discharging" if cap < 10.0 => BatteryState::Low(cap),
        _ => BatteryState::Discharging(cap),
    }
}

#[derive(Hash)]
pub struct ListenData {
    pub delay: u64,
    pub bat: String,
}

pub fn listen<'a>(
    ListenData { delay, bat }: &ListenData,
) -> BoxStream<'a, anyhow::Result<BatteryState>> {
    let delay = *delay;
    let bat = bat.clone();
    stream::repeat(())
        .then(move |_| {
            let bat = bat.clone();
            async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                if has_battery(&bat).await.is_ok_and(|x| x) {
                    let stat = get_status(&bat).await;
                    Ok(stat)
                } else {
                    Ok(BatteryState::None)
                }
            }
        })
        .boxed()
}
