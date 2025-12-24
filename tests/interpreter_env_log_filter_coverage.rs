use mdhavers::Interpreter;

fn set_env(key: &str, value: Option<&str>) -> Option<String> {
    let prev = std::env::var(key).ok();
    match value {
        Some(v) => std::env::set_var(key, v),
        None => std::env::remove_var(key),
    }
    prev
}

#[test]
fn interpreter_new_reads_mdh_log_env_var_paths() {
    let prev_log = std::env::var("MDH_LOG").ok();
    let prev_level = std::env::var("MDH_LOG_LEVEL").ok();

    set_env("MDH_LOG_LEVEL", None);

    set_env("MDH_LOG", Some("mutter"));
    let _ = Interpreter::new();

    set_env("MDH_LOG", Some("definitely_not_a_level"));
    let _ = Interpreter::new();

    // Cover the fallback MDH_LOG_LEVEL branch when MDH_LOG is not set.
    set_env("MDH_LOG", None);
    set_env("MDH_LOG_LEVEL", Some("mutter"));
    let _ = Interpreter::new();

    set_env("MDH_LOG", prev_log.as_deref());
    set_env("MDH_LOG_LEVEL", prev_level.as_deref());
}
