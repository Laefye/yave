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
            facade.invoke(ShutdownVirtualMachinesInput {name}).await.expect("Error with shuting down");
        },
        Commands::Netdev { name, ifname, command } => {
            facade.invoke(NetdevVirtualMachinesInput {
                name,
                ifname,
                status: match command {
                    NetdevCommand::Up => true,
                    NetdevCommand::Down => false,
                }
            }).await.expect("Error with shuting down");
        }
    }

}
