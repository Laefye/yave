use clap::{Parser, Subcommand};
use tokio::process::Command;
use yave::{config::{Config, VirtualMachine}, pathes::{get_config_path, get_run_path}, qmp::{self, types::InvokeCommand}, run::RunFactory};


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
}

#[tokio::main]
async fn main() {
    let config = Config::load(&get_config_path()).expect("Failed to load config");
    println!("Loaded config: {:?}", config);
    let cli = Cli::parse();
    let vm = VirtualMachine::load(&cli.vm).expect("Failed to load VM config");
    println!("Loaded VM config: {:?}", vm);

    let run = RunFactory::new(
        get_run_path(),
        get_run_path(),
        &vm,
        &config,
    );

    match cli.command {
        Subcommands::Run => {
            let args = run.build_qemu_command();

            println!("QEMU arguments: {:?}", args);

            let mut child = Command::new(&args[0])
                .args(&args[1..])
                .spawn()
                .expect("Failed to start QEMU");

            println!("QEMU exited with: {:?}", child.wait().await);

            let qmp = qmp::client::Client::connect(run.get_socket_path()).await.expect("Failed to connect to QMP");
            qmp.invoke(InvokeCommand::set_vnc_password(&vm.vnc.password)).await.expect("Failed to set VNC password");
        },
        Subcommands::Stop => {
            let qmp = qmp::client::Client::connect(run.get_socket_path()).await.expect("Failed to connect to QMP");
            qmp.invoke(InvokeCommand::empty("quit")).await.expect("Failed to quit");
        },
    }
}
