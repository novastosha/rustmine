use std::io;

use rustmine_server::{config::ServerConfig, event::player_events::PlayerJoinedServer, RustmineServer};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = RustmineServer::new(ServerConfig::default());
    {
        let event_bus = &mut server.lock().await.event_bus;

        event_bus.listen::<PlayerJoinedServer, _, _, _>(false, |event| async move {
            let player = event.player.lock().await;
            let server = player.server.lock().await;
            
            None
        }).await;
    }

    return match RustmineServer::run(server.clone()).await {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Caught an unhandled error: {}", err),
        )),
    };
}
