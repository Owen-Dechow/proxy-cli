use crate::debug;
use json;
use std::io::{self, Read as _};

const DB_VERSION: u8 = 1;
const DB_NAME: &str = "proxy.db.json";

pub enum DBError {
    IOError(io::Error),
    JsonError(json::Error),
}

fn get_initial_db_json() -> String {
    let mut init = String::from(
        r#"
        {
            "db_version": {{DB_VERSION}},
            "cmds": {}
        }
    "#,
    );

    init = init.replace("{{DB_VERSION}}", &DB_VERSION.to_string());

    return init;
}

fn save_db(data: &String) -> Result<(), io::Error> {
    return std::fs::write(DB_NAME, data);
}

fn get_db_text() -> Result<String, io::Error> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(DB_NAME)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    if content.len() == 0 {
        content = get_initial_db_json();
        save_db(&content)?;
        debug::print(format!("New proxy database initalized: {}", DB_NAME));
    }

    return Ok(content);
}

pub fn get_db_content() -> Result<json::JsonValue, DBError> {
    let content = match get_db_text() {
        Ok(content) => content,
        Err(err) => {
            return Err(DBError::IOError(err));
        }
    };

    match json::parse(&content) {
        Ok(json) => {
            return Ok(json);
        }
        Err(err) => return Err(DBError::JsonError(err)),
    }
}

pub fn attempt_db_save(data: &String, msg: String) {
    match save_db(data) {
        Ok(_) => debug::print(msg),
        Err(err) => debug::print(format!("Database save failed: {}", err)),
    }
}
