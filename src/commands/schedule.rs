//! Schedule sync command implementation.
//!
//! Create a cronjob to run `ferrofeed sync` on a schedule.

use anyhow::{Context, Result};
use grep::{matcher::Matcher, regex::RegexMatcher};
use std::ops::RangeInclusive;
use tokio::{io::AsyncWriteExt, process::Command as TokioCommand};

/// Valid crontab minute range (up to a day).
///   See: `man 5 crontab`
const SCHEDULE_MINUTES_RANGE: RangeInclusive<u32> = 1..=1440;

/// Convert the user-provided schedule minutes to a (crontab-formatted string, human-readable string) pair.
fn minutes_to_crontab_schedule(minutes: u32) -> Result<(String, String)> {
    match minutes {
        0 => Err(anyhow::anyhow!(
            "Invalid schedule minutes, must be between 1 and {}",
            SCHEDULE_MINUTES_RANGE.end()
        )),
        m @ 1..=59 => Ok((format!("*/{} * * * *", m), format!("every {} minutes", m))),
        60 => Ok(("0 * * * *".to_string(), "every hour".to_string())),
        m if m <= *SCHEDULE_MINUTES_RANGE.end() && m % 60 == 0 => {
            let hrs = m / 60;
            Ok((format!("{} * * * *", hrs), format!("every {} hours", hrs)))
        }
        m if m > *SCHEDULE_MINUTES_RANGE.end() => Err(anyhow::anyhow!(
            "Maximum schedule minutes is {}",
            SCHEDULE_MINUTES_RANGE.end()
        )),
        m => {
            // For non-hourly intervals, use a combination approach
            let hours = m / 60;
            let mins = m % 60;
            Ok((
                format!("{} */{} * * *", mins, hours),
                format!("every {} hours and {} minutes", hours, mins),
            ))
        }
    }
}

/// Schedule `ferrofeed sync` to run on a schedule using `crontab`.
/// TODO: respect/update user's ferrofeed config
pub async fn schedule(minutes: u32) -> Result<()> {
    let exe_path = std::env::current_exe().context("failed to get `ferrofeed` executeable path")?;
    let (crontab_schedule, human_schedule) = minutes_to_crontab_schedule(minutes)?;
    let sync_command = format!("{} sync", exe_path.display());

    // Check that crontab is installed
    let crontab_output = TokioCommand::new("crontab").arg("-l").output().await;
    let mut existing_crontab = match crontab_output {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)?,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no crontab") {
                String::new()
            } else {
                return Err(anyhow::anyhow!("crontab failed: {}", stderr));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(anyhow::anyhow!(
                "`crontab` not installed, please install it to use this feature"
            ));
        }
        Err(e) => return Err(e.into()),
    };

    // Check if crontab already has a job for ferrofeed
    let matcher = RegexMatcher::new(&sync_command.to_string())?;
    if (matcher.find(existing_crontab.as_bytes())?).is_some() {
        println!("ferrofeed sync already scheduled, updating...");

        // Remove existing crontab entries
        existing_crontab = existing_crontab
            .lines()
            .filter(|line| {
                // filter out lines with matches
                matcher.find(line.as_bytes()).ok().flatten().is_none()
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !existing_crontab.is_empty() {
            existing_crontab.push('\n');
        }
    }

    // Append new crontab entry
    let mut new_crontab = existing_crontab;
    if !new_crontab.is_empty() && !new_crontab.ends_with('\n') {
        new_crontab.push('\n');
    }
    println!("Scheduling ferrofeed sync to run {}", human_schedule);
    new_crontab.push_str(&format!("{} {}\n", crontab_schedule, sync_command));
    let mut child = TokioCommand::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .context("failed to spawn `crontab`")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(new_crontab.as_bytes())
            .await
            .context("failed to write to `crontab`")?;
    }
    // Wait for crontab to exit
    let status = child.wait().await.context("failed to wait for `crontab`")?;
    if !status.success() {
        return Err(anyhow::anyhow!("`crontab` failed: {}", status));
    }

    println!("✓ ferrofeed sync scheduled");
    Ok(())
}
