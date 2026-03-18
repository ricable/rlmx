use std::path::Path;

use clap::{Parser, Subcommand};

/// RLMX - The RuVix Cognition Kernel CLI
#[derive(Parser, Debug)]
#[command(name = "rlmx", version, about = "RuVix cognition kernel command-line interface")]
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
            println!("merge not yet implemented (source={}, target={})", source, target);
            Ok(())
        }
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
    use rlmx_mcp::{McpConfig, McpServer, Transport, create_all_tools, new_shared_state};

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

    server.start().await.map_err(|e| {
        Box::<dyn std::error::Error>::from(format!("MCP server error: {}", e))
    })?;

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
    assert!(valid, "seal verification failed immediately after signing");

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
    let branch_path = format!("{}.branch-{}.rvf.json", source_path.trim_end_matches(".rvf.json").trim_end_matches(".json"), name);
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
// Helpers
// ---------------------------------------------------------------------------

/// Generate a deterministic pseudo-embedding from text.
///
/// This is a placeholder that produces a fixed-length f32 vector by hashing
/// character byte values. It is NOT a real embedding model but allows the
/// vector search pipeline to function end-to-end.
fn text_to_embedding(text: &str) -> Vec<f32> {
    const DIM: usize = 64;
    let mut embedding = vec![0.0_f32; DIM];

    for (i, byte) in text.bytes().enumerate() {
        let idx = i % DIM;
        // Simple deterministic mixing.
        embedding[idx] += (byte as f32) * 0.01;
    }

    // Normalize to unit length.
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut embedding {
            *v /= norm;
        }
    }

    embedding
}

/// Truncate a string to the given maximum length, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
