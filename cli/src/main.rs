use clap::{Parser, Subcommand};
use qmp::types::InvokeCommand;
use vm_types::vm::DriveBus;
use yave::{DefaultYaveContext, builders::{CloudInitBuilder, VmLaunchRequestBuilder}, cloudinit::CloudInitInstaller, net::NetworkManager, registry::{AddIPv4Address, CreateDrive, CreateNetworkInterface, CreateVirtualMachine}, storage::{DriveInstallMode, InstallOptions}};


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum NetdevCommand {
    Up,
    Down,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long, default_value = "1")]
        vcpu: u32,
        #[arg(short, long, default_value = "1024")]
        memory: u32,
        #[arg(short, long, default_value = "15360")]
        capacity: u64,
        #[arg(short, long)]
        image: Option<String>,
    },
    List,
    Install {
        #[arg(short, long)]
        name: String,
    },
    Inspect {
        #[arg(short, long)]
        name: String,
    },
    Address {
        #[arg(short, long)]
        ifname: String,
        #[arg(short, long)]
        address: String,
        #[arg(short, long)]
        netmask: u32,
        #[arg(short, long)]
        gateway: Option<String>,
    },
    Run {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        vnc: Option<String>,
    },
    Shutdown {
        #[arg(short, long)]
        name: String,
    },
    Reboot {
        #[arg(short, long)]
        name: String,
    },
    Netdev {
        #[arg(short, long)]
        ifname: String,
        #[command(subcommand)]
        command: NetdevCommand,
    },
    Delete {
        #[arg(short, long)]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.cmd {
        Commands::Create { name, vcpu, memory, capacity, image } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let registry = context.registry();
            registry.create_tables().await.expect("Error creating tables");
            registry.create_vm(CreateVirtualMachine {
                id: name.to_string(),
                hostname: name.to_string(),
                vcpu,
                memory,
                ovmf: true,
                network_interfaces: vec![CreateNetworkInterface {
                    id: "net0".to_string(),
                }],
                drives: vec![CreateDrive {
                    id: "drive0".to_string(),
                    drive_bus: DriveBus::VirtioBlk { boot_index: Some(1) },
                }],
            }).await.expect("Error creating VM");
            let storage = context.storage();
            storage.install_vm(
                &name,
                &InstallOptions {
                    drives: vec![
                        match image {
                            Some(image_path) => DriveInstallMode::Existing {
                                id: "drive0".to_string(),
                                resize: capacity,
                                image: image_path,
                            },
                            None => DriveInstallMode::New {
                                id: "drive0".to_string(),
                                size: capacity,
                            },
                        }
                    ],
                }
            ).await.expect("Error installing VM");
        },
        Commands::Install { name } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let builder = yave::builders::VmLaunchRequestBuilder::new(&context);
            let launch_request = builder.build(&name).await.expect("Error building launch request");
            let cloud_config = CloudInitBuilder::new(&context).build(&name, "changeme").await.expect("Error building cloud-init config");
            let installer = CloudInitInstaller::new(&context);
            installer.install(&launch_request, &cloud_config).await.expect("Error installing cloud-init ISO");
        },
        Commands::List => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let registry = context.registry();
            let vms = registry.get_virtual_machines().await.expect("Error listing VMs");
            for vm in vms {
                println!("VM: {}", vm.id);
            }
        },
        Commands::Run { name, vnc } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let builder = yave::builders::VmLaunchRequestBuilder::new(&context);
            let launch_request = builder.build(&name).await.expect("Error building launch request");
            let runtime = context.runtime();
            let vnc = vnc.unwrap_or("changeme".to_string());
            runtime.run_vm(&launch_request).await.expect("Error running VM");
            runtime.qmp_connect(&launch_request).await.expect("Error connecting to QMP")
                .invoke(InvokeCommand::set_vnc_password(&vnc)).await.expect("Error setting VNC password");
        },
        Commands::Shutdown { name } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let builder = yave::builders::VmLaunchRequestBuilder::new(&context);
            let launch_request = builder.build(&name).await.expect("Error building launch request");
            let runtime = context.runtime();
            runtime.shutdown_vm(&launch_request).await.expect("Error shutting down VM");
        },
        Commands::Reboot { name } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let builder = yave::builders::VmLaunchRequestBuilder::new(&context);
            let launch_request = builder.build(&name).await.expect("Error building launch request");
            let runtime = context.runtime();
            runtime.qmp_connect(&launch_request).await.expect("Error connecting to QMP")
                .invoke(InvokeCommand::reboot()).await.expect("Error rebooting VM");
        },
        Commands::Netdev { ifname, command } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let nm = NetworkManager::new(&context);
            match command {
                NetdevCommand::Up => {
                    nm.up_interface(&ifname).await.expect("Error bringing up interface");
                },
                NetdevCommand::Down => {
                    // Currently not implemented
                    println!("Not implemented yet");
                },
            }
        },
        Commands::Inspect { name } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let registry = context.registry();
            let vm = registry.get_vm_full(&name).await.expect("Error inspecting VM");
            println!("VM: {:?}", vm);
        },
        Commands::Address { ifname, address, netmask, gateway } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let registry = context.registry();
            registry.add_ipv4_address(AddIPv4Address {
                ifname,
                address,
                netmask,
                gateway,
            }).await.expect("Error adding address");
        },
        Commands::Delete { name } => {
            let context = DefaultYaveContext::create().await.expect("Error creating context");
            let builder = VmLaunchRequestBuilder::new(&context);
            let launch_request = builder.build(&name).await.expect("Error building launch request");
            let runtime = context.runtime();
            if runtime.is_running(&launch_request).await.expect("Error checking if VM is running") {
                runtime.shutdown_vm(&launch_request).await.expect("Error shutting down VM");
            }
            let storage = context.storage();
            storage.delete_vm(&name).await.expect("Error deleting VM");
            let registry = context.registry();
            registry.delete_vm(&name).await.expect("Error deleting VM from registry");
        },
    }

}
