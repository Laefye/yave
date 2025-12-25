use clap::{Parser, Subcommand};
use qmp::types::InvokeCommand;
use yave::{DefaultFacade, Facade, vms::{InputOperatingSystem, ListVirtualMachinesInput, NetdevVirtualMachinesInput, RunVirtualMachinesInput, ShutdownVirtualMachinesInput, VirtualMachineCreateInput}, yavecontext::{CreateDriveOptions, CreateVirtualMachineInput, YaveContext}};

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
        image: Option<String>
    },
    List,
    Run {
        #[arg(short, long)]
        name: String,
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
    let facade = DefaultFacade{};
    match args.cmd {
        Commands::Create { name, vcpu, memory, capacity, image } => {
            let context = YaveContext::default();
            context.create_vm(
                CreateVirtualMachineInput::new(&name)
                    .drive(CreateDriveOptions::Empty { size: capacity })
                    .vcpu(vcpu)
                    .memory(memory)
            ).await.expect("Error with creation");
        },
        Commands::List => {
            let context = YaveContext::default();
            let vms = context.list().expect("Error listing VMs");
            for vm in vms  {
                println!("{}", vm);
            }
        },
        Commands::Run { name } => {
            let context = YaveContext::default();
            let vm = context.open_vm(&name);
            vm.run().await.expect("Error running VM");
            
        },
        Commands::Shutdown { name } => {
            let context = YaveContext::default();
            let vm = context.open_vm(&name);
            let client = vm.connect_qmp().await.expect("Error connecting to QMP");
            client.invoke(InvokeCommand::quit()).await.expect("Error with shutting down");
        },
        Commands::Netdev { name, ifname, command } => {
            match command {
                NetdevCommand::Up => {
                    let context = YaveContext::default();
                    let vm = context.open_vm(&name);
                    
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
