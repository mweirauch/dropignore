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

#[cfg(test)]
mod tests {
    use super::Dropbox;
    use directories::BaseDirs;
    use rstest::rstest;
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::{Builder, NamedTempFile, TempDir};

    const TEST_ITEM_PREFIX: &str = "dropignore-testing";
    const TEST_IGNORE_ATTRIBUTE_KEY: &str = "user.com.dropbox.ignored";
    const TEST_IGNORE_ATTRIBUTE_VALUE_IGNORED: [u8; 1] = [b'1'];

    #[rstest(arrange_ignored, case::not_ignored(false), case::ignored(true))]
    fn is_ignored_directory(arrange_ignored: bool) {
        let temp = arrange_test_directory();
        let path = temp.path();

        if arrange_ignored {
            arrange_ignored_attribute(path);
        }

        let dropbox = Dropbox::new().unwrap();

        assert_eq!(arrange_ignored, dropbox.is_ignored(path));
    }

    #[rstest(arrange_ignored, case::not_ignored(false), case::ignored(true))]
    fn is_ignored_file(arrange_ignored: bool) {
        let temp = arrange_test_file();
        let path = temp.path();

        if arrange_ignored {
            arrange_ignored_attribute(path);
        }

        let dropbox = Dropbox::new().unwrap();

        assert_eq!(arrange_ignored, dropbox.is_ignored(path));
    }

    #[test]
    fn ignore_directory() {
        let temp = arrange_test_directory();
        let path = temp.path();

        let dropbox = Dropbox::new().unwrap();

        dropbox.ignore(path);

        assert!(dropbox.is_ignored(path));
    }

    #[test]
    fn ignore_file() {
        let temp = arrange_test_file();
        let path = temp.path();

        let dropbox = Dropbox::new().unwrap();

        dropbox.ignore(path);

        assert!(dropbox.is_ignored(path));
    }

    fn arrange_test_directory() -> TempDir {
        let mut builder = Builder::new();
        builder.prefix(TEST_ITEM_PREFIX);

        let temp = if cfg!(target_os = "linux") {
            let cache_dir = user_cache_directory_path();
            builder.tempdir_in(cache_dir)
        } else {
            builder.tempdir()
        };

        temp.unwrap()
    }

    fn arrange_test_file() -> NamedTempFile {
        let mut builder = Builder::new();
        builder.prefix(TEST_ITEM_PREFIX);

        let temp = if cfg!(target_os = "linux") {
            let cache_dir = user_cache_directory_path();
            builder.tempfile_in(cache_dir)
        } else {
            builder.tempfile()
        };

        temp.unwrap()
    }

    #[cfg(target_os = "linux")]
    fn user_cache_directory_path() -> PathBuf {
        // we can't use tmpfs on linux as it doesn't
        // support extended file system attributes
        let base_dirs = BaseDirs::new().unwrap();
        let cache_dir = base_dirs.cache_dir();

        // create in case it's not present
        fs::create_dir_all(cache_dir).unwrap();

        cache_dir.into()
    }

    fn arrange_ignored_attribute(path: &Path) {
        xattr::set(
            path,
            TEST_IGNORE_ATTRIBUTE_KEY,
            &TEST_IGNORE_ATTRIBUTE_VALUE_IGNORED,
        )
        .unwrap();
    }
}
