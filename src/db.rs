use rusqlite::Connection;
use vm_types::VirtualMachine;

pub fn insert_vm(
    conn: &Connection,
    name: &str,
    vm: &VirtualMachine,
) -> Result<(), crate::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO vms (name, config) VALUES (?1, ?2)",
        &[name, &serde_json::to_string(vm)?],
    )?;
    Ok(())
}

pub fn get_vm(
    conn: &Connection,
    name: &str,
) -> Result<Option<VirtualMachine>, crate::Error> {
    let mut stmt = conn.prepare("SELECT config FROM vms WHERE name = ?1")?;
    let mut rows = stmt.query([name])?;
    if let Some(row) = rows.next()? {
        let config_str: String = row.get(0)?;
        let vm: VirtualMachine = serde_json::from_str(&config_str)?;
        Ok(Some(vm))
    } else {
        Ok(None)
    }
}

pub fn get_vms(
    conn: &Connection,
) -> Result<Vec<VirtualMachine>, crate::Error> {
    let mut stmt = conn.prepare("SELECT config FROM vms")?;
    let vms = stmt.query_map([], |row| {
        row.get(0)
    })?.collect::<Result<Vec<String>, _>>()?.iter().map(|x| {
        serde_json::from_str::<VirtualMachine>(x).map_err(crate::Error::from)
    }).collect::<Result<Vec<VirtualMachine>, crate::Error>>()?;
    Ok(vms)
}

fn format_vnc(display: u32) -> String {
    format!(":{}", display)
}

pub fn allocate_vnc_display(
    conn: &mut Connection,
    name: &str,
) -> Result<String, crate::Error> {
    let tx = conn.transaction()?;

    let used_displays: Vec<String> = {
        let mut stmt = tx.prepare("SELECT display FROM vnc")?;
        stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?
    };

    let mut display = 1;
    while used_displays.iter().find(|x| *x == &format_vnc(display)).is_some() {
        display += 1;
    }

    let display_str = format_vnc(display);
    tx.execute(
        "INSERT INTO vnc (vm, display) VALUES (?1, ?2)",
        rusqlite::params![name, &display_str],
    )?;
    tx.commit()?;
    Ok(display_str)
}

pub fn create_tables(
    conn: &Connection,
) -> Result<(), crate::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vms (
            name TEXT PRIMARY KEY,
            config TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vnc (
            display TEXT PRIMARY KEY,
            vm TEXT NOT NULL,
            FOREIGN KEY(display) REFERENCES vms(name)
        )",
        [],
    )?;
    Ok(())
}
