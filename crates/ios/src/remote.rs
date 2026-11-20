//! The remote pairing a computer without Xcode keeps for a device on Wi-Fi (architecture 7.1, decision D103). Made
//! once over the cable, it opens a tunnel to the developer services that lockdown closes over the network.

use std::net::{IpAddr, SocketAddr, SocketAddrV6};
use std::path::{Path, PathBuf};
use std::time::Duration;

use idevice::remote_pairing::{
    PeerDevice, RemotePairingClient, RemotePairingLockdownService, RpPairingFile, RpPairingSocket,
    connect_tls_psk_tunnel_native,
};
use idevice::rsd::RsdHandshake;
use idevice::{IdeviceError, IdeviceService};
use phonedriver_core::error::Error;
use tokio::net::TcpStream;

use crate::transport::{self, Attached, Tunnel};

/// What a device announces on the network once a computer has paired with it this way.
pub const SERVICE: &str = "_remotepairing._tcp.local.";

/// Over the cable the device asks for a pairing code and shows none. It accepts this one, the code idevice sends
/// when a device asks for consent instead (iPhone17,5, iOS 26.5, 2026-11-20).
const CODE: &str = "000000";

fn path(home: &Path, udid: &str) -> PathBuf {
    home.join("ios-wifi-pairing").join(format!("{udid}.plist"))
}

/// The record `pair` kept for `udid`.
pub fn read(home: &Path, udid: &str) -> Option<RpPairingFile> {
    RpPairingFile::from_bytes(&std::fs::read(path(home, udid)).ok()?).ok()
}

/// The key that tells the device's announcements apart, from the record `pair` kept for `udid`.
pub fn key(home: &Path, udid: &str) -> Option<Vec<u8>> {
    read(home, udid)?.alt_irk().map(<[u8]>::to_vec)
}

fn write(home: &Path, udid: &str, record: &RpPairingFile) -> Result<(), Error> {
    let path = path(home, udid);
    let io = |e: std::io::Error| Error::Transport {
        detail: format!("cannot write {}: {e}", path.display()),
    };
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    std::io::Write::write_all(&mut options.open(&path).map_err(io)?, &record.to_bytes()).map_err(io)
}

/// Deletes the record kept for `udid`, if there is one.
pub fn forget(home: &Path, udid: &str) -> Result<(), Error> {
    let path = path(home, udid);
    match std::fs::remove_file(&path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(Error::Transport {
            detail: format!("cannot delete {}: {e}", path.display()),
        }),
        _ => Ok(()),
    }
}

/// Whether a `SERVICE` announcement with this `identifier` and `auth_tag` comes from the device whose record holds
/// `key`.
pub fn announced_by(key: &[u8], identifier: &str, auth_tag: &str) -> bool {
    PeerDevice::validate_auth_tag(key, identifier, auth_tag)
}

pub fn socket_addr(addr: IpAddr, scope: Option<u32>, port: u16) -> SocketAddr {
    match (addr, scope) {
        (IpAddr::V6(v6), Some(scope)) => SocketAddr::V6(SocketAddrV6::new(v6, port, 0, scope)),
        _ => SocketAddr::new(addr, port),
    }
}

/// Pairs with the device's `remotepairingdeviced` over the cable and keeps the record in `home`. A record that still
/// verifies is kept. The cable pairing already trusts this computer, so nothing shows on the device.
pub async fn pair(device: &Attached, home: &Path) -> Result<(), Error> {
    let id = format!("ios:{}", device.udid);
    let lost = |e| transport::classify(&id, &e);
    let provider = device.provider().await.map_err(lost)?;
    let socket = transport::lockdown_service(
        provider.as_ref(),
        RemotePairingLockdownService::service_name(),
    )
    .await
    .map_err(lost)?;
    let mut client = RemotePairingLockdownService::new(socket)
        .into_client(transport::LABEL)
        .map_err(lost)?;
    let mut record =
        read(home, &device.udid).unwrap_or_else(|| RpPairingFile::generate(transport::LABEL));
    client
        .connect(&mut record, || async { CODE.to_string() })
        .await
        .map_err(lost)?;
    write(home, &device.udid, &record)
}

/// Opens the tunnel through the remote pairing service the device `udid` announces at `addr`, and its RSD service
/// list, or fails after `transport::ANSWER_WITHIN`. It only verifies the record: a new pairing needs the cable.
pub async fn open(
    udid: &str,
    addr: SocketAddr,
    record: &mut RpPairingFile,
) -> Result<Tunnel, Error> {
    open_within(transport::ANSWER_WITHIN, udid, addr, record).await
}

async fn open_within(
    limit: Duration,
    udid: &str,
    addr: SocketAddr,
    record: &mut RpPairingFile,
) -> Result<Tunnel, Error> {
    let id = format!("ios:{udid}");
    transport::answered_within(limit, udid, async {
        tunnel(addr, record)
            .await
            .map_err(|e| transport::classify(&id, &e))
    })
    .await
}

/// The software TCP stack is set up the way idevice's `CoreDeviceProxy::create_software_tunnel` sets it up for the
/// cable.
async fn tunnel(addr: SocketAddr, record: &mut RpPairingFile) -> Result<Tunnel, IdeviceError> {
    let stream = TcpStream::connect(addr).await?;
    let mut client = RemotePairingClient::new(RpPairingSocket::new(stream), transport::LABEL);
    client.attempt_pair_verify().await?;
    client.validate_pairing(record).await?;
    let mut listener = addr;
    listener.set_port(client.create_tcp_listener().await?);
    let tunnel =
        connect_tls_psk_tunnel_native(TcpStream::connect(listener).await?, client.encryption_key())
            .await?;
    let info = tunnel.info.clone();
    let mut adapter = idevice::tcp::adapter::Adapter::new(
        Box::new(tunnel.into_inner()),
        info.client_address.parse()?,
        info.server_address.parse()?,
    );
    adapter.set_mss((info.mtu as usize).saturating_sub(60));
    let mut adapter = adapter.to_async_handle();
    let stream = adapter.connect(info.server_rsd_port).await?;
    let rsd = RsdHandshake::new(stream).await?;
    Ok(Tunnel { adapter, rsd })
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD;
    use idevice::remote_pairing::compute_auth_tag;

    use super::*;

    const KEY: [u8; 16] = [7; 16];

    fn tag(key: &[u8; 16], identifier: &str) -> String {
        STANDARD.encode(compute_auth_tag(key, identifier))
    }

    #[test]
    fn an_announcement_matches_only_the_device_the_record_was_made_with() {
        let ours = tag(&KEY, "ID-2");
        assert!(announced_by(&KEY, "ID-2", &ours));
        assert!(!announced_by(&[8; 16], "ID-2", &ours));
        assert!(!announced_by(&KEY, "ID-3", &ours));
        assert!(!announced_by(&KEY, "ID-2", "not a tag"));
        assert!(!announced_by(&KEY[..8], "ID-2", &ours));
    }

    #[test]
    fn a_record_is_kept_per_device_in_the_home_and_forgetting_it_twice_is_fine() {
        let home = tempfile::tempdir().unwrap();
        assert!(read(home.path(), "A").is_none());
        let mut record = RpPairingFile::generate("phonedriver");
        record.alt_irk = Some(KEY.to_vec());
        write(home.path(), "A", &record).unwrap();
        let back = read(home.path(), "A").unwrap();
        assert_eq!(back.identifier(), record.identifier());
        assert_eq!(back.alt_irk(), Some(&KEY[..]));
        assert_eq!(key(home.path(), "A"), Some(KEY.to_vec()));
        assert!(read(home.path(), "B").is_none());
        assert_eq!(key(home.path(), "B"), None);
        forget(home.path(), "A").unwrap();
        assert!(read(home.path(), "A").is_none());
        forget(home.path(), "A").unwrap();
    }

    #[tokio::test]
    async fn a_device_that_takes_the_connection_and_never_answers_fails_the_tunnel_instead_of_waiting_for_good()
     {
        let silent = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = silent.local_addr().unwrap();
        let holding = tokio::spawn(async move {
            let mut held = vec![];
            while let Ok((stream, _)) = silent.accept().await {
                held.push(stream);
            }
        });
        let mut record = RpPairingFile::generate("phonedriver");
        let opened = open_within(Duration::from_millis(200), "A", addr, &mut record).await;
        holding.abort();
        let detail = match opened {
            Err(Error::Transport { detail }) => detail,
            Err(e) => panic!("{e:?}"),
            Ok(_) => panic!("a tunnel opened to a socket that never answers"),
        };
        assert!(detail.starts_with("ios:A did not answer"), "{detail}");
    }

    #[test]
    fn a_link_local_address_keeps_its_interface_in_the_socket_address() {
        assert_eq!(
            socket_addr("192.168.1.164".parse().unwrap(), None, 49152).to_string(),
            "192.168.1.164:49152"
        );
        assert_eq!(
            socket_addr("fe80::1".parse().unwrap(), Some(14), 49152).to_string(),
            "[fe80::1%14]:49152"
        );
    }
}
