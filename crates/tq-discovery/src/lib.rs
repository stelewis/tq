pub(crate) mod error;
pub(crate) mod filesystem;
pub(crate) mod index;

pub use error::DiscoveryError;
pub use filesystem::build_analysis_index;
pub use index::AnalysisIndex;
