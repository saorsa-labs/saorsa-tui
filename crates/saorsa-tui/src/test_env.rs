//! Test-only helpers for process environment variables.
//!
//! Unit tests in this crate run concurrently and must not race on global
//! environment variables like `NO_COLOR`.

use std::ffi::OsString;
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvVarGuard {
    key: &'static str,
    old: Option<OsString>,
}

impl EnvVarGuard {
    fn new_set(key: &'static str, value: &str) -> Self {
        let old = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, old }
    }

    fn new_unset(key: &'static str) -> Self {
        let old = std::env::var_os(key);
        unsafe {
            std::env::remove_var(key);
        }
        Self { key, old }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.old.take() {
            Some(v) => unsafe {
                std::env::set_var(self.key, v);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

pub(crate) fn without_no_color<R>(f: impl FnOnce() -> R) -> R {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = match lock.lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    let _env = EnvVarGuard::new_unset("NO_COLOR");
    f()
}

pub(crate) fn with_no_color_set<R>(value: &str, f: impl FnOnce() -> R) -> R {
    let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = match lock.lock() {
        Ok(g) => g,
        Err(poison) => poison.into_inner(),
    };
    let _env = EnvVarGuard::new_set("NO_COLOR", value);
    f()
}
