use tokio::process::Command;
use yave::{config::Config, pathes::{get_config_path, get_run_path}, qemu::QEMU, qmp};


#[tokio::main]
async fn main() {
    let config = Config::load(&get_config_path()).expect("Failed to load config");
    println!("Loaded config: {:?}", config);
    let qemu_args = QEMU::new(&config.kvm.bin)
        .memory(512)
        .smp(4)
        .qmp(&get_run_path().join("qmp.sock")).expect("Failed to set QMP socket")
        .pidfile(&get_run_path().join("qemu.pid")).expect("Failed to set PID file")
        .daemonize()
        .vnc(":1", true)
        .build();
    println!("QEMU arguments: {:?}", qemu_args);
    let child = Command::new(&qemu_args[0])
        .args(&qemu_args[1..])
        .spawn()
        .expect("Failed to start QEMU");
    let output = child.wait_with_output().await.expect("Failed to wait on QEMU");
    println!("QEMU exited with: {:?}", output);

    let client = qmp::client::Client::connect(&get_run_path().join("qmp.sock")).await;
    match client {
        Ok(client) => {
            let response = client.invoke(qmp::types::InvokeCommand::set_vnc_password("12345678")).await;
            match response {
                Ok(resp) => println!("QMP response: {:?}", resp),
                Err(e) => eprintln!("Failed to invoke QMP command: {}", e),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            drop(client);
        },
        Err(e) => eprintln!("Failed to connect to QMP socket: {}", e),
    }

}
