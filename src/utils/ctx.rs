//! Ctx is the context object passed through any IPC calls.
//! It can be queried to get the necessary states/services to perform any steps of a request.
//!
//! Notes:
//!     - Simple implementation for now.
//!     - For cloud applications, this will be used for authorization.
//!     - Eventually, this will also be used for "full context" logging/tracing or even performance tracing.
//!     - For a single user, desktop application, this object is much simpler as authorization and logging requirements are much reduced.

use crate::app::{ComponentType, Mode};
use crate::datastore::ModelStore;
use crate::utils::AppConfiguration;
use std::sync::Arc;

use super::Args;

#[derive(Clone)]
//Struct changed to removed app handle, and use Data instead
pub struct Ctx {
    store: Arc<ModelStore>,
    config: AppConfiguration,
    args: Args,
    pub mode: Mode,
    active_components: Vec<ComponentType>,
    pub auth: bool,
}
impl Ctx {
    pub fn new(store: Arc<ModelStore>, appconfig: AppConfiguration, args: Args) -> Self {
        Ctx {
            store: store.clone(),
            config: appconfig.clone(),
            args: args.clone(),
            mode: Mode::default(),
            active_components: Vec::new(),
            auth: false,
        }
    }

    pub fn get_model_manager(&self) -> Arc<ModelStore> {
        self.store.clone()
    }

    pub fn get_config(&self) -> AppConfiguration {
        self.config.clone()
    }
}
