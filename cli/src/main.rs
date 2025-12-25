use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use yave::{DefaultFacade, Facade, vms::{InputOperatingSystem, ListVirtualMachinesInput, NetdevVirtualMachinesInput, RunVirtualMachinesInput, ShutdownVirtualMachinesInput, VirtualMachineCreateInput}};

/// Simple program to greet a person
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
        #[arg(short, long, default_value = "512")]
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
            facade.invoke(VirtualMachineCreateInput{
                name,
                vcpu,
                memory,
                capacity,
                os: match image {
                    None => InputOperatingSystem::Empty,
                    Some(path) => InputOperatingSystem::Image(path)
                },
            }).await.expect("Error with creating");
        },
        Commands::List => {
            let vms = facade.invoke(ListVirtualMachinesInput).await.expect("Errow with listing");
            for vm in vms  {
                println!("{}", vm);
            }
        },
        Commands::Run { name } => {
            facade.invoke(RunVirtualMachinesInput {name}).await.expect("Error with running");
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
