use std::env;

pub fn get_env(key: &str) -> String {
    env::var(key).expect(&format!("Environment variable {} not set", key))
}

pub fn get_parsed_env<T: std::str::FromStr>(key: &str) -> T {
    let val_str = get_env(key);
    val_str.parse::<T>().unwrap_or_else(|_| {
        panic!(
            "Environment variable {} is not a valid number: {}",
            key, val_str
        )
    })
}
