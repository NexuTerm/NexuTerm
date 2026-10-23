//! FTP credentials stored in Secret Service or in memory.
use nexuterm::config::Profile;
use std::collections::HashMap;
use zeroize::Zeroizing;

pub type Key = (String, String, u16, String);
pub fn key(profile: &Profile) -> Key {
    (
        profile.id.clone(),
        profile.host.clone(),
        profile.port,
        profile.username.clone(),
    )
}
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        Err("La contraseña FTP no puede contener NUL ni saltos de línea".into())
    } else {
        Ok(())
    }
}
fn schema() -> secret::Schema {
    secret::Schema::new(
        "io.nexuterm.FtpPassword",
        secret::SchemaFlags::NONE,
        ["reference", "host", "port", "username"]
            .map(|name| (name, secret::SchemaAttributeType::String))
            .into(),
    )
}
fn attributes<'a>(
    profile: &'a Profile,
    reference: &'a str,
    port: &'a str,
) -> HashMap<&'a str, &'a str> {
    [
        ("reference", reference),
        ("host", profile.host.as_str()),
        ("port", port),
        ("username", profile.username.as_str()),
    ]
    .into()
}
pub async fn store(profile: &Profile, password: &str) -> Result<String, String> {
    let reference = uuid::Uuid::new_v4().to_string();
    secret::password_store_future(Some(&schema()), attributes(profile, &reference, &profile.port.to_string()),
        Some(secret::COLLECTION_DEFAULT), &format!("NexuTerm FTP · {}", profile.host), password).await
        .map_err(|error| format!("No se pudo guardar en el llavero: {error}. Desmarca Guardar en el llavero para usarla solo durante esta ejecución."))?;
    Ok(reference)
}
pub async fn lookup(profile: &Profile) -> Result<Option<Zeroizing<String>>, String> {
    let Some(reference) = profile.secret_ref.as_deref() else {
        return Ok(None);
    };
    secret::password_lookup_future(
        Some(&schema()),
        attributes(profile, reference, &profile.port.to_string()),
    )
    .await
    .map(|password| password.map(|p| Zeroizing::new(p.to_string())))
    .map_err(|error| format!("No se pudo leer la contraseña del llavero: {error}"))
}
pub async fn forget(profile: &Profile) -> Result<(), String> {
    let Some(reference) = profile.secret_ref.as_deref() else {
        return Ok(());
    };
    secret::password_clear_future(
        Some(&schema()),
        attributes(profile, reference, &profile.port.to_string()),
    )
    .await
    .map_err(|error| format!("No se pudo eliminar la contraseña del llavero: {error}"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn credentials_are_bound_to_profile_and_destination() {
        let profile = Profile {
            id: "fixture".into(),
            host: "example.invalid".into(),
            username: "demo".into(),
            port: 21,
            ..Profile::default()
        };
        let mut changed = profile.clone();
        changed.port = 2121;
        assert_ne!(key(&profile), key(&changed));
        assert!(validate_password("fictitious \"\\ value").is_ok());
        assert!(validate_password("invalid\nvalue").is_err());
    }
}
