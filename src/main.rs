use clap::{Parser, Subcommand};
use qmp::client::Client;
use qmp::types::InvokeCommand;
use tokio::process::Command;
use yave::{config::{Config, VirtualMachine}, pathes::{get_config_path, get_run_path}, run::RunFactory};


#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    vm: String,
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Run,
    Stop,
    Show,
}

#[tokio::main]
async fn main() {
    let config = Config::load(&get_config_path()).expect("Failed to load config");
    println!("Loaded config");
    let cli = Cli::parse();
    let vm = VirtualMachine::load(&cli.vm).expect("Failed to load VM config");
    println!("Loaded VM config");

    let run = RunFactory::new(
        get_run_path(),
        get_run_path(),
        &vm,
        &config,
    );

    match cli.command {
        Subcommands::Run => {
            let args = run.build_qemu_command();

            let mut child = Command::new(&args[0])
                .args(&args[1..])
                .spawn()
                .expect("Failed to start QEMU");

            println!("QEMU exited with: {:?}", child.wait().await);

            let qmp = Client::connect(run.get_socket_path()).await.expect("Failed to connect to QMP");
            qmp.invoke(InvokeCommand::set_vnc_password(&vm.vnc.password)).await.expect("Failed to set VNC password");
        },
        Subcommands::Stop => {
            let qmp = Client::connect(run.get_socket_path()).await.expect("Failed to connect to QMP");
            qmp.invoke(InvokeCommand::empty("quit")).await.expect("Failed to quit");
        },
        Subcommands::Show => {
            println!("VM Name: {}", vm.name);
            println!("Memory: {} MB", vm.hardware.memory);
            println!("vCPUs: {}", vm.hardware.vcpu);
            println!("Drives:");
            for (id, drive) in &vm.drives {
                println!("  ID: {}, Path: {}", id, drive.path);
            }
            println!("Networks:");
            for (id, net) in &vm.networks {
                match net {
                    yave::config::NetworkInterface::Tap(tap) => {
                        println!("  ID: {}, Type: Tap, Ifname: {}, MAC: {}", id, tap.ifname, tap.device.mac);
                    },
                }
            }
            let args = run.build_qemu_command();
            println!("QEMU Command: {:?}", args.join(" "));
        }
    }
}
