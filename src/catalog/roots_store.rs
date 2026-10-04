use super::CatalogError;
use super::RootsConfiguration;
use directories_next::ProjectDirs;
use std::fmt;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

const MAX_CONFIGURATION_BYTES: u64 = 1024 * 1024;
const MAX_ROOTS: usize = 1024;
static TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Injectable settings location; the default lives in the application's local
/// appdata directory. Debug output intentionally excludes its private path.
#[derive(Clone)]
pub struct RootsStore {
    configuration_path: PathBuf,
}

impl fmt::Debug for RootsStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootsStore").finish_non_exhaustive()
    }
}

impl RootsStore {
    #[must_use]
    pub fn new(configuration_path: PathBuf) -> Self {
        Self { configuration_path }
    }

    /// # Errors
    /// Returns a fixed code if the platform appdata directory is unavailable.
    pub fn in_appdata() -> Result<Self, CatalogError> {
        let directories = ProjectDirs::from("com", "TeamDman", "teamy-instagram")
            .ok_or(CatalogError::AppDataUnavailable)?;
        Ok(Self::new(
            directories.data_local_dir().join("archive-roots.json"),
        ))
    }

    /// Read strictly without changing settings. A missing file means no roots.
    ///
    /// # Errors
    /// Returns a fixed code for unreadable, oversized, or invalid settings.
    pub fn list(&self) -> Result<RootsConfiguration, CatalogError> {
        let file = match File::open(&self.configuration_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RootsConfiguration::default());
            }
            Err(_error) => return Err(CatalogError::ConfigurationReadFailed),
        };
        let mut bytes = Vec::new();
        file.take(MAX_CONFIGURATION_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_error| CatalogError::ConfigurationReadFailed)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_CONFIGURATION_BYTES {
            return Err(CatalogError::ConfigurationTooLarge);
        }
        let configuration: RootsConfiguration = crate::models::parse_json(&bytes)
            .map_err(|_error| CatalogError::ConfigurationInvalid)?;
        if configuration.version != 1
            || configuration.roots.len() > MAX_ROOTS
            || configuration
                .roots
                .iter()
                .any(|root| !Path::new(root).is_absolute())
        {
            return Err(CatalogError::ConfigurationInvalid);
        }
        Ok(configuration)
    }

    /// Canonicalize an existing directory, deduplicate, and persist settings.
    ///
    /// # Errors
    /// Returns fixed codes for an invalid root or settings failure. Invalid
    /// existing settings are preserved; they are never replaced with defaults.
    pub fn add(&self, root: &Path) -> Result<RootsConfiguration, CatalogError> {
        let canonical = root
            .canonicalize()
            .map_err(|_error| CatalogError::InvalidRoot)?;
        if !canonical.is_dir() {
            return Err(CatalogError::InvalidRoot);
        }
        let value = canonical
            .to_str()
            .ok_or(CatalogError::InvalidPath)?
            .to_owned();
        let _lock = self.lock()?;
        let mut configuration = self.list()?;
        if !configuration.roots.contains(&value) {
            if configuration.roots.len() >= MAX_ROOTS {
                return Err(CatalogError::RootLimitReached);
            }
            configuration.roots.push(value);
            configuration.roots.sort();
            self.write(&configuration)?;
        }
        Ok(configuration)
    }

    /// Remove a canonical directory or its exact stored path if it no longer
    /// exists. An unregistered path leaves valid settings unchanged.
    ///
    /// # Errors
    /// Returns fixed codes for a non-UTF-8 path or settings failure.
    pub fn remove(&self, root: &Path) -> Result<RootsConfiguration, CatalogError> {
        let canonical = root.canonicalize().ok();
        let value = canonical
            .as_deref()
            .unwrap_or(root)
            .to_str()
            .ok_or(CatalogError::InvalidPath)?;
        let _lock = self.lock()?;
        let mut configuration = self.list()?;
        let original_length = configuration.roots.len();
        configuration.roots.retain(|stored| stored != value);
        if configuration.roots.len() != original_length {
            self.write(&configuration)?;
        }
        Ok(configuration)
    }

    fn lock(&self) -> Result<CleanupFile, CatalogError> {
        fs::create_dir_all(self.parent())
            .map_err(|_error| CatalogError::ConfigurationWriteFailed)?;
        let path = self.configuration_path.with_extension("lock");
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    CatalogError::ConfigurationBusy
                } else {
                    CatalogError::ConfigurationWriteFailed
                }
            })?;
        drop(file);
        Ok(CleanupFile(path))
    }

    fn parent(&self) -> &Path {
        self.configuration_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }

    fn write(&self, configuration: &RootsConfiguration) -> Result<(), CatalogError> {
        let bytes = facet_json::to_string_pretty(configuration)
            .map_err(|_error| CatalogError::ConfigurationWriteFailed)?;
        let sequence = TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = self
            .configuration_path
            .with_extension(format!("tmp-{}-{sequence}", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_error| CatalogError::ConfigurationWriteFailed)?;
        let _temporary = CleanupFile(path.clone());
        file.write_all(bytes.as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|_error| CatalogError::ConfigurationWriteFailed)?;
        drop(file);
        fs::rename(&path, &self.configuration_path)
            .map_err(|_error| CatalogError::ConfigurationWriteFailed)?;
        Ok(())
    }
}

#[derive(Debug)]
struct CleanupFile(PathBuf);

impl Drop for CleanupFile {
    fn drop(&mut self) {
        let _result = fs::remove_file(&self.0);
    }
}
