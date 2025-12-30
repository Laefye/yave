use clap::{Parser, Subcommand};
use qmp::types::InvokeCommand;
use vm_types::{cloudinit::{Chpasswd, ChpasswdUser, CloudConfig}, vm::DriveBus};
use yave::{DefaultYaveContext, registry::{self, CreateDrive, CreateNetworkInterface, CreateVirtualMachine}, storage::{DriveInstallMode, InstallOptions}};


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
        capacity: u32,
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
    Netdev {
        #[arg(short, long)]
        ifname: String,
        #[command(subcommand)]
        command: NetdevCommand,
    }
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
                    boot_order: Some(1),
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
            // let context = contexts::yave::YaveContext::default();
            // let vm = context.vm(&name);
            // let installer = yave::installer::Installer::new(vm, CloudConfig {
            //     hostname: "pussy".to_string(), chpasswd: Chpasswd {
            //         expire: false,
            //         users: vec![
            //             ChpasswdUser {
            //                 name: "root".to_string(),
            //                 password: "uwu".to_string(),
            //                 type_password: "text".to_string(),
            //             }
            //         ],
            //     }, ssh_pwauth: true, power_state: Default::default() ,
            // });
            // installer.install().await.expect("Error installing VM");
            todo!();
        }
        Commands::List => {
            todo!();
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
        Commands::Netdev { ifname, command } => {
            // let context = contexts::yave::YaveContext::default();
            // let vm = context.get_vm_by_ifname(&ifname).expect("Error getting VM by ifname");
            // if let Some(vm) = vm {
            //     let vm_config = vm.vm().expect("Impossible read");
            //     let (_id, net) = vm_config.networks.iter()
            //         .find(|(_, net)| net.ifname == ifname)
            //         .expect("No network found for interface");
            //     match command {
            //         NetdevCommand::Up => {
            //             yave::interface::set_link_up(&ifname).await.expect("Error bringing up interface");
            //             if let Some(master) = &net.device.master {
            //                 yave::interface::set_master(&ifname, master).await.expect("Error setting master");
            //             }
            //         },
            //         NetdevCommand::Down => {
            //         },
            //     }
            // } else {
            //     eprintln!("No VM found for interface {}", ifname);
            // }
            println!("Not implemented yet");
        },
        Commands::Inspect { name } => {
            // let context = contexts::yave::YaveContext::default();
            // let vm = context.vm(&name);
            // let vm_config = vm.vm().expect("Error getting VM configuration");
            // println!("{:#?}", vm_config);
            todo!();
        }
    }

}
