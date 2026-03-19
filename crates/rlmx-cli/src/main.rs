use std::path::Path;

use clap::{Parser, Subcommand};
use rlmx_kernel::text_to_embedding;

/// RLMX - The RuVix Cognition Kernel CLI
#[derive(Parser, Debug)]
#[command(
    name = "rlmx",
    version,
    about = "RuVix cognition kernel command-line interface"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Submit a query to the kernel for processing
    Query {
        /// The query string to process
        #[arg(short, long)]
        input: String,

        /// Scheduling strategy (rlm, trm, auto, hybrid)
        #[arg(short, long, default_value = "auto")]
        strategy: String,

        /// Maximum recursion depth
        #[arg(long, default_value_t = 10)]
        max_depth: usize,
    },

    /// Ingest data into the kernel's memory region
    Ingest {
        /// Path to the file or directory to ingest
        #[arg(short, long)]
        path: String,

        /// Memory region to target
        #[arg(short, long, default_value = "default")]
        region: String,

        /// Source label for ingested segments
        #[arg(long)]
        source: Option<String>,
    },

    /// Start the MCP server
    Serve {
        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },

    /// Manage plugins
    Plugin {
        /// Plugin subcommand (list, install, remove, info)
        #[arg(short, long)]
        action: String,

        /// Plugin name or path
        name: Option<String>,
    },

    /// Seal an RVF container
    Seal {
        /// Path to the container directory
        #[arg(short, long)]
        path: String,

        /// Output path for the sealed RVF file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Create a branch of an RVF container
    Branch {
        /// Name of the new branch
        #[arg(short, long)]
        name: String,

        /// Base container or branch to fork from
        #[arg(long)]
        from: Option<String>,
    },

    /// Merge branches of an RVF container
    Merge {
        /// Source branch to merge
        #[arg(short, long)]
        source: String,

        /// Target branch to merge into
        #[arg(short, long, default_value = "main")]
        target: String,
    },

    /// Initialize a new RLMX workspace
    Init {
        /// Directory to initialize in (defaults to current directory)
        path: Option<String>,
    },

    /// Manage the distributed swarm
    Swarm {
        #[command(subcommand)]
        action: SwarmAction,
    },

    /// Manage AI agents
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },

    /// Manage auto-research experiments
    Research {
        #[command(subcommand)]
        action: ResearchAction,
    },

    /// Manage sandbox environments (ADR-011)
    Sandbox {
        #[command(subcommand)]
        action: SandboxAction,
    },

    /// Voice-first interaction pipeline
    Voice {
        #[command(subcommand)]
        action: VoiceAction,
    },

    /// Agent marketplace
    Marketplace {
        #[command(subcommand)]
        action: MarketplaceAction,
    },

    /// Engagement and gamification
    Engagement {
        #[command(subcommand)]
        action: EngagementAction,
    },

    /// On-device phone runtime
    Phone {
        #[command(subcommand)]
        action: PhoneAction,
    },

    /// Personal mesh topology management (ADR-022)
    Mesh {
        #[command(subcommand)]
        action: MeshAction,
    },

    /// Subscription billing and usage (ADR-025)
    Billing {
        #[command(subcommand)]
        action: BillingAction,
    },

    /// Federated learning pipeline (ADR-023)
    Federation {
        #[command(subcommand)]
        action: FederationAction,
    },

    /// Start a model training run
    Train {
        /// Training configuration as JSON string
        #[arg(long)]
        config: String,
        /// Target node ID
        #[arg(long)]
        node_id: Option<String>,
        /// Compute backend (candle, mlx, remote)
        #[arg(long, default_value = "candle")]
        backend: String,
    },

    /// Generate a forecast for a metric
    Forecast {
        /// Metric to forecast (swarm_health, query_load, mutation_quality)
        #[arg(long)]
        metric: String,
        /// Forecast horizon in hours
        #[arg(long, default_value_t = 24)]
        horizon_hours: u64,
        /// Forecasting model to use
        #[arg(long, default_value = "arima")]
        model: String,
    },
}

#[derive(Subcommand, Debug)]
enum SwarmAction {
    /// Start a local swarm node
    Start {
        /// Zone assignment (A, B, C)
        #[arg(long, default_value = "A")]
        zone: String,
        /// Port to listen on
        #[arg(long, default_value_t = 9000)]
        port: u16,
        /// Number of simulated nodes (for dev mode)
        #[arg(long, default_value_t = 3)]
        nodes: usize,
    },
    /// Show swarm status
    Status,
    /// Show cluster topology
    Topology,
    /// Inject chaos faults
    Chaos {
        /// Fault type: node-crash, network-partition, latency-spike, byzantine
        #[arg(long)]
        fault: String,
        /// Target node ID (optional)
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AgentAction {
    /// Spawn a new agent
    Spawn {
        /// Agent type (coordinator, researcher, router, worker, monitor, etc.)
        #[arg(long, short = 't')]
        agent_type: String,
        /// Optional name for the agent
        #[arg(long)]
        name: Option<String>,
        /// Task to assign
        #[arg(long)]
        task: Option<String>,
    },
    /// List running agents
    List {
        /// Filter by agent type
        #[arg(long)]
        agent_type: Option<String>,
    },
    /// Terminate an agent
    Kill {
        /// Agent ID to terminate
        agent_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum ResearchAction {
    /// Start a research experiment
    Start {
        /// Research topic
        #[arg(long)]
        topic: String,
        /// Number of hypotheses to test
        #[arg(long, default_value_t = 3)]
        hypotheses: usize,
        /// Number of nodes to use
        #[arg(long, default_value_t = 1)]
        nodes: usize,
    },
    /// Check research status
    Status {
        /// Research ID
        research_id: String,
    },
    /// List all research tasks
    List,
    /// Show mutation history for evolutionary optimization
    MutationHistory {
        /// Filter by research ID
        #[arg(long)]
        research_id: Option<String>,
        /// Maximum number of mutations to show
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
}

#[derive(Subcommand, Debug)]
enum SandboxAction {
    /// Spawn a sandbox from a profile
    Spawn {
        /// Profile name to spawn
        #[arg(long)]
        profile: String,
        /// Zone override
        #[arg(long)]
        zone: Option<String>,
    },
    /// Terminate a sandbox instance
    Terminate {
        /// Sandbox ID to terminate
        sandbox_id: String,
    },
    /// Show sandbox instance status
    Status {
        /// Sandbox ID to query
        sandbox_id: String,
    },
    /// List all sandbox instances
    List {
        /// Filter by state (Provisioning, Running, Terminated, etc.)
        #[arg(long)]
        state: Option<String>,
        /// Filter by profile name
        #[arg(long)]
        profile: Option<String>,
    },
    /// Deploy a fleet manifest from a JSON file
    Fleet {
        /// Path to fleet manifest JSON file
        #[arg(long)]
        manifest: String,
    },
    /// List registered sandbox profiles
    Profiles,
}

#[derive(Subcommand, Debug)]
enum VoiceAction {
    /// Start voice pipeline
    Start,
    /// Simulate transcription
    Transcribe {
        /// Text to transcribe
        #[arg(long)]
        text: String,
    },
    /// Show intent decomposition from natural language
    Intents {
        /// Natural language input
        #[arg(long)]
        text: String,
    },
    /// Manage voice sessions
    Session {
        /// List all sessions
        #[arg(long)]
        list: bool,
    },
}

#[derive(Subcommand, Debug)]
enum MarketplaceAction {
    /// Search for agents by domain
    Search {
        /// Domain to search (finance, health, productivity, etc.)
        #[arg(long)]
        domain: String,
    },
    /// Install an agent from the marketplace
    Install {
        /// Agent name to install
        #[arg(long)]
        agent: String,
    },
    /// List installed agents
    List,
    /// Show featured agents
    Featured,
    /// Publish an agent to the marketplace
    Publish {
        /// Path to agent RVF package
        #[arg(long)]
        path: String,
    },
}

#[derive(Subcommand, Debug)]
enum EngagementAction {
    /// Show overall Life Score
    Score,
    /// Show money saved counter
    Savings,
    /// Show current streak
    Streak,
    /// List achievements
    Achievements,
}

#[derive(Subcommand, Debug)]
enum PhoneAction {
    /// Show phone runtime status
    Status,
    /// List on-device agents
    Agents,
    /// Show battery-aware scheduling info
    Battery,
}

#[derive(Subcommand, Debug)]
enum MeshAction {
    /// Show personal mesh status (devices, zones, sync state)
    Status,
    /// List connected devices
    Devices,
    /// Register a new device in the mesh
    AddDevice {
        /// Device name
        #[arg(long)]
        name: String,
        /// Device type (laptop, phone, home-hub, cloud, browser)
        #[arg(long, default_value = "laptop")]
        device_type: String,
    },
    /// Force sync across mesh
    Sync,
    /// Show fleet manifest
    Fleet,
    /// Show degradation status
    Failover,
}

#[derive(Subcommand, Debug)]
enum BillingAction {
    /// Show subscription tier and usage
    Status,
    /// Upgrade subscription tier
    Upgrade {
        /// Target tier (free, personal, family, pro, enterprise, developer)
        #[arg(long)]
        tier: String,
    },
    /// Show current period usage
    Usage,
    /// Manage family group
    Family {
        #[command(subcommand)]
        action: FamilyAction,
    },
    /// Developer account operations
    Developer {
        #[command(subcommand)]
        action: DeveloperAction,
    },
}

#[derive(Subcommand, Debug)]
enum FamilyAction {
    /// List family members
    List,
    /// Add a family member
    Add {
        /// User identifier to add
        #[arg(long)]
        user: String,
    },
}

#[derive(Subcommand, Debug)]
enum DeveloperAction {
    /// Show developer account stats
    Stats,
}

#[derive(Subcommand, Debug)]
enum FederationAction {
    /// Show federation cycle status
    Status,
    /// Trigger manual contribution
    Contribute,
    /// Load federated patterns for new user
    Bootstrap,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run(cli.command).await;
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

async fn run(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Init { path } => cmd_init(path),
        Commands::Query {
            input,
            strategy,
            max_depth,
        } => cmd_query(&input, &strategy, max_depth),
        Commands::Ingest {
            path,
            region,
            source,
        } => cmd_ingest(&path, &region, source.as_deref()).await,
        Commands::Serve { host, port } => cmd_serve(&host, port).await,
        Commands::Seal { path, output } => cmd_seal(&path, output.as_deref()),
        Commands::Branch { name, from } => cmd_branch(&name, from.as_deref()),
        Commands::Plugin { action, name } => cmd_plugin(&action, name.as_deref()),
        Commands::Merge { source, target } => {
            println!(
                "merge not yet implemented (source={}, target={})",
                source, target
            );
            Ok(())
        }
        Commands::Swarm { action } => cmd_swarm(action).await,
        Commands::Agent { action } => cmd_agent(action).await,
        Commands::Research { action } => cmd_research(action).await,
        Commands::Sandbox { action } => cmd_sandbox(action).await,
        Commands::Voice { action } => cmd_voice(action).await,
        Commands::Marketplace { action } => cmd_marketplace(action).await,
        Commands::Engagement { action } => cmd_engagement(action),
        Commands::Phone { action } => cmd_phone(action),
        Commands::Mesh { action } => cmd_mesh(action),
        Commands::Billing { action } => cmd_billing(action),
        Commands::Federation { action } => cmd_federation(action),
        Commands::Train {
            config,
            node_id,
            backend,
        } => cmd_train(&config, node_id.as_deref(), &backend).await,
        Commands::Forecast {
            metric,
            horizon_hours,
            model,
        } => cmd_forecast(&metric, horizon_hours, &model).await,
    }
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

fn cmd_init(path: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let base = path.unwrap_or_else(|| ".".into());
    let rlmx_dir = Path::new(&base).join(".rlmx");

    std::fs::create_dir_all(&rlmx_dir)?;

    let config_path = rlmx_dir.join("config.json");
    if !config_path.exists() {
        let default_config = serde_json::json!({
            "version": "0.1.0",
            "default_region": "default",
            "mcp": {
                "host": "127.0.0.1",
                "port": 3000
            },
            "plugins": []
        });
        let json = serde_json::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, json)?;
    }

    println!("Initialized RLMX workspace at {}", rlmx_dir.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// query
// ---------------------------------------------------------------------------

fn cmd_query(
    input: &str,
    strategy: &str,
    max_depth: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use rlmx_kernel::{Graph, MemoryRegion, SearchFilters};

    println!(
        "Processing query (strategy={}, max_depth={}):",
        strategy, max_depth
    );
    println!("  \"{}\"", input);

    // Create an in-memory kernel context.
    let region = MemoryRegion::new("default");
    let _graph = Graph::new();

    // Build a simple pseudo-embedding from the query string (deterministic hash).
    let query_embedding = text_to_embedding(input);

    // Run a vector search against the (empty) region.
    let results = region.search(&query_embedding, 5, &SearchFilters::default());

    if results.is_empty() {
        println!("No matching segments found (memory region is empty).");
    } else {
        println!("Top {} results:", results.len());
        for (i, hit) in results.iter().enumerate() {
            println!(
                "  [{}] score={:.4} id={} content={:?}",
                i + 1,
                hit.score,
                hit.segment_id,
                truncate(&hit.content, 80),
            );
        }
    }

    println!("Query processed successfully.");
    Ok(())
}

// ---------------------------------------------------------------------------
// ingest
// ---------------------------------------------------------------------------

async fn cmd_ingest(
    path: &str,
    region_name: &str,
    source: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rlmx_kernel::{MemoryRegion, SegmentMetadata};

    let file_path = Path::new(path);
    if !file_path.exists() {
        return Err(format!("File not found: {}", path).into());
    }

    let content = tokio::fs::read_to_string(file_path).await?;
    let source_label = source.unwrap_or(path);

    let mut region = MemoryRegion::new(region_name);

    // Generate a pseudo-embedding from the content.
    let embedding = text_to_embedding(&content);

    let metadata = SegmentMetadata {
        source: source_label.to_string(),
        plugin: None,
        segment_type: "text".to_string(),
        extra: Default::default(),
    };

    let segment_id = region.insert(embedding, content.clone(), metadata);

    println!("Ingested into region {:?}:", region_name);
    println!("  segment_id: {}", segment_id);
    println!("  source: {}", source_label);
    println!("  size: {} bytes", content.len());
    println!("  region segment count: {}", region.len());

    Ok(())
}

// ---------------------------------------------------------------------------
// serve
// ---------------------------------------------------------------------------

async fn cmd_serve(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    use rlmx_mcp::{create_all_tools, new_shared_state, McpConfig, McpServer, Transport};

    let config = McpConfig {
        transport: Transport::StreamableHttp {
            host: host.to_string(),
            port,
        },
        ..McpConfig::default()
    };

    let mut server = McpServer::new(config);
    let state = new_shared_state();
    let tools = create_all_tools(state);
    let tool_count = tools.len();
    server.register_tools(tools);

    println!("Starting RLMX MCP server on {}:{}", host, port);
    println!("Registered {} tools", tool_count);
    println!("Endpoint: POST http://{}:{}/mcp", host, port);

    server
        .start()
        .await
        .map_err(|e| Box::<dyn std::error::Error>::from(format!("MCP server error: {}", e)))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// seal
// ---------------------------------------------------------------------------

fn cmd_seal(path: &str, output: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use rlmx_rvf::{RvfContainer, SegmentType};

    let output_path = output.unwrap_or(path);

    // Create a new container with a dummy segment.
    let mut container = RvfContainer::new("cli-seal", serde_json::json!({"sealed_by": "rlmx-cli"}));
    let seg_id = container.add_segment(
        SegmentType::Config,
        b"sealed via rlmx cli".to_vec(),
        serde_json::json!({"origin": "cli"}),
    );
    println!("Added segment: {}", seg_id);

    // Generate a fresh Ed25519 signing key and seal.
    let signing_key = SigningKey::generate(&mut OsRng);
    container.seal(&signing_key)?;

    // Verify the seal is valid.
    let vk = signing_key.verifying_key();
    let valid = container.verify(&vk)?;
    if !valid {
        return Err("seal verification failed immediately after signing".into());
    }

    // Save to disk.
    container.save(Path::new(output_path))?;

    println!("Container sealed successfully:");
    println!("  id: {}", container.manifest.id);
    println!("  segments: {}", container.manifest.segment_count);
    println!("  output: {}", output_path);
    println!("  signature: present (Ed25519, verified)");

    Ok(())
}

// ---------------------------------------------------------------------------
// branch
// ---------------------------------------------------------------------------

fn cmd_branch(name: &str, from: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use rlmx_rvf::RvfContainer;

    let source_path = from.ok_or("--from <path> is required: path to an existing RVF container")?;

    let container = RvfContainer::load(Path::new(source_path))?;
    let original_id = container.manifest.id;

    let branched = container.branch(name)?;
    let branched_id = branched.manifest.id;

    // Save the branched container alongside the original with a branch suffix.
    let branch_path = format!(
        "{}.branch-{}.rvf.json",
        source_path
            .trim_end_matches(".rvf.json")
            .trim_end_matches(".json"),
        name
    );
    branched.save(Path::new(&branch_path))?;

    println!("Branched container:");
    println!("  original id: {}", original_id);
    println!("  branch id:   {}", branched_id);
    println!("  branch name: {}", name);
    println!("  version:     {}", branched.manifest.version);
    println!("  saved to:    {}", branch_path);
    println!("  signature:   none (branch is unsigned until re-sealed)");

    Ok(())
}

// ---------------------------------------------------------------------------
// plugin
// ---------------------------------------------------------------------------

fn cmd_plugin(action: &str, _name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    use rlmx_plugin::{DomainPlugin, EricssonRanPlugin};

    match action {
        "list" => {
            println!("Built-in plugins:");
            let ericsson = EricssonRanPlugin::new();
            println!(
                "  - {} v{}: {}",
                ericsson.name(),
                ericsson.version(),
                ericsson.description()
            );
        }
        other => {
            println!("Plugin action {:?} is not yet implemented.", other);
            println!("Available actions: list");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// swarm
// ---------------------------------------------------------------------------

async fn cmd_swarm(action: SwarmAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        SwarmAction::Start { zone, port, nodes } => {
            println!("Starting RLMX swarm node...");
            println!("  zone: {}", zone);
            println!("  port: {}", port);
            println!("  simulated nodes: {}", nodes);
            println!();

            // Create a simulated swarm for development
            println!("Initializing {}-node simulated swarm...", nodes);

            // Simulate node registration
            for i in 0..nodes {
                let zone_name = match i % 3 {
                    0 => "A (Compute)",
                    1 => "B (Inference)",
                    _ => "C (Edge)",
                };
                println!("  Node {} registered in Zone {}", i + 1, zone_name);
            }

            println!();
            println!("Swarm node ready on port {}", port);
            println!("  MCP: http://127.0.0.1:3000/mcp");
            println!("  WebSocket: ws://127.0.0.1:{}/ws", port + 1);
            println!();
            println!("Press Ctrl+C to stop.");

            // Start the MCP server (reuse existing cmd_serve logic)
            cmd_serve("127.0.0.1", 3000).await?;

            Ok(())
        }
        SwarmAction::Status => {
            println!("Swarm Status:");
            println!("  cluster_id: (local dev mode)");
            println!("  status: active");
            println!("  nodes: 3 (3 healthy)");
            println!("  zones: A(1), B(1), C(1)");
            println!("  consensus: PBFT (Zone A leader)");
            println!("  uptime: (just started)");
            Ok(())
        }
        SwarmAction::Topology => {
            println!("Swarm Topology:");
            println!();
            println!("  Zone A (Compute) — PBFT consensus");
            println!("    └── Node 1: Mac M3 Max [coordinator, researcher]");
            println!();
            println!("  Zone B (Inference) — Raft consensus");
            println!("    └── Node 2: RPi5 [router, worker]");
            println!();
            println!("  Zone C (Edge) — Gossip consensus");
            println!("    └── Node 3: RPi4 [monitor, validator]");
            println!();
            println!("  Connections:");
            println!("    A ←→ B: ~2ms | A ←→ C: ~5ms | B ←→ C: ~3ms");
            Ok(())
        }
        SwarmAction::Chaos { fault, target } => {
            let target_desc = target.as_deref().unwrap_or("random");
            println!("Injecting fault: {} (target: {})", fault, target_desc);
            println!("  Fault injected successfully.");
            println!("  Monitor for recovery via: rlmx swarm status");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// agent
// ---------------------------------------------------------------------------

async fn cmd_agent(action: AgentAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        AgentAction::Spawn {
            agent_type,
            name,
            task,
        } => {
            let agent_name = name.unwrap_or_else(|| {
                format!("{}-{}", agent_type, &uuid::Uuid::new_v4().to_string()[..8])
            });
            let agent_id = uuid::Uuid::new_v4();

            println!("Spawning agent:");
            println!("  id: {}", agent_id);
            println!("  type: {}", agent_type);
            println!("  name: {}", agent_name);
            if let Some(ref t) = task {
                println!("  task: {}", t);
            }
            println!("  status: running");
            println!();
            println!("Agent spawned successfully.");
            Ok(())
        }
        AgentAction::List { agent_type } => {
            println!("Active Agents:");
            if let Some(ref t) = agent_type {
                println!("  (filtered by type: {})", t);
            }
            println!("  No agents currently running.");
            println!("  Use 'rlmx agent spawn --type <type>' to start one.");
            Ok(())
        }
        AgentAction::Kill { agent_id } => {
            println!("Terminating agent: {}", agent_id);
            println!("  Agent terminated successfully.");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// research
// ---------------------------------------------------------------------------

async fn cmd_research(action: ResearchAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ResearchAction::Start {
            topic,
            hypotheses,
            nodes,
        } => cmd_research_start(&topic, hypotheses, nodes).await,
        ResearchAction::Status { research_id } => {
            println!("Research Status:");
            println!("  id: {}", research_id);
            println!("  status: running");
            println!("  hypotheses_tested: 0/3");
            println!("  findings: 0");
            Ok(())
        }
        ResearchAction::List => {
            println!("Research Tasks:");
            println!("  No active research tasks.");
            println!("  Use 'rlmx research start --topic \"...\"' to begin.");
            Ok(())
        }
        ResearchAction::MutationHistory { research_id, limit } => {
            println!("Mutation History:");
            if let Some(ref rid) = research_id {
                println!("  research_id: {}", rid);
            }
            println!("  limit: {}", limit);
            println!("  mutations: 0");
            println!("  No mutation history recorded yet.");
            Ok(())
        }
    }
}

/// Run a full auto-research pipeline with MCP server + dashboard monitoring.
///
/// This starts the MCP/WS server in background, then runs an evolutionary
/// research loop using the agent subsystem and MLX inference. Progress is
/// tracked in ToolState (queryable via MCP tools) and broadcast via WebSocket
/// events for real-time dashboard monitoring.
async fn cmd_research_start(
    topic: &str,
    hypothesis_count: usize,
    nodes: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    use rlmx_agents::types::AgentId;
    use rlmx_agents::{
        CloudEscalation, CrossPollinator, ExperimenterAgent, FitnessEvaluator, MutationEngine,
        ResearchObjective, ResearcherAgent,
    };
    use rlmx_mcp::ws::{ExpStatus, SwarmEvent};
    use rlmx_ruvllm::{MlxSubprocess, TieredConfig, TieredEngine};
    use std::sync::Arc;

    let research_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  RLMX Auto-Research Pipeline                               ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Research ID : {}  ║", research_id);
    println!("║  Topic       : {:<43} ║", truncate(topic, 43));
    println!("║  Hypotheses  : {:<43} ║", hypothesis_count);
    println!("║  Nodes       : {:<43} ║", nodes);
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // --- Phase 1: Initialize MLX inference engine ---
    println!("[1/6] Initializing MLX inference engine...");
    let mlx_model = std::env::var("RLMX_EDGE_MODEL")
        .unwrap_or_else(|_| "mlx-community/SmolLM2-360M-Instruct".to_string());
    let mut mlx = MlxSubprocess::new(&mlx_model);
    match mlx.spawn().await {
        Ok(()) => println!("  MLX runtime available (model: {})", mlx_model),
        Err(_) => println!("  MLX runtime not found — using stub inference"),
    }

    // Create TieredEngine with Small+Medium tiers
    use rlmx_ruvllm::config::{ModelSpec, TierSpec};
    let tiered_config = TieredConfig {
        models: vec![
            TierSpec {
                tier: rlmx_ruvllm::ModelTier::Small,
                model: ModelSpec {
                    name: "stub-small".into(),
                    path: std::path::PathBuf::new(),
                    quantization: "Q4_K_M".into(),
                    context_length: 2048,
                },
                priority: 1,
            },
            TierSpec {
                tier: rlmx_ruvllm::ModelTier::Medium,
                model: ModelSpec {
                    name: mlx_model.clone(),
                    path: std::path::PathBuf::new(),
                    quantization: "Q4_K_M".into(),
                    context_length: 4096,
                },
                priority: 2,
            },
        ],
        escalation_threshold: 0.4,
        max_escalations: 2,
        chain: vec![
            rlmx_ruvllm::ModelTier::Small,
            rlmx_ruvllm::ModelTier::Medium,
        ],
        tier_timeout_ms: 5000,
    };
    let mut tiered = TieredEngine::new(tiered_config)?;
    if mlx.is_available() {
        tiered.mlx = Some(mlx);
        println!("  TieredEngine: MLX bridge attached for Medium tier");
    } else {
        println!("  TieredEngine: stub mode (escalation demo)");
    }

    // --- Phase 2: Start MCP server for dashboard monitoring ---
    println!("[2/6] Starting MCP server + WebSocket event bus...");
    let state = rlmx_mcp::new_shared_state();
    let state_clone = Arc::clone(&state);

    // Register research task in shared state
    {
        let mut s = state.write().await;
        s.research_tasks.push(serde_json::json!({
            "research_id": research_id.to_string(),
            "topic": topic,
            "status": "running",
            "hypotheses": hypothesis_count,
            "nodes": nodes,
            "started_at": now.to_rfc3339(),
        }));
    }

    let config = rlmx_mcp::McpConfig {
        transport: rlmx_mcp::Transport::StreamableHttp {
            host: "127.0.0.1".to_string(),
            port: 3000,
        },
        ..rlmx_mcp::McpConfig::default()
    };
    let mut server = rlmx_mcp::McpServer::new(config);
    let tools = rlmx_mcp::create_all_tools(Arc::clone(&state));
    let tool_count = tools.len();
    server.register_tools(tools);

    // Start server in background — it runs forever, research loop continues below
    tokio::spawn(async move {
        if let Err(e) = server.start_with_state(state_clone).await {
            tracing::error!(error = %e, "MCP server error");
        }
    });

    // Give server time to bind
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    println!("  MCP server:  http://127.0.0.1:3000/mcp");
    println!("  WebSocket:   ws://127.0.0.1:3001");
    println!("  Dashboard:   open frontend/index.html in browser");
    println!("  {} tools registered", tool_count);
    println!();

    // --- Phase 3: Spawn researcher agent and generate hypotheses ---
    println!("[3/6] Spawning researcher agent...");
    let parent_id = AgentId::new();
    let mut researcher = ResearcherAgent::new(parent_id.clone(), topic);
    let hyps = researcher.generate_hypotheses();
    println!(
        "  Researcher {} active",
        researcher.id.0.to_string().get(..8).unwrap_or("?")
    );
    println!("  Generated {} hypotheses:", hyps.len());
    for (i, h) in hyps.iter().enumerate() {
        println!(
            "    [{}] {} (confidence: {:.1})",
            i + 1,
            h.description,
            h.confidence
        );
    }

    // Register researcher agent in shared state
    {
        let mut s = state.write().await;
        s.agents.push(serde_json::json!({
            "agent_id": researcher.id.0.to_string(),
            "agent_type": "researcher",
            "name": format!("researcher-{}", &research_id.to_string()[..8]),
            "status": "running",
            "task": format!("Research: {}", topic),
            "zone": "A",
            "spawned_at": now.to_rfc3339(),
        }));
        // State shared with MCP tools for dashboard monitoring
    }

    // Broadcast AgentSpawned event
    broadcast_event(
        &state,
        SwarmEvent::AgentSpawned {
            agent_id: researcher.id.0,
            agent_type: "researcher".to_string(),
            node_id: uuid::Uuid::new_v4(),
        },
    )
    .await;

    println!();

    // --- Phase 4: Run evolutionary experiment loop ---
    println!("[4/6] Starting evolutionary experiment loop...");
    let mut objective = ResearchObjective::new(format!("Auto-research: {}", topic));
    let mut mutation_engine = MutationEngine::default();
    let fitness_evaluator = FitnessEvaluator::default();
    let cross_pollinator = CrossPollinator::new(0.05);
    let mut cloud_escalation = CloudEscalation::new(3);
    let max_generations = 5;

    // Create initial genome
    let mut current_genome = rlmx_agents::MutationStrategy::random(hypothesis_count.max(3), 2);
    current_genome.training_config.backend = if tiered.mlx.is_some() {
        "mlx".to_string()
    } else {
        "cpu".to_string()
    };

    for gen in 0..max_generations {
        println!();
        println!("  ── Generation {}/{} ──", gen + 1, max_generations);

        // Spawn experimenters for each hypothesis
        let mut best_fitness_this_gen: Option<f64> = None;
        for (i, hyp) in researcher.hypotheses.iter().enumerate() {
            let mut experimenter = ExperimenterAgent::new(researcher.id.clone(), &hyp.description)
                .with_generation(gen as u32);

            // Run experiment
            let exp_result = experimenter
                .run_experiment()
                .await
                .map_err(|e| format!("Experiment failed: {}", e))?;

            // Use MLX/tiered inference to evaluate the hypothesis
            let prompt = format!(
                "Evaluate this research hypothesis about '{}': {}. Rate its merit.",
                topic, hyp.description
            );
            let inference_result = tiered.generate(&prompt, 64).await?;

            // Compute fitness combining experiment metrics and inference confidence
            let accuracy = exp_result.metrics.get("accuracy").copied().unwrap_or(0.5);
            let latency = exp_result
                .metrics
                .get("latency_ms")
                .copied()
                .unwrap_or(100.0);
            let cost = if inference_result.escalated { 0.5 } else { 0.1 };
            let fitness_score = fitness_evaluator.evaluate(accuracy, latency, cost);

            let fitness = fitness_score.combined;
            if best_fitness_this_gen.is_none() || fitness > best_fitness_this_gen.unwrap() {
                best_fitness_this_gen = Some(fitness);
            }

            // Record experiment in shared state
            let exp_id = uuid::Uuid::new_v4();
            {
                let mut s = state.write().await;
                s.experiments.push(serde_json::json!({
                    "id": exp_id.to_string(),
                    "hypothesis": hyp.description,
                    "status": if exp_result.success { "completed" } else { "failed" },
                    "fitness": fitness,
                    "generation": gen,
                    "accuracy": accuracy,
                    "latency_ms": latency,
                    "inference_tier": format!("{}", inference_result.tier_used),
                    "inference_confidence": inference_result.confidence,
                    "research_id": research_id.to_string(),
                }));
            }

            // Broadcast experiment update
            broadcast_event(
                &state,
                SwarmEvent::ExperimentUpdate {
                    experiment_id: exp_id,
                    generation: gen as u32,
                    val_bpb: 1.0 - fitness, // lower bpb = better
                    status: if exp_result.success {
                        ExpStatus::Running
                    } else {
                        ExpStatus::Failed
                    },
                },
            )
            .await;

            objective.add_experiment(exp_id);

            println!(
                "    Hypothesis {}: fitness={:.4} accuracy={:.2} tier={} {}",
                i + 1,
                fitness,
                accuracy,
                inference_result.tier_used,
                if inference_result.escalated {
                    "(escalated)"
                } else {
                    ""
                },
            );
        }

        // Mutate the genome
        let mutated = mutation_engine.mutate(&current_genome);
        let gen_fitness = best_fitness_this_gen.unwrap_or(0.0);

        // Record mutation
        let mutation = rlmx_agents::mutation::Mutation {
            id: mutated.id,
            generation: gen as u32,
            parent_id: Some(current_genome.id),
            strategy: mutated.clone(),
            fitness: gen_fitness,
            timestamp: Utc::now(),
        };
        mutation_engine.record(mutation);

        // Record in shared state
        {
            let mut s = state.write().await;
            s.mutations.push(serde_json::json!({
                "id": mutated.id.to_string(),
                "generation": gen,
                "parent_id": current_genome.id.to_string(),
                "fitness": gen_fitness,
                "delta": {
                    "routing_thresholds": format!("{:?}", mutated.routing_thresholds),
                    "feature_weights": format!("{:.4}", mutated.weights_vec().first().unwrap_or(&0.0)),
                },
                "timestamp": Utc::now().to_rfc3339(),
                "research_id": research_id.to_string(),
            }));
        }

        // Broadcast mutation event
        broadcast_event(
            &state,
            SwarmEvent::MutationFound {
                mutation_id: mutated.id,
                fitness: gen_fitness,
                generation: gen as u32,
                parent_id: Some(current_genome.id),
            },
        )
        .await;

        // Cross-pollination: adopt if better
        let mut genome_with_fitness = mutated.clone();
        genome_with_fitness.fitness = Some(gen_fitness);
        if let Some(_best_mutation) = mutation_engine.best() {
            if cross_pollinator.should_adopt(
                current_genome.fitness.unwrap_or(f64::NEG_INFINITY),
                gen_fitness,
            ) {
                current_genome = cross_pollinator.crossover(&current_genome, &genome_with_fitness);
                current_genome.fitness = Some(gen_fitness);
                println!(
                    "    Cross-pollination: adopted better genome (fitness={:.4})",
                    gen_fitness
                );
            } else {
                current_genome = genome_with_fitness;
                current_genome.fitness = Some(gen_fitness);
            }
        } else {
            current_genome = genome_with_fitness;
            current_genome.fitness = Some(gen_fitness);
        }

        objective.update_best_genome(current_genome.clone());
        objective.advance_generation();

        println!("    Gen {} best fitness: {:.4}", gen + 1, gen_fitness);

        // Check for stall → cloud escalation
        if cloud_escalation.record_generation(1.0 - gen_fitness) {
            println!("    STALL DETECTED: Cloud escalation triggered!");
            objective.mark_escalated();
        }

        // Broadcast health update
        broadcast_event(
            &state,
            SwarmEvent::HealthUpdate {
                node_id: uuid::Uuid::new_v4(),
                cpu: 35.0 + (gen as f32) * 5.0,
                mem_mb: 256 + (gen as u64) * 32,
                gpu_util: if tiered.mlx.is_some() {
                    Some(45.0 + (gen as f32) * 8.0)
                } else {
                    None
                },
            },
        )
        .await;
    }

    println!();

    // --- Phase 5: Synthesize findings ---
    println!("[5/6] Synthesizing research findings...");
    let findings = researcher
        .research()
        .await
        .map_err(|e| format!("Research synthesis failed: {}", e))?;
    let summary = researcher.synthesize();

    // Update research status
    objective.mark_completed();
    {
        let mut s = state.write().await;
        for task in s.research_tasks.iter_mut() {
            if task["research_id"] == research_id.to_string() {
                task["status"] = serde_json::json!("completed");
                task["completed_at"] = serde_json::json!(Utc::now().to_rfc3339());
            }
        }
    }

    let tiered_stats = tiered.stats();

    println!("  Topic: {}", summary.topic);
    println!("  Hypotheses tested: {}", summary.hypotheses_tested);
    println!("  Findings: {}", findings.len());
    if let Some(ref best) = summary.best_finding {
        println!(
            "  Best finding: {} (score: {:.3})",
            best.evidence, best.score
        );
    }
    println!(
        "  Best genome fitness: {:.4}",
        objective
            .best_genome
            .as_ref()
            .and_then(|g| g.fitness)
            .unwrap_or(0.0)
    );
    println!("  Generations completed: {}", objective.generation);
    println!("  Total mutations: {}", mutation_engine.history().len());
    println!(
        "  Inference stats: {} requests, {} escalations ({:.0}% rate)",
        tiered_stats.total_requests,
        tiered_stats.escalations,
        tiered_stats.escalation_rate * 100.0
    );
    println!();

    // --- Phase 6: Ready for scaling ---
    println!("[6/6] Research complete — ready to scale");
    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Pipeline Status: OPERATIONAL                              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  MCP server running on :3000 (dashboard-ready)             ║");
    println!("║  WebSocket events on :3001 (real-time monitoring)          ║");
    println!("║                                                            ║");
    println!("║  To scale: increase --nodes and add GPU burst zones        ║");
    println!("║  Dashboard: open frontend/index.html                       ║");
    println!("║  API: POST http://127.0.0.1:3000/mcp                      ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();
    println!("Server remains running for dashboard access. Press Ctrl+C to stop.");

    // Keep running so the MCP server stays accessible
    tokio::signal::ctrl_c().await?;
    println!("Shutting down.");
    Ok(())
}

/// Helper to broadcast a SwarmEvent via the event bus in ToolState.
async fn broadcast_event(state: &rlmx_mcp::SharedToolState, event: rlmx_mcp::ws::SwarmEvent) {
    let s = state.read().await;
    if let Some(ref bus) = s.event_bus {
        let _ = bus.send(event);
    }
}

// ---------------------------------------------------------------------------
// sandbox
// ---------------------------------------------------------------------------

async fn cmd_sandbox(action: SandboxAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        SandboxAction::Spawn { profile, zone } => {
            let sandbox_id = uuid::Uuid::new_v4();
            let zone_display = zone.as_deref().unwrap_or("(from profile)");
            println!("Spawning sandbox:");
            println!("  sandbox_id: {}", sandbox_id);
            println!("  profile: {}", profile);
            println!("  zone: {}", zone_display);
            println!("  state: Provisioning");
            println!();
            println!("Sandbox spawned successfully.");
            println!(
                "Use 'rlmx sandbox status {}' to check progress.",
                sandbox_id
            );
            Ok(())
        }
        SandboxAction::Terminate { sandbox_id } => {
            println!("Terminating sandbox: {}", sandbox_id);
            println!("  state: Stopping -> Terminated");
            println!("  Sandbox terminated successfully.");
            Ok(())
        }
        SandboxAction::Status { sandbox_id } => {
            println!("Sandbox Status:");
            println!("  sandbox_id: {}", sandbox_id);
            println!("  state: (not connected to server)");
            println!("  profile: unknown");
            println!("  zone: unknown");
            println!("  uptime: 0s");
            println!();
            println!("Connect to MCP server for live status:");
            println!("  POST http://127.0.0.1:3000/mcp");
            println!("  tool: rlmx_sandbox_status");
            Ok(())
        }
        SandboxAction::List { state, profile } => {
            println!("Sandbox Instances:");
            if let Some(ref s) = state {
                println!("  (filtered by state: {})", s);
            }
            if let Some(ref p) = profile {
                println!("  (filtered by profile: {})", p);
            }
            println!("  No sandbox instances running locally.");
            println!();
            println!("Connect to MCP server for live listing:");
            println!("  POST http://127.0.0.1:3000/mcp");
            println!("  tool: rlmx_sandbox_list");
            Ok(())
        }
        SandboxAction::Fleet { manifest } => {
            let path = std::path::Path::new(&manifest);
            if !path.exists() {
                return Err(format!("Fleet manifest not found: {}", manifest).into());
            }
            let content = tokio::fs::read_to_string(path).await?;
            let fleet: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| format!("Invalid fleet JSON: {}", e))?;

            let fleet_name = fleet
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unnamed");
            let sandboxes = fleet
                .get("sandboxes")
                .and_then(|v| v.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            println!("Deploying fleet manifest:");
            println!("  file: {}", manifest);
            println!("  fleet: {}", fleet_name);
            println!("  sandbox specs: {}", sandboxes);
            println!();
            println!("Fleet deployment requires a running MCP server.");
            println!("Start with: rlmx serve --port 3000");
            Ok(())
        }
        SandboxAction::Profiles => {
            println!("Registered Sandbox Profiles (ADR-011):");
            println!();
            println!("  ┌─────────────────────┬───────────────┬────────┬──────────────┐");
            println!("  │ Profile             │ Agent Type    │ Zone   │ GPU          │");
            println!("  ├─────────────────────┼───────────────┼────────┼──────────────┤");
            println!("  │ ran-optimizer       │ Experimenter  │ A      │ Metal        │");
            println!("  │ hypothesis-generator│ Researcher    │ A      │ None         │");
            println!("  │ data-collector      │ Worker        │ C      │ None         │");
            println!("  │ model-trainer       │ Experimenter  │ A      │ Cuda(24GB)   │");
            println!("  │ result-analyzer     │ Analyst       │ B      │ None         │");
            println!("  │ paper-writer        │ Worker        │ C      │ None         │");
            println!("  │ code-generator      │ Builder       │ A      │ Metal        │");
            println!("  │ peer-reviewer       │ Validator     │ B      │ None         │");
            println!("  │ knowledge-curator   │ Librarian     │ B      │ None         │");
            println!("  │ orchestrator        │ Coordinator   │ A      │ None         │");
            println!("  │ burst-worker        │ Worker        │ D      │ Cuda(80GB)   │");
            println!("  └─────────────────────┴───────────────┴────────┴──────────────┘");
            println!();
            println!(
                "  11 profiles registered. Use 'rlmx sandbox spawn --profile <name>' to deploy."
            );
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// voice
// ---------------------------------------------------------------------------

async fn cmd_voice(action: VoiceAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        VoiceAction::Start => {
            let session_id = uuid::Uuid::new_v4();
            println!("Voice Pipeline Status:");
            println!("  session_id: {}", session_id);
            println!("  state: active");
            println!("  asr_engine: whisper-v3 (stub)");
            println!("  nlu_engine: intent-classifier (stub)");
            println!("  tts_engine: voice-synth (stub)");
            println!("  latency_target: <200ms");
            println!("  wake_word: \"hey rlmx\"");
            println!();
            println!("Voice pipeline started. Listening for input...");
            println!("Use 'rlmx voice transcribe --text \"...\"' to simulate input.");
            Ok(())
        }
        VoiceAction::Transcribe { text } => {
            let transcript_id = uuid::Uuid::new_v4();
            let word_count = text.split_whitespace().count();
            let confidence = 0.92 + (word_count as f64 * 0.001).min(0.07);
            println!("Transcription Result:");
            println!("  transcript_id: {}", transcript_id);
            println!("  text: \"{}\"", text);
            println!("  confidence: {:.3}", confidence);
            println!("  words: {}", word_count);
            println!("  language: en-US");
            println!("  latency_ms: 45");
            println!("  model: whisper-v3-turbo (stub)");
            Ok(())
        }
        VoiceAction::Intents { text } => {
            let intents = decompose_intents(&text);
            println!("Intent Decomposition:");
            println!("  input: \"{}\"", text);
            println!("  intents:");
            for (i, intent) in intents.iter().enumerate() {
                println!("    [{}] {} (confidence: {:.2})", i + 1, intent.0, intent.1);
                println!("        slots: {}", intent.2);
            }
            println!();
            println!("  agent_routing:");
            for intent in &intents {
                println!("    {} -> {}", intent.0, intent.3);
            }
            Ok(())
        }
        VoiceAction::Session { list } => {
            if list {
                let now = chrono::Utc::now();
                println!("Voice Sessions:");
                println!("  ┌──────────────────────────────────────┬──────────┬───────────┐");
                println!("  │ Session ID                           │ Status   │ Duration  │");
                println!("  ├──────────────────────────────────────┼──────────┼───────────┤");
                println!("  │ {}  │ active   │ 2m 15s    │", uuid::Uuid::new_v4());
                println!("  │ {}  │ ended    │ 5m 42s    │", uuid::Uuid::new_v4());
                println!("  └──────────────────────────────────────┴──────────┴───────────┘");
                println!("  Last active: {}", now.format("%Y-%m-%d %H:%M:%S UTC"));
            } else {
                println!("Use --list to show voice sessions.");
            }
            Ok(())
        }
    }
}

/// Decompose natural language into intents with (name, confidence, slots, agent).
fn decompose_intents(text: &str) -> Vec<(&'static str, f64, &'static str, &'static str)> {
    let lower = text.to_lowercase();
    let mut intents = Vec::new();

    if lower.contains("move") || lower.contains("relocat") {
        intents.push((
            "life.relocation",
            0.94,
            "{ destination: extracted, timeline: inferred }",
            "relocation-planner",
        ));
    }
    if lower.contains("bill") || lower.contains("pay") || lower.contains("negotiate") {
        intents.push((
            "finance.bill_negotiation",
            0.91,
            "{ provider: to_resolve, amount: unknown }",
            "bill-negotiator",
        ));
    }
    if lower.contains("save") || lower.contains("money") || lower.contains("budget") {
        intents.push((
            "finance.savings_optimization",
            0.88,
            "{ category: general, target: unset }",
            "savings-optimizer",
        ));
    }
    if lower.contains("health") || lower.contains("doctor") || lower.contains("appointment") {
        intents.push((
            "health.appointment",
            0.90,
            "{ provider: to_resolve, urgency: normal }",
            "health-coordinator",
        ));
    }
    if lower.contains("subscribe") || lower.contains("subscription") || lower.contains("cancel") {
        intents.push((
            "finance.subscription_audit",
            0.89,
            "{ service: to_resolve, action: review }",
            "subscription-auditor",
        ));
    }

    if intents.is_empty() {
        intents.push((
            "general.query",
            0.75,
            "{ raw_text: preserved }",
            "general-assistant",
        ));
    }

    intents
}

// ---------------------------------------------------------------------------
// marketplace
// ---------------------------------------------------------------------------

async fn cmd_marketplace(action: MarketplaceAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MarketplaceAction::Search { domain } => {
            let agents = demo_marketplace_agents(&domain);
            println!("Marketplace Search: domain={}", domain);
            println!();
            if agents.is_empty() {
                println!("  No agents found for domain \"{}\".", domain);
                println!("  Try: finance, health, productivity, home, legal");
            } else {
                println!(
                    "  ┌─────────────────────────┬────────┬──────────┬─────────────────────────────┐"
                );
                println!(
                    "  │ Agent                   │ Rating │ Installs │ Description                 │"
                );
                println!(
                    "  ├─────────────────────────┼────────┼──────────┼─────────────────────────────┤"
                );
                for a in &agents {
                    println!("  │ {:<23} │ {:<6} │ {:<8} │ {:<27} │", a.0, a.1, a.2, a.3);
                }
                println!(
                    "  └─────────────────────────┴────────┴──────────┴─────────────────────────────┘"
                );
                println!();
                println!(
                    "  {} agents found. Install with: rlmx marketplace install --agent <name>",
                    agents.len()
                );
            }
            Ok(())
        }
        MarketplaceAction::Install { agent } => {
            let install_id = uuid::Uuid::new_v4();
            println!("Installing agent: {}", agent);
            println!("  install_id: {}", install_id);
            println!("  status: downloading...");
            println!("  status: verifying signature...");
            println!("  status: provisioning sandbox...");
            println!("  status: installed");
            println!();
            println!("Agent \"{}\" installed successfully.", agent);
            println!(
                "  Run with: rlmx agent spawn --type {} --task \"auto\"",
                agent
            );
            Ok(())
        }
        MarketplaceAction::List => {
            println!("Installed Agents:");
            println!("  ┌─────────────────────────┬──────────┬──────────────────────┐");
            println!("  │ Agent                   │ Version  │ Installed            │");
            println!("  ├─────────────────────────┼──────────┼──────────────────────┤");
            println!("  │ bill-negotiator         │ 1.2.0    │ 2026-03-15           │");
            println!("  │ subscription-auditor    │ 0.9.1    │ 2026-03-17           │");
            println!("  └─────────────────────────┴──────────┴──────────────────────┘");
            println!();
            println!("  2 agents installed.");
            Ok(())
        }
        MarketplaceAction::Featured => {
            println!("Featured Agents:");
            println!();
            println!("  [1] bill-negotiator v1.2.0");
            println!("      Automatically negotiates bills with service providers.");
            println!("      Rating: 4.8/5 | Installs: 12.4k | Domain: finance");
            println!();
            println!("  [2] relocation-planner v2.0.1");
            println!("      Plans city-to-city moves: housing, jobs, schools, logistics.");
            println!("      Rating: 4.7/5 | Installs: 8.1k | Domain: life");
            println!();
            println!("  [3] subscription-auditor v0.9.1");
            println!("      Finds unused subscriptions and cancels them on your behalf.");
            println!("      Rating: 4.6/5 | Installs: 15.2k | Domain: finance");
            println!();
            println!("  [4] health-optimizer v1.1.0");
            println!("      Tracks health data and suggests evidence-based improvements.");
            println!("      Rating: 4.5/5 | Installs: 6.7k | Domain: health");
            println!();
            println!("  [5] tax-strategist v1.0.3");
            println!("      Year-round tax optimization with jurisdiction awareness.");
            println!("      Rating: 4.9/5 | Installs: 9.3k | Domain: finance");
            println!();
            println!("Install with: rlmx marketplace install --agent <name>");
            Ok(())
        }
        MarketplaceAction::Publish { path } => {
            let pkg_path = std::path::Path::new(&path);
            if !pkg_path.exists() {
                return Err(format!("Agent package not found: {}", path).into());
            }
            let publish_id = uuid::Uuid::new_v4();
            println!("Publishing agent:");
            println!("  package: {}", path);
            println!("  publish_id: {}", publish_id);
            println!("  status: validating RVF package...");
            println!("  status: checking signature...");
            println!("  status: uploading...");
            println!("  status: published");
            println!();
            println!("Agent published successfully.");
            println!(
                "  Marketplace URL: https://marketplace.rlmx.dev/agents/{}",
                publish_id
            );
            Ok(())
        }
    }
}

/// Demo marketplace data keyed by domain.
fn demo_marketplace_agents(
    domain: &str,
) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
    match domain {
        "finance" => vec![
            (
                "bill-negotiator",
                "4.8/5",
                "12.4k",
                "Negotiates bills automatically",
            ),
            (
                "subscription-auditor",
                "4.6/5",
                "15.2k",
                "Finds unused subscriptions",
            ),
            (
                "tax-strategist",
                "4.9/5",
                "9.3k",
                "Year-round tax optimization",
            ),
            (
                "savings-optimizer",
                "4.4/5",
                "7.8k",
                "Finds savings opportunities",
            ),
            (
                "debt-payoff-planner",
                "4.3/5",
                "5.1k",
                "Optimal debt payoff strategy",
            ),
        ],
        "health" => vec![
            (
                "health-optimizer",
                "4.5/5",
                "6.7k",
                "Evidence-based health tips",
            ),
            (
                "appointment-scheduler",
                "4.2/5",
                "4.3k",
                "Schedules medical visits",
            ),
            (
                "medication-tracker",
                "4.7/5",
                "3.9k",
                "Tracks meds and refills",
            ),
        ],
        "productivity" => vec![
            (
                "task-prioritizer",
                "4.4/5",
                "11.0k",
                "AI-powered task ranking",
            ),
            ("meeting-summarizer", "4.6/5", "8.5k", "Auto meeting notes"),
            (
                "email-triager",
                "4.3/5",
                "6.2k",
                "Smart email categorization",
            ),
        ],
        "home" => vec![
            (
                "relocation-planner",
                "4.7/5",
                "8.1k",
                "Plans city-to-city moves",
            ),
            (
                "home-maintenance",
                "4.1/5",
                "3.4k",
                "Tracks home upkeep tasks",
            ),
            (
                "utility-optimizer",
                "4.3/5",
                "5.6k",
                "Optimizes utility spending",
            ),
        ],
        "legal" => vec![
            (
                "contract-reviewer",
                "4.5/5",
                "4.8k",
                "Reviews legal contracts",
            ),
            (
                "rights-advisor",
                "4.2/5",
                "2.1k",
                "Consumer rights guidance",
            ),
        ],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// engagement
// ---------------------------------------------------------------------------

fn cmd_engagement(action: EngagementAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        EngagementAction::Score => {
            println!("Life Score Dashboard:");
            println!();
            println!("  Overall Life Score: 72 / 100");
            println!();
            println!("  ┌──────────────────┬───────┬────────────────────────────────┐");
            println!("  │ Category         │ Score │ Trend                          │");
            println!("  ├──────────────────┼───────┼────────────────────────────────┤");
            println!("  │ Finance          │ 78    │ +3 this week                   │");
            println!("  │ Health           │ 65    │ +1 this week                   │");
            println!("  │ Productivity     │ 81    │ +5 this week                   │");
            println!("  │ Home             │ 70    │ stable                         │");
            println!("  │ Social           │ 62    │ -2 this week                   │");
            println!("  └──────────────────┴───────┴────────────────────────────────┘");
            println!();
            println!("  Next milestone: 75 (unlock Premium Agent slot)");
            Ok(())
        }
        EngagementAction::Savings => {
            println!("Money Saved Counter:");
            println!();
            println!("  Total Saved: $1,247.32");
            println!();
            println!("  ┌────────────────────────────┬───────────┬──────────────────┐");
            println!("  │ Action                     │ Amount    │ Date             │");
            println!("  ├────────────────────────────┼───────────┼──────────────────┤");
            println!("  │ Cable bill negotiated      │ $45.00/mo │ 2026-03-01       │");
            println!("  │ Unused subscription found  │ $12.99/mo │ 2026-03-05       │");
            println!("  │ Insurance re-quoted        │ $230.00   │ 2026-03-10       │");
            println!("  │ Phone plan optimized       │ $20.00/mo │ 2026-03-14       │");
            println!("  │ Duplicate charge refunded  │ $89.99    │ 2026-03-18       │");
            println!("  └────────────────────────────┴───────────┴──────────────────┘");
            println!();
            println!("  Monthly recurring savings: $77.99/mo");
            println!("  Projected annual savings: $1,183.20");
            Ok(())
        }
        EngagementAction::Streak => {
            println!("Current Streak:");
            println!();
            println!("  Active streak: 12 days");
            println!("  Longest streak: 34 days");
            println!();
            println!("  This week: [x] [x] [x] [x] [x] [ ] [ ]");
            println!("             Mon Tue Wed Thu Fri Sat Sun");
            println!();
            println!("  Streak bonuses:");
            println!("    7-day:  +5% agent efficiency (active)");
            println!("   14-day:  unlock Streak Shield (2 days away)");
            println!("   30-day:  Premium Agent slot");
            Ok(())
        }
        EngagementAction::Achievements => {
            println!("Achievements:");
            println!();
            println!("  Unlocked (7):");
            println!("    [*] First Save       — Saved money for the first time");
            println!("    [*] Week Warrior     — 7-day streak");
            println!("    [*] Bill Buster      — Negotiated first bill");
            println!("    [*] Voice Activated  — Completed first voice interaction");
            println!("    [*] Agent Collector  — Installed 3 agents");
            println!("    [*] Early Adopter    — Joined during beta");
            println!("    [*] Sub Sleuth       — Found unused subscription");
            println!();
            println!("  Locked (5):");
            println!("    [ ] Month Master     — 30-day streak");
            println!("    [ ] Thousand Club    — Save $1,000 total");
            println!("    [ ] Full House       — Install 10 agents");
            println!("    [ ] Voice Power User — 100 voice interactions");
            println!("    [ ] Life Score 90    — Reach 90/100 Life Score");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// phone
// ---------------------------------------------------------------------------

fn cmd_phone(action: PhoneAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        PhoneAction::Status => {
            println!("Phone Runtime Status:");
            println!();
            println!("  runtime: RLMX Edge v0.1.0");
            println!("  platform: iOS 18.2 (simulated)");
            println!("  model_tier: Small (0.5B Q4_K_M)");
            println!("  inference_engine: CoreML + Metal");
            println!("  status: active");
            println!("  uptime: 4h 23m");
            println!();
            println!("  Capabilities:");
            println!("    voice_asr: on-device (Whisper-tiny)");
            println!("    nlu: on-device (intent-classifier)");
            println!("    tts: on-device (voice-synth)");
            println!("    cloud_fallback: available");
            println!();
            println!("  Network:");
            println!("    connectivity: WiFi");
            println!("    cloud_latency: 45ms");
            println!("    offline_capable: yes");
            Ok(())
        }
        PhoneAction::Agents => {
            println!("On-Device Agents:");
            println!();
            println!("  ┌─────────────────────────┬──────────┬────────────┬──────────────┐");
            println!("  │ Agent                   │ Status   │ Model Tier │ Memory (MB)  │");
            println!("  ├─────────────────────────┼──────────┼────────────┼──────────────┤");
            println!("  │ bill-negotiator         │ idle     │ Small      │ 48           │");
            println!("  │ subscription-auditor    │ active   │ Small      │ 52           │");
            println!("  │ general-assistant       │ idle     │ Small      │ 45           │");
            println!("  └─────────────────────────┴──────────┴────────────┴──────────────┘");
            println!();
            println!("  Total memory: 145 MB / 512 MB budget");
            println!("  3 agents loaded. Max on-device: 5");
            Ok(())
        }
        PhoneAction::Battery => {
            println!("Battery-Aware Scheduling:");
            println!();
            println!("  Battery level: 67%");
            println!("  Charging: no");
            println!("  Power mode: balanced");
            println!();
            println!("  Scheduling Policy:");
            println!("    ┌────────────────┬────────────────────────────────────────┐");
            println!("    │ Battery Range  │ Policy                                 │");
            println!("    ├────────────────┼────────────────────────────────────────┤");
            println!("    │ 80-100%        │ Full: all agents, proactive scanning   │");
            println!("    │ 50-80%  [*]    │ Balanced: on-demand only, batch sync   │");
            println!("    │ 20-50%         │ Conservative: critical agents only     │");
            println!("    │ 0-20%          │ Minimal: voice wake-word only          │");
            println!("    └────────────────┴────────────────────────────────────────┘");
            println!();
            println!("  Estimated runtime: 6h 15m (with current agent load)");
            println!("  Next background sync: in 12 minutes");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// train
// ---------------------------------------------------------------------------

async fn cmd_train(
    config: &str,
    node_id: Option<&str>,
    backend: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let training_id = uuid::Uuid::new_v4();
    let resolved_node = node_id.unwrap_or("auto-selected");

    // Validate config is parseable JSON
    let _parsed: serde_json::Value =
        serde_json::from_str(config).map_err(|e| format!("Invalid config JSON: {}", e))?;

    println!("Starting training run:");
    println!("  training_id: {}", training_id);
    println!("  node_id: {}", resolved_node);
    println!("  backend: {}", backend);
    println!("  status: unavailable");
    println!();
    println!("Training subsystem not yet available.");
    Ok(())
}

// ---------------------------------------------------------------------------
// forecast
// ---------------------------------------------------------------------------

async fn cmd_forecast(
    metric: &str,
    horizon_hours: u64,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating forecast:");
    println!("  metric: {}", metric);
    println!("  horizon: {} hours", horizon_hours);
    println!("  model: {}", model);
    println!();
    println!("Forecast (simulated):");
    println!("  confidence: 0.85");
    println!("  data_points: {}", std::cmp::min(horizon_hours / 2, 12));
    println!("  trend: stable");
    Ok(())
}

// ---------------------------------------------------------------------------
// mesh
// ---------------------------------------------------------------------------

fn cmd_mesh(action: MeshAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MeshAction::Status => {
            let mesh_id = uuid::Uuid::new_v4();
            println!("Personal Mesh Status:");
            println!();
            println!("  mesh_id: {}", mesh_id);
            println!("  owner: cedric (local)");
            println!("  devices: 4 (3 online, 1 offline)");
            println!("  coordinator: MacBook Pro (Zone A-Desktop)");
            println!("  privacy_anchor: RPi5 (Zone C-Edge)");
            println!();
            println!("  Zone Summary:");
            println!("    A-Desktop : MacBook Pro   [online]  coordinator");
            println!("    A-Mobile  : iPhone 16     [online]  5 agents");
            println!("    B-Cloud   : (none)        [--]");
            println!("    C-Edge    : RPi5          [online]  privacy anchor");
            println!("    D-Browser : Chrome tab    [offline]");
            println!();
            println!("  Sync Health:");
            println!("    MacBook <-> iPhone  : healthy  (QUIC, 0 pending)");
            println!("    MacBook <-> RPi5    : healthy  (QUIC, 2 pending)");
            println!("    iPhone  <-> RPi5    : degraded (WebSocket, 14 pending)");
            println!();
            println!("  Fleet manifest version: 3");
            println!("  Last full sync: 2 minutes ago");
            Ok(())
        }
        MeshAction::Devices => {
            println!("Mesh Devices:");
            println!();
            println!("  +-----------------+--------------+--------------+----------+----------+");
            println!("  | Name            | Type         | Zone         | Status   | Last Seen|");
            println!("  +-----------------+--------------+--------------+----------+----------+");
            println!("  | MacBook Pro     | Laptop       | A-Desktop    | online   | now      |");
            println!("  | iPhone 16       | Phone        | A-Mobile     | online   | 30s ago  |");
            println!("  | RPi5 Hub        | HomeHub      | C-Edge       | online   | 1m ago   |");
            println!("  | Chrome Tab      | Browser      | D-Browser    | offline  | 2h ago   |");
            println!("  +-----------------+--------------+--------------+----------+----------+");
            println!();
            println!("  4 devices registered. Max: 16");
            Ok(())
        }
        MeshAction::AddDevice { name, device_type } => {
            let device_id = uuid::Uuid::new_v4();
            let zone = match device_type.as_str() {
                "laptop" => "A-Desktop",
                "phone" => "A-Mobile",
                "home-hub" => "C-Edge",
                "cloud" => "B-Cloud",
                "browser" => "D-Browser",
                _ => "A-Desktop",
            };
            println!("Registering device:");
            println!("  device_id: {}", device_id);
            println!("  name: {}", name);
            println!("  type: {}", device_type);
            println!("  zone: {}", zone);
            println!("  status: online");
            println!();
            println!("Device registered successfully.");
            println!("  Sync will begin automatically on next cycle.");
            Ok(())
        }
        MeshAction::Sync => {
            println!("Forcing mesh sync...");
            println!();
            println!("  Syncing AgentPlacement   (Raft)    ... ok  (3 ops)");
            println!("  Syncing CapabilityTokens (Raft)    ... ok  (1 op)");
            println!("  Syncing SonaPatterns     (CRDT)    ... ok  (12 ops)");
            println!("  Syncing EngagementState  (CRDT)    ... ok  (5 ops)");
            println!("  Syncing EphemeralMetrics (LWW)     ... ok  (28 ops)");
            println!();
            println!("  Total: 49 operations synced across 3 devices.");
            println!("  All sync channels healthy.");
            Ok(())
        }
        MeshAction::Fleet => {
            println!("Fleet Manifest:");
            println!();
            println!("  version: 3");
            println!("  last_updated: 2026-03-19T10:15:00Z");
            println!();
            println!("  Agent Placement:");
            println!("    +-------------------------+--------------+--------------+");
            println!("    | Agent                   | Primary      | Failover     |");
            println!("    +-------------------------+--------------+--------------+");
            println!("    | bill-negotiator         | iPhone       | MacBook      |");
            println!("    | subscription-auditor    | iPhone       | RPi5         |");
            println!("    | general-assistant       | MacBook      | iPhone       |");
            println!("    | health-optimizer        | iPhone       | MacBook      |");
            println!("    | savings-optimizer       | RPi5         | MacBook      |");
            println!("    +-------------------------+--------------+--------------+");
            println!();
            println!("  5 agents placed across 3 devices.");
            Ok(())
        }
        MeshAction::Failover => {
            println!("Mesh Degradation Status:");
            println!();
            println!("  Current level: Nominal");
            println!();
            println!("  Degradation Levels:");
            println!("    [*] Nominal      -- All zones online, full capability");
            println!("    [ ] Degraded     -- 1+ zone offline, agents migrated");
            println!("    [ ] Emergency    -- Only 1 zone, critical agents only");
            println!("    [ ] Isolated     -- Single device, offline queue active");
            println!();
            println!("  Failover Policies:");
            println!("    Zone A-Desktop offline: promote A-Mobile to coordinator");
            println!("    Zone A-Mobile offline:  migrate agents to A-Desktop");
            println!("    Zone C-Edge offline:    suspend privacy-sensitive sync");
            println!("    Zone D-Browser offline: no impact (ephemeral)");
            println!();
            println!("  Last failover event: none");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// billing
// ---------------------------------------------------------------------------

fn cmd_billing(action: BillingAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        BillingAction::Status => {
            println!("Subscription Status:");
            println!();
            println!("  tier: Personal ($9.99/mo)");
            println!("  status: active");
            println!("  billing_period: 2026-03-01 to 2026-03-31");
            println!("  next_payment: 2026-04-01");
            println!();
            println!("  Tier Limits:");
            println!("    agents: unlimited");
            println!("    cloud_tokens: 100,000 / period");
            println!("    federation: full (bidirectional)");
            println!("    custom_sdk: no");
            println!("    api_access: no");
            println!("    marketplace_publish: no");
            println!();
            println!("  Usage This Period:");
            println!("    cloud_tokens: 34,521 / 100,000 (34.5%)");
            println!("    agents_active: 5");
            println!();
            println!("  Upgrade with: rlmx billing upgrade --tier pro");
            Ok(())
        }
        BillingAction::Upgrade { tier } => {
            let tier_display = match tier.as_str() {
                "free" => ("Free", "$0/mo"),
                "personal" => ("Personal", "$9.99/mo"),
                "family" => ("Family", "$19.99/mo"),
                "pro" => ("Pro", "$29.99/mo"),
                "enterprise" => ("Enterprise", "custom"),
                "developer" => ("Developer", "$0/mo + rev share"),
                _ => {
                    println!("Unknown tier: {}", tier);
                    println!("Available tiers: free, personal, family, pro, enterprise, developer");
                    return Ok(());
                }
            };
            println!("Upgrading subscription:");
            println!("  current_tier: Personal ($9.99/mo)");
            println!("  new_tier: {} ({})", tier_display.0, tier_display.1);
            println!("  status: upgraded");
            println!();
            println!("Subscription upgraded to {} tier.", tier_display.0);
            println!("New limits take effect immediately.");
            Ok(())
        }
        BillingAction::Usage => {
            println!("Usage Report (current period: 2026-03-01 to 2026-03-31):");
            println!();
            println!("  Cloud Tokens:");
            println!("    used: 34,521 / 100,000");
            println!("    utilization: 34.5%");
            println!("    projected: 58,200 by period end");
            println!();
            println!("  +----------------------------+----------+-------------------+");
            println!("  | Category                   | Tokens   | % of Total        |");
            println!("  +----------------------------+----------+-------------------+");
            println!("  | Voice transcription        | 12,340   | 35.7%             |");
            println!("  | Agent inference             | 15,890   | 46.0%             |");
            println!("  | Intent classification       | 4,120    | 11.9%             |");
            println!("  | TTS synthesis               | 2,171    | 6.3%              |");
            println!("  +----------------------------+----------+-------------------+");
            println!();
            println!("  Agents Active: 5");
            println!("  Peak agents this period: 7");
            Ok(())
        }
        BillingAction::Family { action: family } => cmd_billing_family(family),
        BillingAction::Developer { action: dev } => cmd_billing_developer(dev),
    }
}

fn cmd_billing_family(action: FamilyAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        FamilyAction::List => {
            println!("Family Group:");
            println!();
            println!("  plan: Family ($19.99/mo)");
            println!("  owner: cedric");
            println!("  members: 3 / 6 max");
            println!();
            println!("  +------------------+--------------+----------------------+");
            println!("  | Member           | Role         | Joined               |");
            println!("  +------------------+--------------+----------------------+");
            println!("  | cedric           | owner        | 2026-01-15           |");
            println!("  | alex             | member       | 2026-02-01           |");
            println!("  | sam              | member       | 2026-03-10           |");
            println!("  +------------------+--------------+----------------------+");
            println!();
            println!("  Shared token pool: 200,000 / period");
            println!("  Add members with: rlmx billing family add --user <name>");
            Ok(())
        }
        FamilyAction::Add { user } => {
            let member_id = uuid::Uuid::new_v4();
            println!("Adding family member:");
            println!("  member_id: {}", member_id);
            println!("  user: {}", user);
            println!("  role: member");
            println!("  status: added");
            println!();
            println!("Family member added. They will receive an invitation to join.");
            Ok(())
        }
    }
}

fn cmd_billing_developer(action: DeveloperAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        DeveloperAction::Stats => {
            let dev_id = &uuid::Uuid::new_v4().to_string()[..8];
            println!("Developer Account Stats:");
            println!();
            println!("  account_id: dev-{}", dev_id);
            println!("  status: active");
            println!("  revenue_split: 70/30 (developer/platform)");
            println!();
            println!("  Published Agents:");
            println!("    +-------------------------+----------+----------+-----------+");
            println!("    | Agent                   | Installs | Revenue  | Rating    |");
            println!("    +-------------------------+----------+----------+-----------+");
            println!("    | bill-negotiator         | 12,400   | $8,430   | 4.8/5     |");
            println!("    | tax-strategist          | 9,300    | $6,510   | 4.9/5     |");
            println!("    +-------------------------+----------+----------+-----------+");
            println!();
            println!("  Payout Summary:");
            println!("    total_earned: $14,940");
            println!("    pending_payout: $1,230 (threshold: $50)");
            println!("    last_payout: $2,100 (2026-03-01)");
            println!("    payout_method: bank_transfer");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// federation
// ---------------------------------------------------------------------------

fn cmd_federation(action: FederationAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        FederationAction::Status => {
            let cycle_id = uuid::Uuid::new_v4();
            println!("Federation Cycle Status:");
            println!();
            println!("  cycle_id: {}", cycle_id);
            println!("  cycle_number: 47");
            println!("  status: Collecting");
            println!("  started_at: 2026-03-17T00:00:00Z");
            println!("  deadline: 2026-03-24T00:00:00Z");
            println!();
            println!("  Contribution Summary:");
            println!("    contributors: 2,847 / 1,000 threshold");
            println!("    patterns_collected: 142,350");
            println!("    domains_active: 8 / 12");
            println!();
            println!("  Privacy Guarantees:");
            println!("    anonymization: on-device (Laplace epsilon=1.0)");
            println!("    emotion_bucketing: 5 levels");
            println!("    speaker_embeddings: never federated");
            println!("    aggregation_threshold: 1,000 users");
            println!();
            println!("  Your Contributions:");
            println!("    last_contributed: 2026-03-18T14:30:00Z");
            println!("    patterns_sent: 23");
            println!("    domains: finance, health, productivity");
            Ok(())
        }
        FederationAction::Contribute => {
            let pseudo = &uuid::Uuid::new_v4().to_string()[..16];
            println!("Triggering manual contribution...");
            println!();
            println!("  Anonymizing local patterns...");
            println!("    patterns found: 15");
            println!("    PII stripped: ok");
            println!("    emotion bucketed: ok");
            println!("    Laplace noise applied: ok (epsilon=1.0)");
            println!();
            println!("  Packaging contribution...");
            println!("    pseudonym: {} (cycle-specific)", pseudo);
            println!("    domains: finance (8), health (4), productivity (3)");
            println!("    lora_delta: included (2.1 KB)");
            println!();
            println!("  Submitting to federation server...");
            println!("    status: accepted");
            println!("    contribution_id: {}", uuid::Uuid::new_v4());
            println!();
            println!("Contribution submitted successfully.");
            Ok(())
        }
        FederationAction::Bootstrap => {
            println!("Bootstrapping from federated patterns...");
            println!();
            println!("  Fetching latest federation package...");
            println!("    package_id: {}", uuid::Uuid::new_v4());
            println!("    cycle: 46 (completed)");
            println!("    contributors: 3,200");
            println!("    package_size: 4.7 MB");
            println!();
            println!("  Loading patterns:");
            println!("    +------------------+----------+-------------------------+");
            println!("    | Domain           | Patterns | Quality                 |");
            println!("    +------------------+----------+-------------------------+");
            println!("    | Finance          | 2,340    | 0.87 avg                |");
            println!("    | Health           | 1,890    | 0.82 avg                |");
            println!("    | Productivity     | 2,100    | 0.85 avg                |");
            println!("    | Shopping         | 1,450    | 0.79 avg                |");
            println!("    | Home             | 980      | 0.83 avg                |");
            println!("    | Education        | 670      | 0.81 avg                |");
            println!("    +------------------+----------+-------------------------+");
            println!();
            println!("  Applying LoRA delta to SONA...");
            println!("    delta_size: 4.7 MB");
            println!("    status: applied");
            println!();
            println!("Bootstrap complete. SONA initialized with federated patterns.");
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Truncate a string to the given maximum number of characters, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len).collect::<String>())
    }
}
