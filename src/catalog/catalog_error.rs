use std::fmt;

/// Fixed diagnostic codes without filesystem paths or underlying error values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogError {
    AppDataUnavailable,
    InvalidRoot,
    InvalidPath,
    InvalidExportDate,
    DateBasisRequired,
    ConfigurationReadFailed,
    ConfigurationInvalid,
    ConfigurationTooLarge,
    ConfigurationBusy,
    ConfigurationWriteFailed,
    RootLimitReached,
    InvalidInventoryLimits,
    Cancelled,
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AppDataUnavailable => "catalog_appdata_unavailable",
            Self::InvalidRoot => "catalog_invalid_root",
            Self::InvalidPath => "catalog_invalid_path",
            Self::InvalidExportDate => "catalog_invalid_export_date",
            Self::DateBasisRequired => "catalog_date_basis_required",
            Self::ConfigurationReadFailed => "catalog_configuration_read_failed",
            Self::ConfigurationInvalid => "catalog_configuration_invalid",
            Self::ConfigurationTooLarge => "catalog_configuration_too_large",
            Self::ConfigurationBusy => "catalog_configuration_busy",
            Self::ConfigurationWriteFailed => "catalog_configuration_write_failed",
            Self::RootLimitReached => "catalog_root_limit_reached",
            Self::InvalidInventoryLimits => "catalog_invalid_inventory_limits",
            Self::Cancelled => "catalog_cancelled",
        })
    }
}

impl std::error::Error for CatalogError {}
