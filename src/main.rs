mod server;
mod client;

use log::info;
use amethyst::network::simulation::{DeliveryRequirement, UrgencyRequirement};
use amethyst::network::simulation::laminar::LaminarConfig;
use std::time::Duration;

pub const DELIVERY_REQUIREMENT: DeliveryRequirement = DeliveryRequirement::Unreliable;
pub const URGENCY_REQUIREMENT: UrgencyRequirement = UrgencyRequirement::OnTick;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());
    let args: Vec<String> = std::env::args().collect();
    let rtn: amethyst::Result<()>;

    if args.len() < 2 || args[1].starts_with("c") {
        info!("Starting the client");
        rtn = client::start_client();
    } else if args[1].starts_with("s") {
        info!("Starting the server");
        rtn = server::start_server();
    } else {
        panic!("Invalid command line args. Use 's' for server or 'c' for client");
    }
    rtn
}

fn create_laminar_config(heartbeat_interval: Option<Duration>) -> LaminarConfig {
    LaminarConfig {
        blocking_mode: false,
        idle_connection_timeout: Duration::from_secs(3),
        heartbeat_interval,
        max_packet_size: (16 * 1024) as usize,
        max_fragments: 16 as u8,
        fragment_size: 1024,
        fragment_reassembly_buffer_size: 64,
        receive_buffer_max_size: 1452 as usize,
        rtt_smoothing_factor: 0.10,
        rtt_max_value: 250,
        socket_event_buffer_size: 1024,
        socket_polling_timeout: Some(Duration::from_millis(1)),
        max_packets_in_flight: 512,
    }
}


