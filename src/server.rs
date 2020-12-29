use amethyst::prelude::*;
use amethyst::network::simulation::laminar::{LaminarSocket, LaminarNetworkBundle};
use amethyst::utils::application_root_dir;
use amethyst::core::SystemBundle;
use amethyst::core::ecs::{DispatcherBuilder, System, ReaderId, Write, Read};
use amethyst::network::simulation::{NetworkSimulationEvent, TransportResource};
use amethyst::core::ecs::shrev::EventChannel;
use amethyst::core::ecs::shred::SystemData;
use log::{info, error};
use std::time::Duration;

pub const SERVER_ADDRESS: &str = "127.0.0.1:2222";

pub fn start_server() -> amethyst::Result<()> {
    // To use default laminar config you can simply use `LaminarSocket::bind(SERVER_ADDRESS)`
    let socket = LaminarSocket::bind_with_config(SERVER_ADDRESS, crate::create_laminar_config(Some(Duration::from_secs(1))))?;
    let game_data = GameDataBuilder::default()
        .with_bundle(LaminarNetworkBundle::new(Some(socket)))?
        .with_bundle(ServerSystemBundle)?;

    let mut app = Application::build(application_root_dir()?, ServerGameState)?
        .build(game_data)?;

    app.run();
    Ok(())
}

struct ServerGameState;

impl SimpleState for ServerGameState {}

#[derive(Debug)]
struct ServerSystemBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for ServerSystemBundle {
    fn build(self, world: &mut World, builder: &mut DispatcherBuilder<'a, 'b>) -> amethyst::Result<()> {
        builder.add(
            ServerSystemDesc::default().build(world),
            "server_system",
            &[],
        );
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct ServerSystemDesc;

impl<'a, 'b> SystemDesc<'a, 'b, ServerSystem> for ServerSystemDesc {
    fn build(self, world: &mut World) -> ServerSystem {
        <ServerSystem as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<NetworkSimulationEvent>>()
            .register_reader();
        ServerSystem::new(reader)
    }
}

struct ServerSystem {
    reader: ReaderId<NetworkSimulationEvent>,
}

impl ServerSystem {
    pub fn new(reader: ReaderId<NetworkSimulationEvent>) -> Self {
        Self { reader }
    }
}

impl<'a> System<'a> for ServerSystem {
    type SystemData = (
        Write<'a, TransportResource>,
        Read<'a, EventChannel<NetworkSimulationEvent>>,
    );

    fn run(&mut self, (mut net, channel): Self::SystemData) {
        for event in channel.read(&mut self.reader) {
            match event {
                NetworkSimulationEvent::Message(addr, payload) => {
                    info!("{}: {:?}", addr, payload);
                    net.send_with_requirements(
                        *addr,
                        b"ok",
                        crate::DELIVERY_REQUIREMENT,
                        crate::URGENCY_REQUIREMENT);
                }
                NetworkSimulationEvent::Connect(addr) => info!("New client connection: {}", addr),
                NetworkSimulationEvent::Disconnect(addr) => {
                    info!("Client Disconnected: {}", addr);
                }
                NetworkSimulationEvent::RecvError(e) => {
                    error!("Recv Error: {:?}", e);
                }
                _ => {}
            }
        }
    }
}

