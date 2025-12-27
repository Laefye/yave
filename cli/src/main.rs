use clap::{Parser, Subcommand};
use qmp::types::InvokeCommand;
use vm_types::cloudinit::{Chpasswd, CloudConfig};
use yave::{contexts::{self, vm::DriveOptions}, newvmrunner::VmRunner, yavecontext::YaveContext};


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
        name: String,
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
            let context = contexts::yave::YaveContext::default();
            let vm_factory = contexts::vm::VirtualMachineFactory::new(&context, name)
                .vcpu(vcpu)
                .memory(memory)
                .drive(match image {
                    Some(image_path) => DriveOptions::From {
                        size: Some(capacity),
                        image: image_path,
                    },
                    None => DriveOptions::Empty {
                        size: capacity,
                    },
                });
            let vm_context = vm_factory.create().await.expect("Error creating VM");
            println!("Created VM at {:?}", vm_context);
        },
        Commands::Install { name } => {
            let context = contexts::yave::YaveContext::default();
            let vm = context.vm(&name);
            let installer = yave::installer::Installer::new(vm, CloudConfig {
                hostname: "pussy".to_string(), password: "123".to_string(), chpasswd: Chpasswd {
                    expire: false
                }, ssh_pwauth: true, power_state: Default::default() 
            });
            installer.install().await.expect("Error installing VM");
        }
        Commands::List => {
            let context = YaveContext::default();
            let vms = context.list().expect("Error listing VMs");
            for vm in vms  {
                println!("{}", vm);
            }
        },
        Commands::Run { name, vnc } => {
            let context = contexts::yave::YaveContext::default();
            let vm = context.vm(name);
            let runner = VmRunner::new(&vm);
            runner.run().await.expect("Error running VM");
            let qmp = qmp::client::Client::connect(&vm.qmp_socket()).await.expect("Error connecting to QMP");
            qmp.invoke(InvokeCommand::set_vnc_password(&vnc.unwrap_or("changeme".to_string()))).await.expect("Error setting VNC password");
        },
        Commands::Shutdown { name } => {
            let context = contexts::yave::YaveContext::default();
            let vm = context.vm(name);
            let qmp = vm.connect_qmp().await.expect("Error connecting to QMP");
            qmp.invoke(InvokeCommand::quit()).await.expect("Error shutting down VM");
        },
        Commands::Netdev { name, ifname, command } => {
            match command {
                NetdevCommand::Up => {
                    let context = YaveContext::default();
                    let vm = context.open_vm(&name).expect("Can't open vm");
                    
                    let vm_config = vm.vm_config().expect("Error loading VM config");
                    let (_, interface) = vm_config.networks.iter().next().expect("No networks configured");
                    if let Some(master) = &interface.get_network_device().master {
                        yave::interface::set_master(&ifname, master).await.expect("Error setting master");
                    }

                    yave::interface::set_link_up(&ifname).await.expect("Error setting link up");
                },
                NetdevCommand::Down => todo!(),
            }
        }
    }

}
