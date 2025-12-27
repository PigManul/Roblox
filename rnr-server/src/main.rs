use clap::Parser;
use std::time::Instant;
use tokio;
use rnr_world::{World, WorldConfig};
// NetworkServer not yet implemented
// use rnr_network::server::NetworkServer;

/// Command line arguments for the RNR server
#[derive(Parser)]
#[command(name = "rnr-server")]
#[command(about = "RNR's Not Roblox - Game Server")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "53640")]
    port: u16,

    /// Place ID to host
    #[arg(short, long)]
    place_id: Option<u32>,

    /// Enable physics simulation
    #[arg(long, default_value = "true")]
    physics: bool,

    /// Enable networking
    #[arg(long, default_value = "true")]
    network: bool,

    /// Target tick rate (Hz)
    #[arg(long, default_value = "30")]
    tick_rate: u32,

    /// Maximum players
    #[arg(long, default_value = "50")]
    max_players: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    println!("RNR's Not Roblox - Server");
    println!("Listening on port: {}", args.port);
    println!("Max players: {}", args.max_players);
    println!("Tick rate: {} Hz", args.tick_rate);

    // Create world configuration (server doesn't need rendering or input)
    let config = WorldConfig {
        enable_rendering: false,  // Server doesn't render
        enable_networking: args.network,
        enable_physics: args.physics,
        enable_input: false,      // Server doesn't handle input
        target_fps: args.tick_rate,
        viewport_width: 1,        // Dummy values
        viewport_height: 1,
    };

    // Create and initialize world
    let mut world = World::new(config);
    world.initialize().await?;

    // Set up networking if enabled
    if args.network {
        println!("Setting up network server...");

        // In a real implementation, this would create and start a NetworkServer
        // and register it with the data model
        println!("Network server started on port {}", args.port);
    }

    // Main server loop
    println!("Starting server loop...");
    let start_time = Instant::now();

    loop {
        // Process one server tick
        world.step().await?;

        // Check for shutdown conditions
        let elapsed = start_time.elapsed();
        if elapsed.as_secs() > 30 {
            // Exit after 30 seconds for demo purposes
            println!("Demo server session completed, shutting down...");
            break;
        }

        // Sleep to maintain tick rate
        let target_tick_duration = std::time::Duration::from_secs_f32(1.0 / args.tick_rate as f32);
        tokio::time::sleep(target_tick_duration).await;
    }

    // Shutdown
    world.shutdown().await;

    println!("Server shutdown complete.");
    Ok(())
}
