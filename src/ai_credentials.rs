//! API keys are scoped to profile and endpoint, and never serialized with profiles.
use super::*;
use nexuterm::config::AiAgent;
use zeroize::Zeroizing;

fn schema() -> secret::Schema {
    secret::Schema::new(
        "io.nexuterm.AiApiKey",
        secret::SchemaFlags::NONE,
        ["reference", "profile", "endpoint"]
            .map(|name| (name, secret::SchemaAttributeType::String))
            .into(),
    )
}
fn attributes<'a>(
    agent: &'a AiAgent,
    reference: &'a str,
) -> std::collections::HashMap<&'a str, &'a str> {
    [
        ("reference", reference),
        ("profile", agent.id.as_str()),
        ("endpoint", agent.endpoint.trim_end_matches('/')),
    ]
    .into()
}
pub(super) async fn lookup(state: &State, agent: &AiAgent) -> Result<Zeroizing<String>, String> {
    if let Some(key) = state.ai_keys.borrow().get(&agent.credential_key()) {
        return Ok(key.clone());
    }
    let Some(reference) = agent.secret_ref.as_deref() else {
        return Ok(Zeroizing::new(String::new()));
    };
    secret::password_lookup_future(Some(&schema()), attributes(agent, reference))
        .await
        .map(|value| Zeroizing::new(value.map(|s| s.to_string()).unwrap_or_default()))
        .map_err(|_| "No se pudo leer la clave del llavero. Revisa que esté desbloqueado".into())
}
async fn store(agent: &AiAgent, key: &str) -> Result<String, String> {
    let reference = uuid::Uuid::new_v4().to_string();
    secret::password_store_future(
        Some(&schema()),
        attributes(agent, &reference),
        Some(secret::COLLECTION_DEFAULT),
        &format!("NexuTerm IA · {}", agent.name),
        key,
    )
    .await
    .map_err(
        |_| "No se pudo guardar la clave en el llavero. Puedes usarla solo durante esta ejecución",
    )?;
    Ok(reference)
}
pub(super) async fn forget(agent: &AiAgent) -> Result<(), String> {
    if let Some(reference) = agent.secret_ref.as_deref() {
        secret::password_clear_future(Some(&schema()), attributes(agent, reference))
            .await
            .map_err(|_| "No se pudo eliminar la clave anterior del llavero")?;
    }
    Ok(())
}

pub(super) async fn save(
    state: &State,
    mut agent: AiAgent,
    old: Option<AiAgent>,
    mut key: Zeroizing<String>,
    remember: bool,
    clear: bool,
) -> Result<(), String> {
    agent.validate()?;
    nexuterm::ai::validate_key(&key)?;
    let same_endpoint = old
        .as_ref()
        .is_some_and(|old| old.credential_key() == agent.credential_key());
    if key.is_empty() && !clear && same_endpoint {
        key = lookup(state, old.as_ref().unwrap()).await?;
    }
    agent.secret_ref = None;
    if !key.is_empty() && remember && !clear {
        agent.secret_ref = Some(store(&agent, &key).await?);
    }
    // Re-read after awaiting the keyring; other profiles may have changed.
    let mut config = state.config.borrow().clone();
    if let Some(index) = config.ai_agents.iter().position(|a| a.id == agent.id) {
        config.ai_agents[index] = agent.clone();
    } else if old.is_none() {
        config.ai_agents.push(agent.clone());
    } else {
        let _ = forget(&agent).await;
        return Err("El agente se eliminó mientras se guardaba".into());
    }
    if let Err(error) = config::save(&state.path, &config) {
        let _ = forget(&agent).await;
        return Err(error);
    }
    *state.config.borrow_mut() = config;
    state
        .ai_keys
        .borrow_mut()
        .retain(|(id, _), _| id != &agent.id);
    if !key.is_empty() && !clear {
        state
            .ai_keys
            .borrow_mut()
            .insert(agent.credential_key(), key);
    }
    if let Some(old) = old
        && let Err(error) = forget(&old).await
    {
        state.status.set_text(&error);
    }
    Ok(())
}

#[cfg(debug_assertions)]
pub(super) fn smoke(state: &Rc<State>, remember: bool) {
    let state = state.clone();
    glib::spawn_future_local(async move {
        let original = state.config.borrow().ai_agents[0].clone();
        let key = Zeroizing::new("fixture-api-key-never-serialized".to_owned());
        save(
            &state,
            original.clone(),
            Some(original),
            key.clone(),
            remember,
            false,
        )
        .await
        .unwrap();
        let saved = config::load(&state.path).unwrap().ai_agents[0].clone();
        assert_eq!(saved.secret_ref.is_some(), remember);
        assert!(
            !std::fs::read_to_string(&state.path)
                .unwrap()
                .contains(key.as_str())
        );
        assert_eq!(lookup(&state, &saved).await.unwrap().as_str(), key.as_str());
        state.ai_keys.borrow_mut().clear();
        let loaded = lookup(&state, &saved).await.unwrap();
        if remember {
            assert_eq!(loaded.as_str(), key.as_str());
        } else {
            assert!(loaded.is_empty());
        }
        let mut changed = saved.clone();
        changed.endpoint = "http://127.0.0.1:11435/v1".into();
        save(
            &state,
            changed,
            Some(saved.clone()),
            Zeroizing::new(String::new()),
            remember,
            false,
        )
        .await
        .unwrap();
        let changed = state.config.borrow().ai_agents[0].clone();
        assert!(changed.secret_ref.is_none());
        assert!(lookup(&state, &changed).await.unwrap().is_empty());
        // The former destination's persisted key must have been removed as well.
        assert!(lookup(&state, &saved).await.unwrap().is_empty());
        save(
            &state,
            changed.clone(),
            Some(changed),
            key.clone(),
            remember,
            false,
        )
        .await
        .unwrap();
        let changed = state.config.borrow().ai_agents[0].clone();
        save(
            &state,
            changed.clone(),
            Some(changed.clone()),
            Zeroizing::new(String::new()),
            remember,
            true,
        )
        .await
        .unwrap();
        assert!(lookup(&state, &changed).await.unwrap().is_empty());
        assert!(
            !std::fs::read_to_string(&state.path)
                .unwrap()
                .contains(key.as_str())
        );
        eprintln!("NEXUTERM_SMOKE_AI_CREDENTIALS_OK keyring={remember}");
    });
}
