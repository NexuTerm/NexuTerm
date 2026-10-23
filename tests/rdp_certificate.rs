//! Only the disposable loopback laboratory; never a user's RDP server.
use nexuterm::{
    config::{Profile, Protocol},
    desktop::Desktop,
};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

fn wait(desktop: &Arc<Desktop>, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "Timed out: {}",
            desktop.status.lock().unwrap()
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn connect(ca: bool) -> Arc<Desktop> {
    let lab = Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab");
    Desktop::connect(
        Profile {
            name: "Fictitious local RDP certificate test".into(),
            protocol: Protocol::Rdp,
            host: "127.0.0.1".into(),
            port: 13389,
            username: "tester".into(),
            certificate_authority: if ca {
                lab.join("rdp-ca.pem").to_string_lossy().into()
            } else {
                String::new()
            },
            ..Profile::default()
        },
        zeroize::Zeroizing::new(std::fs::read_to_string(lab.join("password")).unwrap()),
    )
}

#[test]
#[ignore = "requires disposable local lab: python3 lab/lab.py up && python3 lab/lab.py trust"]
fn rdp_requires_explicit_certificate_approval_before_authentication() {
    let lab = Path::new(env!("CARGO_MANIFEST_DIR")).join(".lab");
    let certificate =
        native_tls::Certificate::from_pem(&std::fs::read(lab.join("rdp-ca.pem")).unwrap()).unwrap();
    let fingerprint = Sha256::digest(certificate.to_der().unwrap())
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    for accept in [false, true] {
        let desktop = connect(false);
        wait(&desktop, || desktop.certificate.lock().unwrap().is_some());
        assert!(!desktop.connected.load(Ordering::Relaxed));
        assert!(desktop.frame.lock().unwrap().is_none());
        let challenge = desktop.certificate.lock().unwrap().take().unwrap();
        assert!(challenge.details.contains(&fingerprint));
        assert!(challenge.details.contains("127.0.0.1:13389"));
        challenge.response.send(accept).unwrap();
        if accept {
            wait(&desktop, || desktop.frame.lock().unwrap().is_some());
            assert!(desktop.connected.load(Ordering::Relaxed));
            desktop.close();
        }
        wait(&desktop, || desktop.finished.load(Ordering::Relaxed));
        if !accept {
            assert!(
                desktop
                    .status
                    .lock()
                    .unwrap()
                    .contains("Certificado no aceptado")
            );
        }
    }
    // A new connection must ask again; closing while pending must stop the worker.
    let desktop = connect(false);
    wait(&desktop, || desktop.certificate.lock().unwrap().is_some());
    desktop.close();
    wait(&desktop, || desktop.finished.load(Ordering::Relaxed));
    assert!(!desktop.connected.load(Ordering::Relaxed));
    // A trusted CA still follows normal PKI, without an approval challenge.
    let desktop = connect(true);
    wait(&desktop, || desktop.frame.lock().unwrap().is_some());
    assert!(desktop.certificate.lock().unwrap().is_none());
    desktop.close();
    wait(&desktop, || desktop.finished.load(Ordering::Relaxed));
}
