use crate::{
    config::Profile,
    desktop::{Desktop, Frame, Input},
};
use ironrdp::pdu::input::{
    fast_path::{FastPathInputEvent, KeyboardFlags},
    mouse::{MousePdu, PointerFlags},
};
use ironrdp::{
    connector::{self, Credentials},
    pdu::{
        gcc::KeyboardType,
        rdp::{
            capability_sets::MajorPlatformType,
            client_info::{PerformanceFlags, TimezoneInfo},
        },
    },
    session::{ActiveStageBuilder, ActiveStageOutput, image::DecodedImage},
};
use ironrdp_tokio::FramedWrite;
use std::sync::{Arc, atomic::Ordering};
use tokio::sync::mpsc;
use x509_cert::der::Decode;

pub async fn run(
    profile: Profile,
    password: zeroize::Zeroizing<String>,
    mut input: mpsc::Receiver<Input>,
    desktop: Arc<Desktop>,
) -> anyhow::Result<()> {
    let socket = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        tokio::net::TcpStream::connect((profile.host.as_str(), profile.port)),
    )
    .await??;
    let config = connector::Config {
        credentials: Credentials::UsernamePassword {
            username: profile.username.clone(),
            password: password.to_string(),
        },
        domain: (!profile.domain.is_empty()).then_some(profile.domain.clone()),
        enable_tls: true,
        enable_credssp: true,
        keyboard_type: KeyboardType::IbmEnhanced,
        keyboard_subtype: 0,
        keyboard_layout: 0,
        keyboard_functional_keys_count: 12,
        ime_file_name: String::new(),
        dig_product_id: String::new(),
        desktop_size: connector::DesktopSize {
            width: 1280,
            height: 800,
        },
        bitmap: None,
        client_build: 0,
        client_name: "NexuTerm".into(),
        client_dir: String::new(),
        platform: MajorPlatformType::UNIX,
        enable_server_pointer: false,
        request_data: None,
        autologon: true,
        enable_audio_playback: false,
        compression_type: None,
        pointer_software_rendering: false,
        multitransport_flags: None,
        performance_flags: PerformanceFlags::default(),
        desktop_scale_factor: 0,
        hardware_id: None,
        license_cache: None,
        timezone_info: TimezoneInfo::default(),
        alternate_shell: String::new(),
        work_dir: String::new(),
    };
    // First use normal PKI. A second, unauthenticated TLS session may only
    // proceed after the user approves its exact certificate on this same stream.
    let mut first_socket = Some(socket);
    let mut validation_error: Option<String> = None;
    let (mut connector, upgrade, stream) = loop {
        let socket = if let Some(socket) = first_socket.take() {
            socket
        } else {
            tokio::time::timeout(
                std::time::Duration::from_secs(15),
                tokio::net::TcpStream::connect((profile.host.as_str(), profile.port)),
            )
            .await??
        };
        let addr = socket.local_addr()?;
        let mut connector = connector::ClientConnector::new(config.clone(), addr);
        connector.attach_static_channel(ironrdp::dvc::DrdynvcClient::new().with_dynamic_channel(
            ironrdp::displaycontrol::client::DisplayControlClient::new(|_| Ok(Vec::new())),
        ));
        let mut framed = ironrdp_tokio::TokioFramed::new(socket);
        let upgrade = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            ironrdp_tokio::connect_begin(&mut framed, &mut connector),
        )
        .await??;
        let mut tls = native_tls::TlsConnector::builder();
        if !profile.certificate_authority.is_empty() {
            let pem = tokio::fs::read(&profile.certificate_authority).await?;
            tls.add_root_certificate(native_tls::Certificate::from_pem(&pem)?);
        }
        if validation_error.is_some() {
            // No RDP credentials are sent on this connection until approval.
            tls.danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true);
        }
        let tls = tokio_native_tls::TlsConnector::from(tls.build()?);
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            tls.connect(&profile.host, framed.into_inner_no_leftover()),
        )
        .await?;
        match result {
            Ok(stream) => {
                if let Some(error) = validation_error {
                    let certificate = stream
                        .get_ref()
                        .peer_certificate()?
                        .ok_or_else(|| anyhow::anyhow!("El servidor no aporta certificado"))?;
                    let der = certificate.to_der()?;
                    let details = certificate_details(&profile.host, profile.port, &der, &error)?;
                    let (response, receiver) = tokio::sync::oneshot::channel();
                    *desktop.status.lock().unwrap() =
                        "Esperando confirmación del certificado RDP…".into();
                    *desktop.certificate.lock().unwrap() =
                        Some(crate::desktop::CertificateChallenge { details, response });
                    let accepted =
                        tokio::time::timeout(std::time::Duration::from_secs(120), receiver).await;
                    anyhow::ensure!(
                        matches!(accepted, Ok(Ok(true))),
                        "Certificado no aceptado; conexión cancelada"
                    );
                    anyhow::ensure!(!desktop.stop.load(Ordering::Relaxed), "Conexión cancelada");
                }
                break (connector, upgrade, stream);
            }
            Err(error) if validation_error.is_none() => {
                validation_error = Some(error.to_string());
            }
            Err(error) => return Err(error.into()),
        }
    };
    let certificate = stream
        .get_ref()
        .peer_certificate()?
        .ok_or_else(|| anyhow::anyhow!("El servidor no aporta certificado"))?;
    let cert = x509_cert::Certificate::from_der(&certificate.to_der()?)?;
    let key = cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .as_bytes()
        .ok_or_else(|| anyhow::anyhow!("Clave pública TLS inválida"))?
        .to_vec();
    let upgraded = ironrdp_tokio::mark_as_upgraded(upgrade, &mut connector);
    let mut framed = ironrdp_tokio::TokioFramed::new(stream);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        ironrdp_tokio::connect_finalize(
            upgraded,
            connector,
            &mut framed,
            &mut ironrdp_tokio::reqwest::ReqwestNetworkClient::new(),
            profile.host.into(),
            key,
            None,
        ),
    )
    .await??;
    anyhow::ensure!(
        result.desktop_size.width <= 8192
            && result.desktop_size.height <= 8192
            && usize::from(result.desktop_size.width) * usize::from(result.desktop_size.height)
                <= 16 * 1024 * 1024,
        "Framebuffer fuera del límite de 64 MiB"
    );
    let mut image = DecodedImage::new(
        ironrdp::graphics::image_processing::PixelFormat::RgbA32,
        result.desktop_size.width,
        result.desktop_size.height,
    );
    let mut stage = ActiveStageBuilder {
        static_channels: result.static_channels,
        user_channel_id: result.user_channel_id,
        io_channel_id: result.io_channel_id,
        message_channel_id: result.message_channel_id,
        share_id: result.share_id,
        compression_type: result.compression_type,
        enable_server_pointer: result.enable_server_pointer,
        pointer_software_rendering: result.pointer_software_rendering,
    }
    .build();
    let (mut reader, mut writer) = ironrdp_tokio::split_tokio_framed(framed);
    desktop.connected.store(true, Ordering::Relaxed);
    *desktop.status.lock().unwrap() = "RDP conectado".into();
    let mut buttons_held = 0u8;
    loop {
        if desktop.stop.load(Ordering::Relaxed) {
            break;
        }
        let outputs = tokio::select! {
            data=reader.read_pdu()=>{let(action,payload)=data?;stage.process(&mut image,action,&payload)?},
            event=input.recv()=>{
                match event{
                    None|Some(Input::Close)=>break,
                    Some(Input::Resize{width,height})=>{if let Some(frame)=stage.encode_resize(width.into(),height.into(),Some(100),None){writer.write_all(&frame?).await?;}continue;},
                    Some(Input::Key{keysym,down})=>{stage.process_fastpath_input(&mut image, &key_events(keysym, down))?},
                    Some(Input::Pointer{x,y,buttons})=>{let events = pointer_events(x,y,buttons,&mut buttons_held);stage.process_fastpath_input(&mut image,&events)?},
                    Some(Input::Clipboard(_))=>continue,
                }
            }
        };
        let mut updated = false;
        for output in outputs {
            match output {
                ActiveStageOutput::ResponseFrame(frame) => writer.write_all(&frame).await?,
                ActiveStageOutput::GraphicsUpdate(_) => updated = true,
                ActiveStageOutput::Terminate(_) => return Ok(()),
                _ => {}
            }
        }
        if image.width() > 8192 || image.height() > 8192 || image.data().len() > 64 * 1024 * 1024 {
            anyhow::bail!("Framebuffer fuera del límite de 64 MiB");
        }
        if updated {
            *desktop.frame.lock().unwrap() = Some(Frame {
                width: image.width(),
                height: image.height(),
                rgba: image.data().to_vec(),
            });
        }
    }
    if let Ok(outputs) = stage.graceful_shutdown() {
        for output in outputs {
            if let ActiveStageOutput::ResponseFrame(frame) = output {
                let _ = tokio::time::timeout(
                    std::time::Duration::from_millis(200),
                    writer.write_all(&frame),
                )
                .await;
            }
        }
    }
    Ok(())
}
fn certificate_details(host: &str, port: u16, der: &[u8], error: &str) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let certificate = x509_cert::Certificate::from_der(der)?;
    let fingerprint = Sha256::digest(der)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":");
    Ok(format!(
        "Servidor: {host}:{port}\nTitular: {}\nEmisor: {}\nValidez: {} — {}\n\nHuella SHA-256:\n{fingerprint}\n\nNo se ha podido verificar su identidad. Comprueba esta huella con el administrador por otro medio antes de confiar. La autorización solo vale para esta sesión y este certificado.\n\nMotivo: {}",
        certificate.tbs_certificate.subject,
        certificate.tbs_certificate.issuer,
        certificate.tbs_certificate.validity.not_before,
        certificate.tbs_certificate.validity.not_after,
        error.chars().take(500).collect::<String>()
    ))
}

fn key_events(key: u32, down: bool) -> Vec<FastPathInputEvent> {
    let mut flags = if down {
        KeyboardFlags::empty()
    } else {
        KeyboardFlags::RELEASE
    };
    let scan = match key {
        0xff0d => Some((0x1c, false)),
        0xff08 => Some((0x0e, false)),
        0xff09 | 0xfe20 => Some((0x0f, false)),
        0xff1b => Some((0x01, false)),
        0xffe1 => Some((0x2a, false)),
        0xffe2 => Some((0x36, false)),
        0xffe3 => Some((0x1d, false)),
        0xffe4 => Some((0x1d, true)),
        0xffe9 => Some((0x38, false)),
        0xffea | 0xfe03 => Some((0x38, true)),
        0xffeb => Some((0x5b, true)),
        0xffec => Some((0x5c, true)),
        0xff50 => Some((0x47, true)),
        0xff57 => Some((0x4f, true)),
        0xff51 => Some((0x4b, true)),
        0xff52 => Some((0x48, true)),
        0xff53 => Some((0x4d, true)),
        0xff54 => Some((0x50, true)),
        0xff55 => Some((0x49, true)),
        0xff56 => Some((0x51, true)),
        0xff63 => Some((0x52, true)),
        0xffff => Some((0x53, true)),
        0xffe5 => Some((0x3a, false)),
        0xffbe..=0xffc7 => Some(((key - 0xffbe) as u8 + 0x3b, false)),
        0xffc8 => Some((0x57, false)),
        0xffc9 => Some((0x58, false)),
        0x61..=0x7a | 0x41..=0x5a => {
            let index = (key as u8).to_ascii_lowercase() - b'a';
            Some((
                [
                    0x1e, 0x30, 0x2e, 0x20, 0x12, 0x21, 0x22, 0x23, 0x17, 0x24, 0x25, 0x26, 0x32,
                    0x31, 0x18, 0x19, 0x10, 0x13, 0x1f, 0x14, 0x16, 0x2f, 0x11, 0x2d, 0x15, 0x2c,
                ][usize::from(index)],
                false,
            ))
        }
        0x31..=0x39 => Some(((key - 0x31) as u8 + 2, false)),
        0x30 => Some((0x0b, false)),
        0x20 => Some((0x39, false)),
        _ => None,
    };
    if let Some((scan, extended)) = scan {
        if extended {
            flags |= KeyboardFlags::EXTENDED;
        }
        return vec![FastPathInputEvent::KeyboardEvent(flags, scan)];
    }
    let scalar = if key & 0xff000000 == 0x01000000 {
        key & 0x00ffffff
    } else if key <= 0xff {
        key
    } else {
        return Vec::new();
    };
    let Some(character) = char::from_u32(scalar).filter(|c| !c.is_control()) else {
        return Vec::new();
    };
    character
        .encode_utf16(&mut [0; 2])
        .iter()
        .map(|unit| FastPathInputEvent::UnicodeKeyboardEvent(flags, *unit))
        .collect()
}

// The viewer sends VNC-style button state, including transient wheel bits.
fn pointer_events(x: u16, y: u16, buttons: u8, held: &mut u8) -> Vec<FastPathInputEvent> {
    let event = |flags, units| {
        FastPathInputEvent::MouseEvent(MousePdu {
            flags,
            number_of_wheel_rotation_units: units,
            x_position: x,
            y_position: y,
        })
    };
    let mut events = vec![event(PointerFlags::MOVE, 0)];
    for (bit, flag) in [
        (1, PointerFlags::LEFT_BUTTON),
        (2, PointerFlags::MIDDLE_BUTTON_OR_WHEEL),
        (4, PointerFlags::RIGHT_BUTTON),
    ] {
        if (buttons ^ *held) & bit != 0 {
            events.push(event(
                if buttons & bit != 0 {
                    flag | PointerFlags::DOWN
                } else {
                    flag
                },
                0,
            ));
        }
    }
    for (bit, flag, units) in [
        (8, PointerFlags::VERTICAL_WHEEL, 120),
        (16, PointerFlags::VERTICAL_WHEEL, -120),
        (32, PointerFlags::HORIZONTAL_WHEEL, -120),
        (64, PointerFlags::HORIZONTAL_WHEEL, 120),
    ] {
        if buttons & bit != 0 {
            events.push(event(flag, units));
        }
    }
    *held = buttons & 7;
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mouse_releases_and_drag_do_not_stick() {
        let mut held = 0;
        let press = pointer_events(12, 24, 4, &mut held);
        assert!(
            matches!(press[1], FastPathInputEvent::MouseEvent(MousePdu { flags, .. }) if flags == PointerFlags::RIGHT_BUTTON | PointerFlags::DOWN)
        );
        assert_eq!(pointer_events(13, 25, 4, &mut held).len(), 1);
        let release = pointer_events(13, 25, 0, &mut held);
        assert!(
            matches!(release[1], FastPathInputEvent::MouseEvent(MousePdu { flags, .. }) if flags == PointerFlags::RIGHT_BUTTON)
        );
        let wheel = pointer_events(13, 25, 16, &mut held);
        assert!(matches!(
            wheel[1],
            FastPathInputEvent::MouseEvent(MousePdu {
                number_of_wheel_rotation_units: -120,
                ..
            })
        ));
    }
    #[test]
    fn keyboard_handles_navigation_and_unicode() {
        assert!(
            matches!(key_events(0xff51, false)[0], FastPathInputEvent::KeyboardEvent(flags,0x4b) if flags.contains(KeyboardFlags::EXTENDED | KeyboardFlags::RELEASE))
        );
        assert!(matches!(
            key_events(0x010020ac, true)[0],
            FastPathInputEvent::UnicodeKeyboardEvent(_, 0x20ac)
        ));
        assert_eq!(key_events(0x0101f600, true).len(), 2);
        assert!(key_events(0xffaa, true).is_empty());
    }
}
