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
    let addr = socket.local_addr()?;
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
    let mut connector = connector::ClientConnector::new(config, addr);
    connector.attach_static_channel(ironrdp::dvc::DrdynvcClient::new().with_dynamic_channel(
        ironrdp::displaycontrol::client::DisplayControlClient::new(|_| Ok(Vec::new())),
    ));
    let mut framed = ironrdp_tokio::TokioFramed::new(socket);
    let upgrade = ironrdp_tokio::connect_begin(&mut framed, &mut connector).await?;
    let stream = framed.into_inner_no_leftover();
    let mut tls = native_tls::TlsConnector::builder();
    if !profile.certificate_authority.is_empty() {
        let pem = tokio::fs::read(&profile.certificate_authority).await?;
        tls.add_root_certificate(native_tls::Certificate::from_pem(&pem)?);
    }
    // Normal PKI and hostname verification remain enabled, unlike upstream demo adapters.
    let tls = tokio_native_tls::TlsConnector::from(tls.build()?);
    let stream = tls.connect(&profile.host, stream).await?;
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
    let result = ironrdp_tokio::connect_finalize(
        upgraded,
        connector,
        &mut framed,
        &mut ironrdp_tokio::reqwest::ReqwestNetworkClient::new(),
        profile.host.into(),
        key,
        None,
    )
    .await?;
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
                    Some(Input::Key{keysym,down})=>{let flags=if down{KeyboardFlags::empty()}else{KeyboardFlags::RELEASE};let event=if let Some(scan)=scancode(keysym){FastPathInputEvent::KeyboardEvent(flags,scan)}else if let Some(c)=char::from_u32(keysym).filter(|c|!c.is_control()){FastPathInputEvent::UnicodeKeyboardEvent(flags,c as u16)}else{continue;};stage.process_fastpath_input(&mut image,&[event])?},
                    Some(Input::Pointer{x,y,buttons})=>{let flags=if buttons==0{PointerFlags::MOVE}else{PointerFlags::DOWN|match buttons{1=>PointerFlags::LEFT_BUTTON,2=>PointerFlags::RIGHT_BUTTON,_=>PointerFlags::MIDDLE_BUTTON_OR_WHEEL}};stage.process_fastpath_input(&mut image,&[FastPathInputEvent::MouseEvent(MousePdu{flags,number_of_wheel_rotation_units:0,x_position:x,y_position:y})])?},
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
    Ok(())
}
fn scancode(key: u32) -> Option<u8> {
    Some(match key {
        0xff0d => 0x1c,
        0xff08 => 0x0e,
        0xff09 => 0x0f,
        0xff1b => 0x01,
        0xffe1 => 0x2a,
        0xffe2 => 0x36,
        0xffe3 | 0xffe4 => 0x1d,
        0xffe9 | 0xffea => 0x38,
        _ => return None,
    })
}
