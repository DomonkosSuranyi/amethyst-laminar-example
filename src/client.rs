use amethyst::network::simulation::laminar::{LaminarSocket, LaminarNetworkBundle};
use amethyst::prelude::*;
use amethyst::utils::application_root_dir;
use amethyst::network::simulation::{NetworkSimulationEvent, NetworkSimulationTime, TransportResource};
use amethyst::core::ecs::{System, Read, Write, ReaderId, DispatcherBuilder};
use amethyst::core::{Time, SystemBundle};
use amethyst::core::ecs::shrev::EventChannel;
use amethyst::core::ecs::shred::SystemData;
use log::{info, error};
use std::time::Duration;

const CLIENT_ADDRESS: &str = "127.0.0.1:3455";

pub fn start_client() -> amethyst::Result<()> {
    // To use default laminar config you can simply use `LaminarSocket::bind(CLIENT_ADDRESS)`
    let socket = LaminarSocket::bind_with_config(CLIENT_ADDRESS, crate::create_laminar_config(Some(Duration::from_secs(1))))?;
    let game_data = GameDataBuilder::default()
        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with_bundle(ClientSystemBundle)?;

    let mut app = Application::build(application_root_dir()?, ClientGameState)?
        .build(game_data)?;

    app.run();
    Ok(())
}

pub struct ClientGameState;
impl SimpleState for ClientGameState {}

#[derive(Debug)]
struct ClientSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for ClientSystemBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> amethyst::Result<()> {
        builder.add(ClientSystemDesc::default().build(world), "client_system", &[]);
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ClientSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, ClientSystem> for ClientSystemDesc {
    fn build(self, world: &mut World) -> ClientSystem {
        <ClientSystem as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();

        ClientSystem::new(reader)
    }
}

struct ClientSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl ClientSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for ClientSystem {
    type SystemData = (
        Read<'a, NetworkSimulationTime>,
        Read<'a, Time>,
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
    );
    fn run(&mut self, (sim_time, time, mut net, event /*, tx*/): Self::SystemData) {
        let server_addr = crate::server::SERVER_ADDRESS.parse().unwrap();

        // stop sending message between 5 and 10 seconds to demonstrate heartbeat
        let abs_secs = time.absolute_time_seconds();
        if abs_secs > 5 as f64 && abs_secs < 10 as f64 {
            info!("Waiting... time: {}", abs_secs);
        } else {
            for frame in sim_time.sim_frames_to_run() {
                info!("Sending message for sim frame {}.", frame);
                let payload = format!(
                    "CL: sim_frame:{},abs_time:{}",
                    frame,
                    time.absolute_time_seconds()
                );

                net.send_with_requirements(
                    server_addr,
                    payload.as_bytes(),
                    crate::DELIVERY_REQUIREMENT,
                    crate::URGENCY_REQUIREMENT);
            }
        }

        for event in event.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(_addr, payload) => info!("Payload: {:?}", payload),
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => info!("Server Disconnected: {}", addr),
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                NetworkSimulationEvent::SendError(e, msg) => {
                    error!("Send Error: {:?}, {:?}", e, msg);
                }
                _ => {}
            }
        }
    }
}
