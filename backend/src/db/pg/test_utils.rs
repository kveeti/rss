use sqlx::{PgPool, migrate};
use std::sync::Arc;

use crate::db::Data;

use super::PgData;

/// Test database wrapper that creates an isolated database per test.
/// The database is automatically dropped when TestDb goes out of scope.
pub struct TestDb {
    pub data: Data,
    db_name: String,
    test_pool: PgPool, // Keep reference to close before dropping DB
}

impl TestDb {
    /// Creates a new isolated test database.
    ///
    /// 1. Connects to the `postgres` database (admin connection)
    /// 2. Creates a unique test database named `test_{ulid}`
    /// 3. Runs all migrations on the new database
    /// 4. Returns a TestDb with `data` ready for use
    pub async fn new() -> Self {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

        // Parse the URL to get the base connection string (without database name)
        let base_url = database_url
            .rsplit_once('/')
            .map(|(base, _)| base)
            .unwrap_or(&database_url);

        // Connect to the default postgres database for admin operations
        let admin_url = format!("{}/postgres", base_url);
        let admin_pool = PgPool::connect(&admin_url)
            .await
            .expect("Failed to connect to postgres database");

        // Create a unique test database
        let db_name = format!("test_{}", ulid::Ulid::new().to_string().to_lowercase());

        sqlx::query(&format!("CREATE DATABASE {}", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");

        // Connect to the new test database
        let test_url = format!("{}/{}", base_url, db_name);
        let test_pool = PgPool::connect(&test_url)
            .await
            .expect("Failed to connect to test database");

        // Run migrations
        migrate!("./src/db/pg/migrations")
            .run(&test_pool)
            .await
            .expect("Failed to run migrations");

        let data: Data = Arc::new(PgData::from_pool(test_pool.clone()));

        TestDb {
            data,
            db_name,
            test_pool,
        }
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let db_name = self.db_name.clone();
        let test_pool = self.test_pool.clone();

        // Get admin URL for fresh connection
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let base_url = database_url
            .rsplit_once('/')
            .map(|(base, _)| base)
            .unwrap_or(&database_url);
        let admin_url = format!("{}/postgres", base_url);

        // Spawn cleanup in background thread (fire and forget)
        // Don't block on .join() - this prevents hangs when running parallel tests
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create cleanup runtime");

            rt.block_on(async {
                // Close our connection to the test database first
                test_pool.close().await;

                // Create fresh connection for cleanup
                if let Ok(admin_conn) = PgPool::connect(&admin_url).await {
                    // Drop database with FORCE (terminates connections automatically, PG 13+)
                    let _ =
                        sqlx::query(&format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name))
                            .execute(&admin_conn)
                            .await;

                    let _ = admin_conn.close().await;
                }
            });
        });
    }
}
