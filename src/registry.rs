pub struct VmRegistry {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(Debug, sqlx::FromRow)]
struct VirtualMachineRecord {
    id: i64,
    name: String,
}

impl VmRegistry {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { pool }
    }
}

