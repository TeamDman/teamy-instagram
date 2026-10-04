use super::CatalogError;
use chrono::NaiveDate;
use facet::Facet;
use std::fmt;

/// A validated calendar date found in a service ZIP filename, without time-zone
/// or time-of-day assumptions.
#[derive(Facet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[facet(json::proxy = String)]
pub struct ExportDate(String);

impl ExportDate {
    /// # Errors
    /// Returns a fixed code unless the input is a valid YYYY-MM-DD date.
    pub fn parse(value: &str) -> Result<Self, CatalogError> {
        if value.len() != 10
            || !value.bytes().enumerate().all(|(index, byte)| {
                if matches!(index, 4 | 7) {
                    byte == b'-'
                } else {
                    byte.is_ascii_digit()
                }
            })
            || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err()
        {
            return Err(CatalogError::InvalidExportDate);
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn from_filename(name: &str) -> Option<Self> {
        let stem = name.get(..name.len().checked_sub(4)?)?;
        let body = stem.get(10..)?;
        let mut dates = body.char_indices().filter_map(|(index, _)| {
            let value = body.get(index..index.checked_add(10)?)?;
            let before = index == 0 || body.as_bytes().get(index - 1) == Some(&b'-');
            let after = index + 10 == body.len() || body.as_bytes().get(index + 10) == Some(&b'-');
            (before && after).then(|| Self::parse(value).ok()).flatten()
        });
        let date = dates.next()?;
        if dates.next().is_some() {
            None
        } else {
            Some(date)
        }
    }
}

impl TryFrom<String> for ExportDate {
    type Error = CatalogError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<&ExportDate> for String {
    fn from(value: &ExportDate) -> Self {
        value.0.clone()
    }
}

impl fmt::Display for ExportDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
