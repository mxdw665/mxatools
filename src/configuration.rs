use crate::{environment::*, tools::json::parse_config};

#[derive(Debug)]
pub struct Configuration {
    pub distribution: String,
    pub verisons: String,
    pub key: String,
    pub value: String,
}

impl Configuration {
    fn change_contenttext(self, path: &str, key: &str) {
        use std::{
            fs::File,
            io::{Read, Write},
        };

        let file = File::open(path);
        let mut contenttext = String::new();
        let _ = file
            .unwrap_or_else(|_| panic!("{} read failed", path))
            .read_to_string(&mut contenttext);
        let new_contenttext = contenttext.replace(key, &self.value);
        let file = File::create(path);
        let _ = file
            .unwrap_or_else(|_| panic!("{} write failed", path))
            .write_all(new_contenttext.as_bytes());
    }
    pub fn parse(self) {
        let json_path = format!(
            "{}/{}-{}/usr/share/at/config.json",
            CONTAINER_PATH, self.distribution, self.verisons
        );
        let key_value = parse_config(&json_path, &self.key)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} not found in config", &self.key),
                )
            })
            .map(|s| s.trim_matches('"').to_string())
            .expect("Failed to process language configuration");
        self.change_contenttext(&json_path, &key_value);
    }
}
