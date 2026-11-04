//! Parsers for each external source. They only read bytes: fetching and
//! caching live in `collect`.

pub mod cve_id;
pub mod kev;
pub mod poc;
