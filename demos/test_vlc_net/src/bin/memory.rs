use sysinfo::{ProcessesToUpdate, System};

fn main() {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let pid = sysinfo::get_current_pid().expect("Failed to get current PID");

    if let Some(process) = sys.process(pid) {
        println!("Memory usage: {} bytes", process.memory());
    } else {
        println!("Process not found");
    }
}
