use std::env;

pub fn get_path(name: &str) -> String {
    String::from(
        env::current_dir()
            .unwrap()
            .join(name)
            .to_str()
            .unwrap(),
    )
}
