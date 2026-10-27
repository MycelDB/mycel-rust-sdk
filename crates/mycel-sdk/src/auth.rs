use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

use prost_types::Timestamp;
use tokio::sync::Mutex;
use tonic::{metadata::MetadataValue, service::Interceptor, Code, Request, Status};

use crate::config::Config;

const AUTHORIZATION_HEADER: &str = "authorization";
pub const DEFAULT_REFRESH_BEFORE: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct TokenSource {
    state: Arc<RwLock<TokenState>>,
}

#[derive(Debug, Clone)]
struct TokenState {
    access_token: String,
    access_token_expire_time: Option<SystemTime>,
    refresh_token: String,
    refresh_before: Duration,
    generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthGeneration(u64);

impl Default for TokenSource {
    fn default() -> Self {
        Self::new("")
    }
}

impl TokenSource {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            state: Arc::new(RwLock::new(TokenState {
                access_token: token.into().trim().to_string(),
                access_token_expire_time: None,
                refresh_token: String::new(),
                refresh_before: DEFAULT_REFRESH_BEFORE,
                generation: 0,
            })),
        }
    }

    pub fn from_config(cfg: &Config) -> Self {
        let tokens = Self::new(cfg.access_token.clone());
        tokens.set_access_token(cfg.access_token.clone(), cfg.access_token_expire_time);
        tokens.set_refresh_token(cfg.refresh_token.clone());
        tokens.set_refresh_before(cfg.refresh_before.unwrap_or(DEFAULT_REFRESH_BEFORE));
        tokens
    }

    pub fn set(&self, token: impl Into<String>) {
        self.set_access_token(token, None);
    }

    pub fn set_access_token(&self, token: impl Into<String>, expire_time: Option<SystemTime>) {
        let mut state = self.state.write().expect("token lock poisoned");
        state.access_token = token.into().trim().to_string();
        state.access_token_expire_time = expire_time;
        state.generation = state.generation.wrapping_add(1);
    }

    pub fn set_refresh_token(&self, token: impl Into<String>) {
        self.state
            .write()
            .expect("token lock poisoned")
            .refresh_token = token.into().trim().to_string();
    }

    pub fn set_tokens(
        &self,
        access_token: impl Into<String>,
        expire_time: Option<SystemTime>,
        refresh_token: impl Into<String>,
    ) {
        let mut state = self.state.write().expect("token lock poisoned");
        state.access_token = access_token.into().trim().to_string();
        state.access_token_expire_time = expire_time;
        let refresh_token = refresh_token.into().trim().to_string();
        if !refresh_token.is_empty() {
            state.refresh_token = refresh_token;
        }
        state.generation = state.generation.wrapping_add(1);
    }

    pub fn clear(&self) {
        let mut state = self.state.write().expect("token lock poisoned");
        state.access_token.clear();
        state.access_token_expire_time = None;
        state.refresh_token.clear();
        state.generation = state.generation.wrapping_add(1);
    }

    pub fn set_refresh_before(&self, refresh_before: Duration) {
        self.state
            .write()
            .expect("token lock poisoned")
            .refresh_before = if refresh_before.is_zero() {
            DEFAULT_REFRESH_BEFORE
        } else {
            refresh_before
        };
    }

    pub fn token(&self) -> String {
        self.state
            .read()
            .expect("token lock poisoned")
            .access_token
            .clone()
    }

    pub fn refresh_token(&self) -> String {
        self.state
            .read()
            .expect("token lock poisoned")
            .refresh_token
            .clone()
    }

    pub fn access_token_expire_time(&self) -> Option<SystemTime> {
        self.state
            .read()
            .expect("token lock poisoned")
            .access_token_expire_time
    }

    pub fn can_refresh(&self) -> bool {
        !self.refresh_token().is_empty()
    }

    pub fn generation(&self) -> AuthGeneration {
        AuthGeneration(self.state.read().expect("token lock poisoned").generation)
    }

    pub fn is_generation(&self, generation: AuthGeneration) -> bool {
        self.generation() == generation
    }

    pub fn needs_refresh(&self, now: SystemTime) -> bool {
        let state = self.state.read().expect("token lock poisoned");
        if state.refresh_token.is_empty() {
            return false;
        }
        let Some(expire_time) = state.access_token_expire_time else {
            return false;
        };
        match expire_time.checked_sub(state.refresh_before) {
            Some(refresh_at) => now >= refresh_at,
            None => true,
        }
    }

    pub fn is_expired(&self, now: SystemTime) -> bool {
        self.access_token_expire_time()
            .map(|expire_time| now >= expire_time)
            .unwrap_or(false)
    }

    pub fn interceptor(&self) -> AuthInterceptor {
        AuthInterceptor {
            tokens: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    tokens: TokenSource,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = self.tokens.token();
        if !token.is_empty() {
            let value = format!("Bearer {token}")
                .parse::<MetadataValue<_>>()
                .map_err(|_| Status::unauthenticated("invalid access token metadata"))?;
            request.metadata_mut().insert(AUTHORIZATION_HEADER, value);
        }
        Ok(request)
    }
}

pub fn is_expired_unauthenticated(status: &Status) -> bool {
    status.code() == Code::Unauthenticated
        && status.message().to_ascii_lowercase().contains("expired")
}

pub fn timestamp_to_system_time(timestamp: Option<Timestamp>) -> Option<SystemTime> {
    let timestamp = timestamp?;
    if timestamp.seconds < 0 || timestamp.nanos < 0 {
        return None;
    }
    Some(SystemTime::UNIX_EPOCH + Duration::new(timestamp.seconds as u64, timestamp.nanos as u32))
}

pub(crate) async fn refresh_generation_once<F, Fut, E>(
    tokens: TokenSource,
    refresh_lock: Arc<Mutex<()>>,
    rejected_generation: AuthGeneration,
    refresh: F,
) -> std::result::Result<(), E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<(), E>>,
{
    let _guard = refresh_lock.lock().await;
    if !tokens.is_generation(rejected_generation) {
        return Ok(());
    }
    refresh().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_source_trims_and_updates() {
        let tokens = TokenSource::new(" abc ");
        assert_eq!(tokens.token(), "abc");
        tokens.set(" def ");
        assert_eq!(tokens.token(), "def");
    }

    #[test]
    fn token_source_tracks_refresh_state() {
        let tokens = TokenSource::new("access");
        let expire_time = SystemTime::now() + Duration::from_secs(10);
        tokens.set_tokens("access-2", Some(expire_time), "refresh");
        assert_eq!(tokens.token(), "access-2");
        assert_eq!(tokens.refresh_token(), "refresh");
        assert!(tokens.needs_refresh(SystemTime::now()));
        assert!(!tokens.is_expired(SystemTime::now()));
        tokens.clear();
        assert_eq!(tokens.token(), "");
        assert_eq!(tokens.refresh_token(), "");
    }

    #[test]
    fn token_source_loads_config_refresh_state() {
        let expire_time = SystemTime::now() + Duration::from_secs(120);
        let cfg = Config {
            access_token: " access ".into(),
            access_token_expire_time: Some(expire_time),
            refresh_token: " refresh ".into(),
            refresh_before: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        let tokens = TokenSource::from_config(&cfg);
        assert_eq!(tokens.token(), "access");
        assert_eq!(tokens.refresh_token(), "refresh");
        assert_eq!(tokens.access_token_expire_time(), Some(expire_time));
        assert!(!tokens.needs_refresh(SystemTime::now()));
    }

    #[test]
    fn cloned_token_sources_publish_rotations_and_clear_atomically() {
        let tokens = TokenSource::new("access-1");
        tokens.set_refresh_token("refresh-1");
        let shared = tokens.clone();
        let rejected_generation = tokens.generation();

        shared.set_tokens("access-2", None, "refresh-2");

        assert_eq!(tokens.token(), "access-2");
        assert_eq!(tokens.refresh_token(), "refresh-2");
        assert!(!tokens.is_generation(rejected_generation));

        shared.clear();
        assert_eq!(tokens.token(), "");
        assert_eq!(tokens.refresh_token(), "");
    }

    #[test]
    fn expired_unauthenticated_matches_daemon_message() {
        assert!(is_expired_unauthenticated(&Status::unauthenticated(
            "authorization token is expired"
        )));
        assert!(!is_expired_unauthenticated(&Status::unauthenticated(
            "bad credentials"
        )));
    }

    #[tokio::test]
    async fn concurrent_recovery_refreshes_one_shared_generation_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use tokio::sync::Barrier;

        let tokens = TokenSource::new("access-1");
        tokens.set_refresh_token("refresh-1");
        let generation = tokens.generation();
        let refresh_lock = Arc::new(Mutex::new(()));
        let attempts = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(3));
        let mut tasks = Vec::new();

        for _ in 0..2 {
            let task_tokens = tokens.clone();
            let update_tokens = tokens.clone();
            let task_lock = refresh_lock.clone();
            let task_attempts = attempts.clone();
            let task_barrier = barrier.clone();
            tasks.push(tokio::spawn(async move {
                task_barrier.wait().await;
                refresh_generation_once(task_tokens, task_lock, generation, || async move {
                    task_attempts.fetch_add(1, Ordering::SeqCst);
                    update_tokens.set_tokens("access-2", None, "refresh-2");
                    Ok::<(), ()>(())
                })
                .await
            }));
        }

        barrier.wait().await;
        for task in tasks {
            task.await.expect("refresh task").expect("refresh result");
        }

        assert_eq!(attempts.load(Ordering::SeqCst), 1);
        assert_eq!(tokens.token(), "access-2");
        assert_eq!(tokens.refresh_token(), "refresh-2");
    }
}
