use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use vm_types::vm::DriveBus;

pub struct VmRegistry {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct VirtualMachineRecord {
    pub id: String,
    pub hostname: String,
    pub vcpu: u32,
    pub memory: u32,
    pub ovmf: bool,
    pub vnc_display: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct DriveRecord {
    pub vm_id: String,
    pub id: String,
    #[sqlx(json)]
    pub drive_bus: DriveBus,
}

#[derive(Debug, sqlx::FromRow)]
pub struct NetworkInterfaceRecord {
    pub ifname: String,
    pub vm_id: String,
    pub id: String,
    pub mac_address: String,
}

#[derive(Debug, sqlx::FromRow)]
pub struct IPv4AddressRecord {
    pub address: String,
    pub ifname: String,
    pub netmask: u32,
}

#[derive(Debug, Clone)]
pub struct CreateVirtualMachine {
    pub id: String,
    pub hostname: String,
    pub vcpu: u32,
    pub memory: u32,
    pub ovmf: bool,
    pub network_interfaces: Vec<CreateNetworkInterface>,
    pub drives: Vec<CreateDrive>,
}

#[derive(Debug, Clone)]
pub struct CreateNetworkInterface {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct AddIPv4Address {
    pub ifname: String,
    pub address: String,
    pub netmask: u32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CreateDrive {
    pub id: String,
    pub boot_order: Option<u32>,
    pub drive_bus: DriveBus,
}

pub fn get_mac(name: &str) -> String {
    use md5::{Digest, Md5};

    let mut hasher = Md5::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    format!(
        "52:54:{:02x}:{:02x}:{:02x}:{:02x}",
        hash[0], hash[1], hash[2], hash[3]
    )
}

type VmInfo = (VirtualMachineRecord, Vec<DriveRecord>, Vec<NetworkInterfaceRecord>, Vec<IPv4AddressRecord>);

impl VmRegistry {
    pub fn new(pool: sqlx::Pool<sqlx::Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create_tables(&self) -> Result<(), crate::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS virtual_machines (
                id TEXT PRIMARY KEY,
                hostname TEXT NOT NULL,
                vcpu INTEGER NOT NULL,
                memory INTEGER NOT NULL,
                ovmf BOOLEAN NOT NULL,
                vnc_display TEXT NOT NULL UNIQUE
            );
            CREATE TABLE IF NOT EXISTS network_interfaces (
                ifname TEXT PRIMARY KEY,
                vm_id TEXT NOT NULL,
                id TEXT NOT NULL,
                mac_address TEXT NOT NULL,
                FOREIGN KEY(vm_id) REFERENCES virtual_machines(id)
            );
            CREATE TABLE IF NOT EXISTS drives (
                vm_id TEXT NOT NULL,
                id TEXT NOT NULL,
                drive_bus TEXT NOT NULL,
                FOREIGN KEY(vm_id) REFERENCES virtual_machines(id)
            );
            CREATE TABLE IF NOT EXISTS ipv4_addresses (
                address TEXT PRIMARY KEY,
                ifname TEXT NOT NULL,
                netmask INTEGER NOT NULL,
                FOREIGN KEY(ifname) REFERENCES network_interfaces(ifname)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_ipv4_address(&self, addr: AddIPv4Address) -> Result<IPv4AddressRecord, crate::Error> {
        let record = sqlx::query_as::<_, IPv4AddressRecord>(
            r#"
            INSERT INTO ipv4_addresses (address, ifname, netmask)
            VALUES (?, ?, ?)
            RETURNING address, ifname, netmask;
            "#,
        )
            .bind(&addr.address)
            .bind(&addr.ifname)
            .bind(addr.netmask as i64)
            .fetch_one(&self.pool)
            .await?;
        Ok(record)
    }

    pub async fn get_virtual_machines(&self) -> Result<Vec<VirtualMachineRecord>, crate::Error> {
        let vms = sqlx::query_as::<_, VirtualMachineRecord>(
            r#"
            SELECT id, hostname, vcpu, memory, ovmf, vnc_display FROM virtual_machines;
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(vms)
    }

    async fn find_free_vnc_display(&self) -> Result<Option<String>, crate::Error> {
        let used_displays = sqlx::query_scalar::<_, String>(
            r#"
            SELECT vnc_display FROM virtual_machines;
            "#,
        )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        for display_num in 1..1000 {
            let display_str = format!(":{}", display_num);
            if !used_displays.contains(&display_str) {
                return Ok(Some(display_str));
            }
        }
        Ok(None)
    }

    async fn find_free_ifname(&self) -> Result<String, crate::Error> {
        let used_ifnames = sqlx::query_scalar::<_, String>(
            r#"
            SELECT ifname FROM network_interfaces;
            "#,
        )
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        for idx in 0..1000 {
            let ifname = format!("yave{}", idx);
            if !used_ifnames.contains(&ifname) {
                return Ok(ifname);
            }
        }
        Err(crate::Error::NoFreeIfname)
    }

    pub async fn create_vm(&self, vm: CreateVirtualMachine) -> Result<VirtualMachineRecord, crate::Error> {
        let vm_record = sqlx::query_as::<_, VirtualMachineRecord>(
            r#"
            INSERT INTO virtual_machines (id, hostname, vcpu, memory, ovmf, vnc_display)
            VALUES (?, ?, ?, ?, ?, ?) RETURNING id, hostname, vcpu, memory, ovmf, vnc_display;
            "#,
        )
            .bind(&vm.id)
            .bind(&vm.hostname)
            .bind(vm.vcpu as i64)
            .bind(vm.memory as i64)
            .bind(vm.ovmf)
            .bind(self.find_free_vnc_display().await?.unwrap())
            .fetch_one(&self.pool)
            .await?;
        for net in &vm.network_interfaces {
            let ifname = self.find_free_ifname().await?;
            sqlx::query(
                r#"
                INSERT INTO network_interfaces (ifname, vm_id, id, mac_address)
                VALUES (?, ?, ?, ?);
                "#,
            )
                .bind(&ifname)
                .bind(&vm.id)
                .bind(&net.id)
                .bind(get_mac(&ifname))
                .execute(&self.pool)
                .await?;
        }
        for drive in &vm.drives {
            sqlx::query(
                r#"
                INSERT INTO drives (vm_id, id, drive_bus)
                VALUES (?, ?, ?);
                "#,
            )
                .bind(&vm.id)
                .bind(&drive.id)
                .bind(serde_json::to_string(&drive.drive_bus)?)
                .execute(&self.pool)
                .await?;
        }
        Ok(vm_record)
    }

    pub async fn get_vm_by_id(&self, vm_id: &str) -> Result<VirtualMachineRecord, crate::Error> {
        let vm_record = sqlx::query_as::<_, VirtualMachineRecord>(
            r#"
            SELECT id, hostname, vcpu, memory, ovmf, vnc_display FROM virtual_machines WHERE id = ?;
            "#,
        )
            .bind(vm_id)
            .fetch_optional(&self.pool)
            .await?;
        vm_record.ok_or(crate::Error::VMNotFound)
    }

    async fn get_drives_by_vm_id(&self, vm_id: &str) -> Result<Vec<DriveRecord>, crate::Error> {
        let drives = sqlx::query_as::<_, DriveRecord>(
            r#"
            SELECT vm_id, id, drive_bus FROM drives WHERE vm_id = ?;
            "#,
        )
            .bind(vm_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(drives)
    }

    async fn get_ipv4_by_ifname(&self, ifname: &str) -> Result<Vec<IPv4AddressRecord>, crate::Error> {
        let addrs = sqlx::query_as::<_, IPv4AddressRecord>(
            r#"
            SELECT address, ifname, netmask FROM ipv4_addresses WHERE ifname = ?;
            "#,
        )
            .bind(ifname)
            .fetch_all(&self.pool)
            .await?;
        Ok(addrs)
    }

    pub async fn get_network_interfaces_by_vm_id(&self, vm_id: &str) -> Result<Vec<NetworkInterfaceRecord>, crate::Error> {
        let nics = sqlx::query_as::<_, NetworkInterfaceRecord>(
            r#"
            SELECT ifname, vm_id, id, mac_address FROM network_interfaces WHERE vm_id = ?;
            "#,
        )
            .bind(vm_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(nics)
    }

    pub async fn get_vm_full(&self, vm_id: &str) -> Result<VmInfo, crate::Error> {
        let vm_record = self.get_vm_by_id(vm_id).await?;
        let drives = self.get_drives_by_vm_id(vm_id).await?;
        let nics = self.get_network_interfaces_by_vm_id(vm_id).await?;
        let mut addrs = vec![];
        for nic in &nics {
            let mut nic_addrs = self.get_ipv4_by_ifname(&nic.ifname).await?;
            addrs.append(&mut nic_addrs);
        }
        Ok((vm_record, drives, nics, addrs))
    }

    pub async fn get_vm_by_ifname(&self, ifname: &str) -> Result<VirtualMachineRecord, crate::Error> {
        let vm_record = sqlx::query_as::<_, VirtualMachineRecord>(
            r#"
            SELECT vm.id, vm.hostname, vm.vcpu, vm.memory, vm.ovmf, vm.vnc_display
            FROM virtual_machines vm
            JOIN network_interfaces ni ON vm.id = ni.vm_id
            WHERE ni.ifname = ?;
            "#,
        )
            .bind(ifname)
            .fetch_optional(&self.pool)
            .await?;
        let vm_record = vm_record.ok_or(crate::Error::VMNotFound)?;
        Ok(vm_record)
    }
}

