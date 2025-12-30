use redb::{ReadableDatabase, ReadableTable, TableDefinition, TableError};
use vm_types::VirtualMachine;

pub const TABLE_VM: TableDefinition<&str, Vec<u8>> = TableDefinition::new("vm");

/// VNC display to VM name mapping
/// Key: display (e.g., ":1")
/// Value: VM name
pub const TABLE_VNC: TableDefinition<&str, &str> = TableDefinition::new("vnc");

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

fn open_table_or_none<'a>(
    tx: &redb::ReadTransaction,
    table_def: TableDefinition<'a, &str, &str>,
) -> Result<Option<redb::ReadOnlyTable<&'a str, &'a str>>, crate::Error> {
    tx.open_table(table_def)
        .map(Some)
        .or_else(|e| match e {
            TableError::TableDoesNotExist(_) => Ok(None),
            _ => Err(map_db_err(e)),
        })
}

pub fn allocate_vnc_display(
    db: &redb::Database,
    name: &str,
) -> Result<String, crate::Error> {
    let tx = db.begin_read().map_err(map_db_err)?;
    let mut port = 1;
    {
        let table = open_table_or_none(&tx, TABLE_VNC)?;
        if let Some(table) = table {
            while table.get(format!(":{}", port).as_str()).map_err(map_db_err)?.is_some() {
                port += 1;
            }
        }
    }
    let port = format!(":{}", port);
    println!("Allocated VNC display {} for VM {}", port, name);
    let rx = db.begin_write().map_err(map_db_err)?;
    {
        let mut table = rx.open_table(TABLE_VNC).map_err(map_db_err)?;
        table.insert(port.as_str(), name).map_err(map_db_err)?;
    }
    rx.commit().map_err(map_db_err)?;
    Ok(port)
}
