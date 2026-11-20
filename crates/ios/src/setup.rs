//! What a physical device needs before phonedriver can drive it, and `phonedriver ios setup` (architecture 7.1, 7.4).

use std::path::Path;

use idevice::services::amfi::AmfiClient;
use idevice::services::mobile_image_mounter::ImageMounter;
use phonedriver_core::error::{Error, UserAction};

use crate::transport::{self, Attached, Link, Services};

/// On a Mac, Xcode mounts the developer disk image and builds, signs and starts the runner on a physical device.
/// Elsewhere phonedriver mounts the image itself and starts a runner the person installed (architecture 7.3).
pub const XCODE: bool = cfg!(target_os = "macos");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
    pub developer_mode: bool,
    pub image_mounted: bool,
}

/// Reads the state; a device that does not trust this computer fails with `NeedsUser(TrustComputer)`. `home` holds
/// the remote pairing a device on Wi-Fi is reached through off a Mac.
pub async fn status(device: &Attached, home: &Path) -> Result<Status, Error> {
    let id = format!("ios:{}", device.udid);
    if XCODE && matches!(device.link, Link::Network { .. }) {
        return Ok(status_of(
            &devicectl(&["info", "details"], &device.udid).await?,
        ));
    }
    let lost = |e| transport::classify(&id, &e);
    let read = async {
        let mut services = Services::open(device, home).await?;
        let developer_mode = services
            .client::<AmfiClient>()
            .await
            .map_err(lost)?
            .get_developer_mode_status()
            .await
            .map_err(lost)?;
        let image_mounted = !services
            .client::<ImageMounter>()
            .await
            .map_err(lost)?
            .copy_devices()
            .await
            .map_err(lost)?
            .is_empty();
        Ok(Status {
            developer_mode,
            image_mounted,
        })
    };
    transport::answered_within(transport::ANSWER_WITHIN, &device.udid, read).await
}

fn status_of(details: &serde_json::Value) -> Status {
    let properties = &details["result"]["deviceProperties"];
    Status {
        developer_mode: properties["developerModeStatus"] == "enabled",
        image_mounted: properties["ddiServicesAvailable"] == true,
    }
}

/// Runs `xcrun devicectl device <args> --device <udid>` and returns its JSON report. Xcode's tool reaches a device
/// on the network through its own pairing, where lockdown services are closed to phonedriver (see `device::Host`).
pub async fn devicectl(args: &[&str], udid: &str) -> Result<serde_json::Value, Error> {
    let report = std::env::temp_dir().join(format!(
        "phonedriver-devicectl-{}.json",
        uuid::Uuid::new_v4()
    ));
    let failed = |detail: String| Error::Transport {
        detail: format!("xcrun devicectl device {}: {detail}", args.join(" ")),
    };
    let out = tokio::process::Command::new("xcrun")
        .args(["devicectl", "device"])
        .args(args)
        .args(["--device", udid, "--json-output"])
        .arg(&report)
        .output()
        .await
        .map_err(|e| failed(e.to_string()))?;
    let text = std::fs::read_to_string(&report);
    let _ = std::fs::remove_file(&report);
    if !out.status.success() {
        return Err(failed(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    serde_json::from_str(&text.map_err(|e| failed(e.to_string()))?)
        .map_err(|e| failed(e.to_string()))
}

/// Shows the Developer Mode switch in Settings; the person turns it on and restarts the device.
pub async fn reveal_developer_mode(device: &Attached, home: &Path) -> Result<(), Error> {
    let id = format!("ios:{}", device.udid);
    let lost = |e| transport::classify(&id, &e);
    Services::open(device, home)
        .await?
        .client::<AmfiClient>()
        .await
        .map_err(lost)?
        .reveal_developer_mode_option_in_ui()
        .await
        .map_err(lost)
}

/// Mounts the developer disk image with Xcode's own tool, which holds the personalized files and talks to Apple's
/// signing server; this is the one step that sends anything to Apple (decision D9). Asking devicectl for the
/// image's services mounts it; `manage ddis update` lost its `--device` option in Xcode 27 (observed 2026-11-12).
pub async fn mount_image(udid: &str) -> Result<(), Error> {
    let out = tokio::process::Command::new("xcrun")
        .args([
            "devicectl",
            "device",
            "info",
            "ddiServices",
            "--device",
            udid,
        ])
        .output()
        .await
        .map_err(|e| Error::Transport {
            detail: format!("cannot run xcrun devicectl: {e}"),
        })?;
    if out.status.success() {
        Ok(())
    } else {
        Err(Error::Transport {
            detail: format!(
                "devicectl could not mount the developer disk image: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        })
    }
}

/// Mounts the personalized image from `files` without Xcode. idevice asks Apple's signing server for this device's
/// ticket, the one request to Apple (decision D9), then uploads and mounts the image. Over Wi-Fi the image mounter is
/// reached through the remote pairing's tunnel; lockdown itself still answers there and gives the chip ID.
pub async fn mount_files(
    device: &Attached,
    home: &Path,
    files: &crate::ddi::Files,
) -> Result<(), Error> {
    let id = format!("ios:{}", device.udid);
    let lost = |e| transport::classify(&id, &e);
    let image = files.read(crate::ddi::IMAGE)?;
    let trust_cache = files.read(crate::ddi::TRUST_CACHE)?;
    let manifest = files.read(crate::ddi::MANIFEST)?;
    let provider = device.provider().await.map_err(lost)?;
    let chip = transport::unique_chip_id(provider.as_ref())
        .await
        .map_err(lost)?;
    let mut services = Services::open(device, home).await?;
    let mut mounter = services.client::<ImageMounter>().await.map_err(lost)?;
    match &mut services {
        Services::Lockdown(provider) => {
            mounter
                .mount_personalized(provider.as_ref(), image, trust_cache, &manifest, None, chip)
                .await
        }
        Services::Tunnel(tunnel) => {
            mounter
                .mount_personalized_rsd(
                    &mut tunnel.adapter,
                    &mut tunnel.rsd,
                    image,
                    trust_cache,
                    &manifest,
                    None,
                    chip,
                )
                .await
        }
    }
    .map_err(lost)
}

/// What the person must do before the image can mount. iOS refuses a personalized image while Developer Mode is off
/// and says only `ImageMountFailed` (iPhone17,5, iOS 26.5, 2026-11-20).
pub fn mount_blocked(status: &Status) -> Option<UserAction> {
    (!status.developer_mode).then_some(UserAction::EnableDeveloperMode)
}

/// On a computer without Xcode, mounts the image from the files `phonedriver ios setup` downloaded when it is not
/// mounted: a restart of the device unmounts it.
pub async fn mount_cached(device: &Attached, home: &Path) -> Result<(), Error> {
    let status = status(device, home).await?;
    if status.image_mounted {
        return Ok(());
    }
    if let Some(action) = mount_blocked(&status) {
        return Err(Error::NeedsUser { action });
    }
    match crate::ddi::cached(home) {
        Some(files) => mount_files(device, home, &files).await,
        None => Err(Error::NeedsUser {
            action: UserAction::MountDeveloperImage,
        }),
    }
}

pub fn developer_mode_action() -> UserAction {
    UserAction::EnableDeveloperMode
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn turning_developer_mode_on_is_a_user_action_with_the_settings_path() {
        assert!(developer_mode_action().hint().contains("Developer Mode"));
    }

    #[test]
    fn developer_mode_off_stops_the_mount_before_it_starts() {
        let off = Status {
            developer_mode: false,
            image_mounted: false,
        };
        assert_eq!(mount_blocked(&off), Some(UserAction::EnableDeveloperMode));
        let on = Status {
            developer_mode: true,
            image_mounted: false,
        };
        assert_eq!(mount_blocked(&on), None);
    }

    #[test]
    fn devicectl_details_give_developer_mode_and_the_mounted_image() {
        let ready = json!({"result": {"deviceProperties": {"developerModeStatus": "enabled", "ddiServicesAvailable": true}}});
        assert_eq!(
            status_of(&ready),
            Status {
                developer_mode: true,
                image_mounted: true
            }
        );
        let not = json!({"result": {"deviceProperties": {"developerModeStatus": "disabled", "ddiServicesAvailable": false}}});
        assert_eq!(
            status_of(&not),
            Status {
                developer_mode: false,
                image_mounted: false
            }
        );
    }
}
