//! Forwardings on an already authenticated ControlMaster. Loading definitions never starts one.
use crate::{config::Tunnel, ssh::SshConnection};
use std::{
    net::IpAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Local,
    Remote,
    Socks,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub kind: Kind,
    pub bind: IpAddr,
    pub port: u16,
    pub host: String,
    pub destination_port: u16,
}
impl Spec {
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Puerto de escucha entre 1 y 65535".into());
        }
        if self.kind != Kind::Socks
            && (self.destination_port == 0
                || self.host.is_empty()
                || self.host.len() > 255
                || !(self.host.parse::<IpAddr>().is_ok()
                    || self
                        .host
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c)))
                || self.host.starts_with('-'))
        {
            return Err("Destino: DNS/IPv4/IPv6 literal y puerto entre 1 y 65535".into());
        }
        Ok(())
    }
    pub fn arguments(&self) -> Result<Vec<String>, String> {
        self.validate()?;
        let bind = address(&self.bind.to_string());
        let rule = if self.kind == Kind::Socks {
            format!("{bind}:{}", self.port)
        } else {
            format!(
                "{bind}:{}:{}:{}",
                self.port,
                address(&self.host),
                self.destination_port
            )
        };
        Ok(vec![
            match self.kind {
                Kind::Local => "-L",
                Kind::Remote => "-R",
                Kind::Socks => "-D",
            }
            .into(),
            rule,
        ])
    }
    pub fn from_definition(d: &Tunnel) -> Result<Self, String> {
        let s = Self {
            kind: match d.kind.as_str() {
                "local" => Kind::Local,
                "remote" => Kind::Remote,
                "socks" => Kind::Socks,
                _ => return Err("Tipo de túnel no compatible".into()),
            },
            bind: d
                .bind_host
                .parse()
                .map_err(|_| "Escucha: dirección IP literal")?,
            port: d.bind_port,
            host: d.destination_host.clone(),
            destination_port: d.destination_port,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn definition(&self, profile_id: String, name: String) -> Tunnel {
        Tunnel {
            id: uuid::Uuid::new_v4().to_string(),
            profile_id,
            name,
            kind: match self.kind {
                Kind::Local => "local",
                Kind::Remote => "remote",
                Kind::Socks => "socks",
            }
            .into(),
            bind_host: self.bind.to_string(),
            bind_port: self.port,
            destination_host: self.host.clone(),
            destination_port: self.destination_port,
        }
    }
    pub fn description(&self) -> String {
        if self.kind == Kind::Socks {
            format!(
                "SOCKS4/5 local · {}:{}",
                address(&self.bind.to_string()),
                self.port
            )
        } else {
            format!(
                "{} · {}:{} → {}:{}",
                if self.kind == Kind::Local {
                    "Local"
                } else {
                    "Remoto"
                },
                address(&self.bind.to_string()),
                self.port,
                address(&self.host),
                self.destination_port
            )
        }
    }
}
fn address(value: &str) -> String {
    if value.contains(':') {
        format!("[{value}]")
    } else {
        value.into()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Starting,
    Active,
    Stopping,
    Stopped,
    Failed,
    Unknown,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub status: Status,
    pub detail: String,
}
pub struct Handle {
    pub spec: Spec,
    pub connection: Arc<SshConnection>,
    record: Arc<Mutex<Snapshot>>,
    cancel: Arc<AtomicBool>,
}
impl Handle {
    pub fn start(spec: Spec, connection: Arc<SshConnection>) -> Result<Arc<Self>, String> {
        let args = spec.arguments()?;
        if connection.closed.load(Ordering::Relaxed) {
            return Err("La sesión SSH está desconectada".into());
        }
        let record = Arc::new(Mutex::new(Snapshot {
            status: Status::Starting,
            detail: "Solicitando apertura al ControlMaster existente…".into(),
        }));
        let cancel = Arc::new(AtomicBool::new(false));
        let h = Arc::new(Self {
            spec,
            connection: connection.clone(),
            record: record.clone(),
            cancel: cancel.clone(),
        });
        crate::runtime::runtime().spawn(async move{
   let set=|status,detail:String|{*record.lock().unwrap_or_else(|e|e.into_inner())=Snapshot{status,detail};};
   if cancel.load(Ordering::Relaxed){set(Status::Stopped,"Cancelado antes de activar".into());return;}
   if let Err(e)=connection.forward_control("check",&[]).await{set(Status::Failed,format!("No se activó: {e}"));return;}
   if cancel.load(Ordering::Relaxed){set(Status::Stopped,"Cancelado antes de activar".into());return;}
   if let Err(error)=connection.forward_control("forward",&args).await{
    set(Status::Unknown,format!("Apertura no confirmada: {error}. Revisa el servidor antes de repetir; no se cancela un posible reenvío ajeno automáticamente."));return;
   }
   set(Status::Active,"Apertura confirmada por SSH; no acredita disponibilidad del servicio de destino. En remoto, GatewayPorts determina la escucha efectiva.".into());
   loop{
    if cancel.load(Ordering::Relaxed){set(Status::Stopping,"Cancelando el reenvío…".into());match connection.forward_control("cancel",&args).await{Ok(_)=>set(Status::Stopped,"Cancelación confirmada por SSH".into()),Err(e)=>set(Status::Unknown,format!("Cancelación no confirmada: {e}. Comprueba el servidor."))}return;}
    tokio::time::sleep(Duration::from_secs(2)).await;
    if cancel.load(Ordering::Relaxed){continue;}
    if connection.closed.load(Ordering::Relaxed)||connection.forward_control("check",&[]).await.is_err(){set(Status::Unknown,"Se perdió la sesión; escucha no confirmada. No se reconecta ni se reactiva automáticamente.".into());return;}
   }
  });
        Ok(h)
    }
    pub fn snapshot(&self) -> Snapshot {
        self.record
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
        let mut record = self.record.lock().unwrap_or_else(|e| e.into_inner());
        if record.status == Status::Unknown {
            record.status = Status::Stopping;
            record.detail = "Cancelación explícita de la regla no confirmada…".into();
            let record = self.record.clone();
            let connection = self.connection.clone();
            let args = self.spec.arguments().expect("validated tunnel");
            crate::runtime::runtime().spawn(async move {
                let result = connection.forward_control("cancel", &args).await;
                let snapshot = match result {
                    Ok(_) => Snapshot {
                        status: Status::Stopped,
                        detail: "Cancelación confirmada por SSH".into(),
                    },
                    Err(e) => Snapshot {
                        status: Status::Unknown,
                        detail: format!("Cancelación no confirmada: {e}"),
                    },
                };
                *record.lock().unwrap_or_else(|e| e.into_inner()) = snapshot;
            });
        }
    }
    pub fn busy(&self) -> bool {
        matches!(
            self.snapshot().status,
            Status::Starting | Status::Active | Status::Stopping
        )
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_rules_and_roundtrips_definitions() {
        let mut s = Spec {
            kind: Kind::Local,
            bind: "127.0.0.1".parse().unwrap(),
            port: 8080,
            host: "::1".into(),
            destination_port: 80,
        };
        assert_eq!(
            s.arguments().unwrap(),
            vec!["-L", "127.0.0.1:8080:[::1]:80"]
        );
        let d = s.definition("profile".into(), "name".into());
        assert_eq!(Spec::from_definition(&d).unwrap(), s);
        s.kind = Kind::Socks;
        assert_eq!(s.arguments().unwrap(), vec!["-D", "127.0.0.1:8080"]);
        s.port = 0;
        assert!(s.arguments().is_err());
    }
    #[test]
    fn rejects_option_and_forwarding_injection() {
        for host in ["-option", "host:22:else", "a b", "$(touch x)", "host;id"] {
            assert!(
                Spec {
                    kind: Kind::Remote,
                    bind: "::1".parse().unwrap(),
                    port: 8000,
                    host: host.into(),
                    destination_port: 80
                }
                .validate()
                .is_err()
            );
        }
    }
}
