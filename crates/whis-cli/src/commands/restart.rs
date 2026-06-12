use crate::ipc;
use anyhow::Result;

pub fn run(autotype: bool) -> Result<()> {
    if ipc::is_service_running() {
        let mut client = ipc::IpcClient::connect()?;
        let _ = client.send_message(ipc::IpcMessage::Stop)?;
        println!("Service stopped");
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    crate::commands::start::run(autotype)
}
