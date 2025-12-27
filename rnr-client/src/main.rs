use clap::Parser;
use std::time::Instant;
use tokio;
use rnr_world::{World, WorldConfig};
// NetworkClient not yet implemented
// use rnr_network::client::NetworkClient;

/// Command line arguments for the RNR client
#[derive(Parser)]
#[command(name = "rnr-client")]
#[command(about = "RNR's Not Roblox - Game Client")]
struct Args {
    /// Server address to connect to
    #[arg(short, long, default_value = "127.0.0.1:53640")]
    server: String,

    /// Place ID to join
    #[arg(short, long)]
    place_id: Option<u32>,

    /// Enable rendering
    #[arg(long, default_value = "true")]
    render: bool,

    /// Enable networking
    #[arg(long, default_value = "true")]
    network: bool,

    /// Enable physics
    #[arg(long, default_value = "true")]
    physics: bool,

    /// Enable input handling
    #[arg(long, default_value = "true")]
    input: bool,

    /// Target FPS
    #[arg(long, default_value = "60")]
    fps: u32,

    /// Window width
    #[arg(long, default_value = "800")]
    width: u32,

    /// Window height
    #[arg(long, default_value = "600")]
    height: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    println!("RNR's Not Roblox - Client");
    println!("Connecting to: {}", args.server);
    println!("Rendering: {}", args.render);
    println!("Networking: {}", args.network);

    // Create world configuration
    let config = WorldConfig {
        enable_rendering: args.render,
        enable_networking: args.network,
        enable_physics: args.physics,
        enable_input: args.input,
        target_fps: args.fps,
        viewport_width: args.width,
        viewport_height: args.height,
    };

    // Create and initialize world
    let mut world = World::new(config);
    world.initialize().await?;

    // Set up networking if enabled
    if args.network {
        println!("Setting up network connection...");

        // Get the network client from the data model
        let datamodel = world.datamodel();
        let network_client = datamodel.borrow().get_service("NetworkClient");

        if let Some(client) = network_client {
            // In a real implementation, this would cast to NetworkClient
            // and attempt to connect to the server
            println!("Network client found, attempting connection...");
        } else {
            println!("Warning: NetworkClient service not found");
        }
    }

    // Main game loop
    println!("Starting game loop...");
    let start_time = Instant::now();

    loop {
        // Process one frame
        world.step().await?;

        // Check for quit conditions (in a real implementation, this would check input)
        let elapsed = start_time.elapsed();
        if elapsed.as_secs() > 10 {
            // Exit after 10 seconds for demo purposes
            println!("Demo completed, shutting down...");
            break;
        }

        // Small delay to prevent busy waiting
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }

    // Shutdown
    world.shutdown().await;

    println!("Client shutdown complete.");
    Ok(())
}
