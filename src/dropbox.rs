use std::path::Path;
#[cfg(windows)]
use std::ptr;

use log::warn;

#[cfg(windows)]
use widestring::U16CString;
#[cfg(windows)]
use winapi::{
    shared::minwindef::{DWORD, FALSE, LPCVOID, LPVOID, TRUE},
    um::{
        errhandlingapi::GetLastError,
        fileapi::{CreateFile2, ReadFile, WriteFile, CREATE_ALWAYS, OPEN_EXISTING},
        handleapi::CloseHandle,
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
};

#[cfg(unix)]
const IGNORE_ATTRIBUTE_KEY: &str = "user.com.dropbox.ignored";
#[cfg(windows)]
const IGNORE_ATTRIBUTE_KEY: &str = "com.dropbox.ignored";
const IGNORE_ATTRIBUTE_VALUE_IGNORED: [u8; 1] = [b'1'];

pub struct Dropbox {}

impl Dropbox {
    pub fn new() -> Result<Self, String> {
        Ok(Dropbox {})
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
    pub fn ignore(&self, path: &Path) -> bool {
        match xattr::set(path, IGNORE_ATTRIBUTE_KEY, &IGNORE_ATTRIBUTE_VALUE_IGNORED) {
            Ok(_) => true,
            Err(e) => {
                warn!("Failed ignoring {:?} due {:?}", path, e);
                false
            }
        }
    }

    #[cfg(windows)]
    pub fn is_ignored(&self, path: &Path) -> bool {
        let attribute = xattr_get(path, IGNORE_ATTRIBUTE_KEY);
        if let Some(bytes) = attribute {
            if bytes.eq(&IGNORE_ATTRIBUTE_VALUE_IGNORED) {
                return true;
            }
            return false;
        }
        false
    }

    #[cfg(windows)]
    pub fn ignore(&self, path: &Path) -> bool {
        xattr_set(path, IGNORE_ATTRIBUTE_KEY, &IGNORE_ATTRIBUTE_VALUE_IGNORED)
    }
}

#[cfg(windows)]
fn xattr_get(path: &Path, attribute_name: &str) -> Option<Vec<u8>> {
    unsafe {
        let stream_path = format!("{}:{}", path.to_str().unwrap(), attribute_name);
        let winapi_path = U16CString::from_str_unchecked(&stream_path);

        let handle = CreateFile2(
            winapi_path.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ,
            OPEN_EXISTING,
            ptr::null_mut(),
        );

        let error = GetLastError();
        if error != 0 {
            return None;
        }

        let mut data = [0u8; 2];
        let mut len = 0;
        let error = ReadFile(
            handle,
            data.as_mut_ptr() as LPVOID,
            data.len() as u32,
            &mut len,
            ptr::null_mut(),
        );

        if error != TRUE {
            let last_error = GetLastError();
            warn!(
                "Failed reading stream data from {:?} with {}",
                stream_path, last_error
            );
            CloseHandle(handle);
            return None;
        }

        CloseHandle(handle);
        let mut attribute_data = data.to_vec();
        attribute_data.truncate(len as usize);

        Some(attribute_data)
    }
}

#[cfg(windows)]
fn xattr_set(path: &Path, attribute_name: &str, attribute_value: &[u8]) -> bool {
    unsafe {
        let stream_path = format!("{}:{}", path.to_str().unwrap(), attribute_name);
        let winapi_path = U16CString::from_str_unchecked(&stream_path);

        let handle = CreateFile2(
            winapi_path.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_WRITE,
            CREATE_ALWAYS,
            ptr::null_mut(),
        );

        let last_error = GetLastError();
        if last_error != 0 {
            warn!(
                "Failed writing stream data to {:?} with {}",
                stream_path, last_error
            );
            return false;
        }

        let mut bytes_written = 0;
        let success = WriteFile(
            handle,
            attribute_value.as_ptr() as LPCVOID,
            attribute_value.len() as DWORD,
            &mut bytes_written,
            ptr::null_mut(),
        );

        if success == FALSE {
            let last_error = GetLastError();
            warn!(
                "Failed writing stream data to {:?} with {}",
                stream_path, last_error
            );
            CloseHandle(handle);
            return false;
        }

        CloseHandle(handle);
        true
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
    #[cfg(unix)]
    const TEST_IGNORE_ATTRIBUTE_KEY: &str = "user.com.dropbox.ignored";
    #[cfg(windows)]
    const TEST_IGNORE_ATTRIBUTE_KEY: &str = "com.dropbox.ignored";
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

    fn user_cache_directory_path() -> PathBuf {
        // we can't use tmpfs on linux as it doesn't
        // support extended file system attributes
        let base_dirs = BaseDirs::new().unwrap();
        let cache_dir = base_dirs.cache_dir();

        // create in case it's not present
        fs::create_dir_all(cache_dir).unwrap();

        cache_dir.into()
    }

    #[cfg(unix)]
    fn arrange_ignored_attribute(path: &Path) {
        xattr::set(
            path,
            TEST_IGNORE_ATTRIBUTE_KEY,
            &TEST_IGNORE_ATTRIBUTE_VALUE_IGNORED,
        )
        .unwrap();
    }

    #[cfg(windows)]
    fn arrange_ignored_attribute(path: &Path) {
        super::xattr_set(
            path,
            TEST_IGNORE_ATTRIBUTE_KEY,
            &TEST_IGNORE_ATTRIBUTE_VALUE_IGNORED,
        );
    }
}
