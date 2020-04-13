use std::path::Path;

const IGNORE_ATTRIBUTE_KEY: &str = "user.com.dropbox.ignored";
const IGNORE_ATTRIBUTE_VALUE_IGNORED: [u8; 1] = [b'1'];

pub struct Dropbox {}

impl Dropbox {
    pub fn new() -> Result<Self, String> {
        Ok(Dropbox {})
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        match xattr::get(path, IGNORE_ATTRIBUTE_KEY) {
            Ok(attribute) => {
                if let Some(bytes) = attribute {
                    if bytes.eq(&IGNORE_ATTRIBUTE_VALUE_IGNORED) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn ignore(&self, path: &Path) -> bool {
        match xattr::set(path, IGNORE_ATTRIBUTE_KEY, &IGNORE_ATTRIBUTE_VALUE_IGNORED) {
            Ok(_) => true,
            Err(e) => {
                println!("Failed ignoring {:?} due {:?}", path, e);
                false
            }
        }
    }
}
