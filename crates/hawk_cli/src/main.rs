use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hawk", about = "AWS infrastructure graph analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze AWS infrastructure
    Analyze {
        #[command(subcommand)]
        target: AnalyzeTarget,
    },
    /// Print summary statistics from a hawk.json file
    Summary {
        /// Input hawk.json file
        #[arg(long, default_value = "hawk.json")]
        r#in: PathBuf,
    },
    /// Export graph to different formats
    Export {
        #[command(subcommand)]
        format: ExportFormat,
    },
    /// Diff two hawk.json files
    Diff {
        /// Old hawk.json
        #[arg(long)]
        old: PathBuf,
        /// New hawk.json
        #[arg(long)]
        new: PathBuf,
    },
}

#[derive(Subcommand)]
enum AnalyzeTarget {
    /// Analyze AWS resources
    Aws {
        #[command(subcommand)]
        scope: AwsScope,
    },
}

#[derive(Subcommand)]
enum AwsScope {
    /// Discover Lambda functions and their triggers only
    Lambda {
        #[command(flatten)]
        opts: AwsOpts,
    },
    /// Discover all supported AWS resources
    All {
        #[command(flatten)]
        opts: AwsOpts,
    },
}

#[derive(Parser, Clone)]
struct AwsOpts {
    /// AWS profile name
    #[arg(long)]
    profile: Option<String>,
    /// AWS region
    #[arg(long)]
    region: Option<String>,
    /// Output file path
    #[arg(long, default_value = "hawk.json")]
    out: PathBuf,
    /// Pretty-print JSON output
    #[arg(long)]
    pretty: bool,
    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum ExportFormat {
    /// Export as Mermaid diagram
    Mermaid {
        /// Input hawk.json file
        #[arg(long, default_value = "hawk.json")]
        r#in: PathBuf,
        /// Output .mmd file
        #[arg(long, default_value = "hawk.mmd")]
        out: PathBuf,
        /// Show all node types (not just Lambda + triggers)
        #[arg(long)]
        full: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { target } => match target {
            AnalyzeTarget::Aws { scope } => {
                let (opts, discover_scope) = match scope {
                    AwsScope::Lambda { opts } => (opts, hawk_aws::discover::Scope::LambdaOnly),
                    AwsScope::All { opts } => (opts, hawk_aws::discover::Scope::All),
                };

                init_tracing(opts.verbose);

                let ctx = hawk_aws::AwsCtx::new(
                    opts.profile.as_deref(),
                    opts.region.as_deref(),
                )
                .await?;

                let graph =
                    hawk_aws::discover_all(&ctx, discover_scope, opts.profile.as_deref()).await?;

                // Write JSON
                let json = if opts.pretty {
                    serde_json::to_string_pretty(&graph)?
                } else {
                    serde_json::to_string(&graph)?
                };
                std::fs::write(&opts.out, &json)?;

                // Print summary
                print_summary(&graph);
                println!("\nGraph written to {}", opts.out.display());
            }
        },

        Commands::Summary { r#in: input } => {
            let data = std::fs::read_to_string(&input)?;
            let graph: hawk_core::Graph = serde_json::from_str(&data)?;
            print_summary(&graph);
        }

        Commands::Export { format } => match format {
            ExportFormat::Mermaid {
                r#in: input,
                out,
                full,
            } => {
                let data = std::fs::read_to_string(&input)?;
                let graph: hawk_core::Graph = serde_json::from_str(&data)?;
                let opts = hawk_render::MermaidOptions {
                    full,
                    ..Default::default()
                };
                let mmd = hawk_render::render_mermaid(&graph, &opts);
                std::fs::write(&out, &mmd)?;
                println!("Mermaid diagram written to {}", out.display());
            }
        },

        Commands::Diff { old, new } => {
            let old_data = std::fs::read_to_string(&old)?;
            let new_data = std::fs::read_to_string(&new)?;
            let old_graph: hawk_core::Graph = serde_json::from_str(&old_data)?;
            let new_graph: hawk_core::Graph = serde_json::from_str(&new_data)?;
            let diff = hawk_core::GraphDiff::compute(&old_graph, &new_graph);

            println!("=== Graph Diff ===\n");
            if diff.added_nodes.is_empty() && diff.removed_nodes.is_empty()
                && diff.added_edges.is_empty() && diff.removed_edges.is_empty()
            {
                println!("No changes detected.");
            } else {
                if !diff.added_nodes.is_empty() {
                    println!("Added nodes ({}):", diff.added_nodes.len());
                    for n in &diff.added_nodes {
                        println!("  + {n}");
                    }
                }
                if !diff.removed_nodes.is_empty() {
                    println!("Removed nodes ({}):", diff.removed_nodes.len());
                    for n in &diff.removed_nodes {
                        println!("  - {n}");
                    }
                }
                if !diff.added_edges.is_empty() {
                    println!("Added edges ({}):", diff.added_edges.len());
                    for e in &diff.added_edges {
                        println!("  + {e}");
                    }
                }
                if !diff.removed_edges.is_empty() {
                    println!("Removed edges ({}):", diff.removed_edges.len());
                    for e in &diff.removed_edges {
                        println!("  - {e}");
                    }
                }
            }
        }
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = if verbose {
        EnvFilter::new("hawk=debug")
    } else {
        EnvFilter::new("hawk=info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn print_summary(graph: &hawk_core::Graph) {
    println!("=== Hawk Summary ===\n");
    println!("Generated: {}", graph.generated_at);
    if let Some(p) = &graph.profile {
        println!("Profile:   {p}");
    }
    println!("Regions:   {}", graph.regions.join(", "));
    println!();
    println!("Nodes: {}", graph.stats.node_count);
    for (kind, count) in &graph.stats.nodes_by_kind {
        println!("  {kind}: {count}");
    }
    println!();
    println!("Edges: {}", graph.stats.edge_count);
    for (kind, count) in &graph.stats.edges_by_kind {
        println!("  {kind}: {count}");
    }

    if !graph.stats.top_fan_in.is_empty() {
        println!();
        println!("Top fan-in (most triggered):");
        for (id, count) in &graph.stats.top_fan_in {
            let short = id.rsplit(':').next().unwrap_or(id);
            println!("  {short}: {count}");
        }
    }

    if !graph.stats.top_fan_out.is_empty() {
        println!();
        println!("Top fan-out (most connections):");
        for (id, count) in &graph.stats.top_fan_out {
            let short = id.rsplit(':').next().unwrap_or(id);
            println!("  {short}: {count}");
        }
    }

    if !graph.warnings.is_empty() {
        println!();
        println!("Warnings ({}):", graph.warnings.len());
        for w in &graph.warnings {
            println!("  ! {w}");
        }
    }
}
