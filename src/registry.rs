pub struct VmRegistry {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(Debug, sqlx::FromRow)]
struct VirtualMachineRecord {
    id: i64,
    hostname: String,
    vcpu: u32,
    memory: u32,
    ovmf: bool,
    vnc_display: String,
}

impl VmRegistry {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { pool }
    }
}

