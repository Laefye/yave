use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use vm_types::VirtualMachine;

pub const TABLE_VM: TableDefinition<&str, Vec<u8>> = TableDefinition::new("vm");

fn map_db_err(e: impl Into<redb::Error>) -> crate::Error {
    crate::Error::Database(e.into())
}

fn map_wincode_err(e: impl Into<wincode::Error>) -> crate::Error {
    crate::Error::Wincode(e.into())
}


pub fn insert_vm(
    db: &redb::Database,
    name: &str,
    vm: &VirtualMachine,
) -> Result<(), crate::Error> {
    let tx = db.begin_write().map_err(map_db_err)?;
    {
        let mut table = tx.open_table(TABLE_VM).map_err(map_db_err)?;
        table.insert(name, &wincode::serialize(vm).map_err(map_wincode_err)?).map_err(map_db_err)?;
    }
    tx.commit().map_err(map_db_err)?;
    Ok(())
}

pub fn get_vm(
    db: &redb::Database,
    name: &str,
) -> Result<Option<VirtualMachine>, crate::Error> {
    let tx = db.begin_read().map_err(map_db_err)?;
    let table = tx.open_table(TABLE_VM).map_err(map_db_err)?;
    if let Some(value) = table.get(name).map_err(map_db_err)? {
        let vm: VirtualMachine = wincode::deserialize(&value.value()).map_err(map_wincode_err)?;
        Ok(Some(vm))
    } else {
        Ok(None)
    }
}

pub fn get_vms(
    db: &redb::Database,
) -> Result<Vec<VirtualMachine>, crate::Error> {
    let tx = db.begin_read().map_err(map_db_err)?;
    let table = tx.open_table(TABLE_VM).map_err(map_db_err)?;
    table.iter()
        .map_err(map_db_err)?
        .map(|entry| {
            let (_key, value) = entry.map_err(map_db_err)?;
            Ok(wincode::deserialize(&value.value()).map_err(map_wincode_err)?)
        })
        .collect()
}
