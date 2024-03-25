use prbot_config::Config;
use prbot_database_interface::DbService;
use prbot_ghapi_interface::ApiService;
use prbot_lock_interface::LockService;

use crate::CoreModule;

pub struct CoreContext<'a> {
    pub config: &'a Config,
    pub core_module: &'a CoreModule,
    pub api_service: &'a (dyn ApiService + 'a),
    pub db_service: &'a (dyn DbService + 'a),
    pub lock_service: &'a (dyn LockService + 'a),
}

#[cfg(any(test, feature = "testkit"))]
pub(crate) mod tests {
    use prbot_config::Config;
    use prbot_database_memory::MemoryDb;
    use prbot_ghapi_interface::MockApiService;
    use prbot_lock_interface::MockLockService;

    use crate::{CoreContext, CoreModule};

    #[allow(dead_code)]
    pub struct CoreContextTest {
        pub config: Config,
        pub core_module: CoreModule,
        pub api_service: MockApiService,
        pub db_service: MemoryDb,
        pub lock_service: MockLockService,
    }

    impl CoreContextTest {
        #[allow(dead_code)]
        pub fn new() -> Self {
            Self {
                config: Config::from_env_no_version(),
                core_module: CoreModule::builder().build(),
                api_service: MockApiService::new(),
                db_service: MemoryDb::new(),
                lock_service: MockLockService::new(),
            }
        }

        #[allow(dead_code)]
        pub fn as_context(&self) -> CoreContext {
            CoreContext {
                config: &self.config,
                core_module: &self.core_module,
                api_service: &self.api_service,
                db_service: &self.db_service,
                lock_service: &self.lock_service,
            }
        }
    }
}
