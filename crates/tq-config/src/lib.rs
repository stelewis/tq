mod error;
mod loader;
mod loader_materialize;
mod loader_parse;
mod model;
mod paths;

pub use error::ConfigError;
pub use loader::{resolve_tq_config, resolve_tq_config_with_user_config};
pub use model::{CliOverrides, DEFAULT_INIT_MODULES, TqConfig, TqTargetConfig};
pub use tq_core::{
    DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES, InitModulesMode, PackageName, QualifierStrategy,
    RelativePathBuf, RuleId, Severity, TargetName,
};
