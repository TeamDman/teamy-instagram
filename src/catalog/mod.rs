//! Local archive-root configuration and bounded filesystem inventory.
//!
//! Paths are private runtime output. Failures contain fixed codes; filesystem
//! error values and personal paths are never included in diagnostics.
mod archive_record;
mod catalog_error;
mod date_basis;
mod export_date;
mod inventory;
mod inventory_limits;
mod inventory_report;
mod latest_report;
mod roots_configuration;
mod roots_store;

pub use archive_record::ArchiveRecord;
pub use catalog_error::CatalogError;
pub use date_basis::DateBasis;
pub use export_date::ExportDate;
pub use inventory::inventory_roots;
pub use inventory_limits::InventoryLimits;
pub use inventory_report::InventoryReport;
pub use latest_report::LatestReport;
pub use latest_report::select_latest;
pub use roots_configuration::RootsConfiguration;
pub use roots_store::RootsStore;
