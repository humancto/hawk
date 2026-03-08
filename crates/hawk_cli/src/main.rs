use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
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
        /// Output format: text or json
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
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
        /// Output format: text or json
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        /// Exit with code 1 if changes detected (for CI)
        #[arg(long)]
        exit_code: bool,
    },
    /// Validate a hawk.json file for schema correctness
    Validate {
        /// Input hawk.json file
        #[arg(long, default_value = "hawk.json")]
        r#in: PathBuf,
    },
    /// Watch for infrastructure changes by re-scanning periodically
    Watch {
        #[command(subcommand)]
        target: WatchTarget,
    },
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
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
    /// AWS profile name(s), comma-separated for multi-account
    #[arg(long, value_delimiter = ',')]
    profile: Vec<String>,
    /// AWS region(s), comma-separated for multi-region scanning
    #[arg(long, value_delimiter = ',')]
    region: Vec<String>,
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
    /// Export as Graphviz DOT diagram
    Dot {
        /// Input hawk.json file
        #[arg(long, default_value = "hawk.json")]
        r#in: PathBuf,
        /// Output .dot file
        #[arg(long, default_value = "hawk.dot")]
        out: PathBuf,
        /// Show all node types (not just Lambda + triggers)
        #[arg(long)]
        full: bool,
    },
}

#[derive(Subcommand)]
enum WatchTarget {
    /// Watch AWS resources for changes
    Aws {
        #[command(subcommand)]
        scope: WatchAwsScope,
    },
}

#[derive(Subcommand)]
enum WatchAwsScope {
    /// Watch all supported AWS resources
    All {
        #[command(flatten)]
        opts: WatchOpts,
    },
}

#[derive(Parser, Clone)]
struct WatchOpts {
    /// AWS profile name(s), comma-separated for multi-account
    #[arg(long, value_delimiter = ',')]
    profile: Vec<String>,
    /// AWS region(s), comma-separated for multi-region
    #[arg(long, value_delimiter = ',')]
    region: Vec<String>,
    /// Interval between scans (e.g. "5m", "30s", "1h")
    #[arg(long, default_value = "5m")]
    interval: String,
    /// Output file path
    #[arg(long, default_value = "hawk.json")]
    out: PathBuf,
    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
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

                let graph = run_discovery(&opts, discover_scope).await?;

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

        Commands::Summary {
            r#in: input,
            format,
        } => {
            let data = std::fs::read_to_string(&input)?;
            let graph: hawk_core::Graph = serde_json::from_str(&data)?;
            match format {
                OutputFormat::Text => print_summary(&graph),
                OutputFormat::Json => {
                    let summary = serde_json::json!({
                        "generated_at": graph.generated_at,
                        "profile": graph.profile,
                        "regions": graph.regions,
                        "stats": graph.stats,
                        "warnings": graph.warnings,
                    });
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                }
            }
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
            ExportFormat::Dot {
                r#in: input,
                out,
                full,
            } => {
                let data = std::fs::read_to_string(&input)?;
                let graph: hawk_core::Graph = serde_json::from_str(&data)?;
                let opts = hawk_render::DotOptions {
                    full,
                    ..Default::default()
                };
                let dot = hawk_render::render_dot(&graph, &opts);
                std::fs::write(&out, &dot)?;
                println!("DOT diagram written to {}", out.display());
            }
        },

        Commands::Diff {
            old,
            new,
            format,
            exit_code,
        } => {
            let old_data = std::fs::read_to_string(&old)?;
            let new_data = std::fs::read_to_string(&new)?;
            let old_graph: hawk_core::Graph = serde_json::from_str(&old_data)?;
            let new_graph: hawk_core::Graph = serde_json::from_str(&new_data)?;
            let diff = hawk_core::GraphDiff::compute(&old_graph, &new_graph);

            let has_changes = !diff.added_nodes.is_empty()
                || !diff.removed_nodes.is_empty()
                || !diff.added_edges.is_empty()
                || !diff.removed_edges.is_empty();

            match format {
                OutputFormat::Text => {
                    println!("=== Graph Diff ===\n");
                    if !has_changes {
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
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&diff)?);
                }
            }

            if exit_code && has_changes {
                std::process::exit(1);
            }
        }

        Commands::Validate { r#in: input } => {
            let data = std::fs::read_to_string(&input)?;
            let graph: hawk_core::Graph = match serde_json::from_str(&data) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("\x1b[31m✗ Failed to parse {}: {e}\x1b[0m", input.display());
                    std::process::exit(1);
                }
            };

            let result = hawk_core::validate::validate_graph(&graph);

            println!("=== Hawk Validate ===\n");
            println!("File:  {}", input.display());
            println!("Nodes: {}", graph.nodes.len());
            println!("Edges: {}", graph.edges.len());
            println!();

            if result.errors.is_empty() && result.warnings.is_empty() {
                println!("\x1b[32m✓ Valid — no issues found\x1b[0m");
            } else {
                for e in &result.errors {
                    eprintln!("\x1b[31m  ✗ ERROR: {e}\x1b[0m");
                }
                for w in &result.warnings {
                    println!("\x1b[33m  ⚠ WARN:  {w}\x1b[0m");
                }
                println!();
                if !result.is_valid() {
                    eprintln!(
                        "\x1b[31m✗ Validation failed with {} error(s)\x1b[0m",
                        result.errors.len()
                    );
                    std::process::exit(1);
                } else {
                    println!(
                        "\x1b[32m✓ Valid\x1b[0m with {} warning(s)",
                        result.warnings.len()
                    );
                }
            }
        }

        Commands::Watch { target } => match target {
            WatchTarget::Aws { scope } => match scope {
                WatchAwsScope::All { opts: watch_opts } => {
                    init_tracing(watch_opts.verbose);

                    let interval = parse_duration(&watch_opts.interval)?;
                    let aws_opts = AwsOpts {
                        profile: watch_opts.profile,
                        region: watch_opts.region,
                        out: watch_opts.out,
                        pretty: true,
                        verbose: watch_opts.verbose,
                    };

                    println!(
                        "Watching for changes every {}s. Press Ctrl-C to stop.\n",
                        interval.as_secs()
                    );

                    let mut prev_graph: Option<hawk_core::Graph> = None;
                    loop {
                        let graph =
                            run_discovery(&aws_opts, hawk_aws::discover::Scope::All).await?;

                        let json = serde_json::to_string_pretty(&graph)?;
                        std::fs::write(&aws_opts.out, &json)?;

                        if let Some(ref old) = prev_graph {
                            let diff = hawk_core::GraphDiff::compute(old, &graph);
                            let has_changes = !diff.added_nodes.is_empty()
                                || !diff.removed_nodes.is_empty()
                                || !diff.added_edges.is_empty()
                                || !diff.removed_edges.is_empty();
                            if has_changes {
                                println!(
                                    "[{}] Changes detected:",
                                    chrono::Utc::now().format("%H:%M:%S")
                                );
                                for n in &diff.added_nodes {
                                    println!("  + node: {n}");
                                }
                                for n in &diff.removed_nodes {
                                    println!("  - node: {n}");
                                }
                                for e in &diff.added_edges {
                                    println!("  + edge: {e}");
                                }
                                for e in &diff.removed_edges {
                                    println!("  - edge: {e}");
                                }
                            } else {
                                println!(
                                    "[{}] No changes ({} nodes, {} edges)",
                                    chrono::Utc::now().format("%H:%M:%S"),
                                    graph.stats.node_count,
                                    graph.stats.edge_count
                                );
                            }
                        } else {
                            println!(
                                "[{}] Initial scan: {} nodes, {} edges",
                                chrono::Utc::now().format("%H:%M:%S"),
                                graph.stats.node_count,
                                graph.stats.edge_count
                            );
                        }

                        prev_graph = Some(graph);
                        tokio::time::sleep(interval).await;
                    }
                }
            },
        },
    }

    Ok(())
}

/// Run discovery across multiple profiles and regions, merging results.
async fn run_discovery(
    opts: &AwsOpts,
    scope: hawk_aws::discover::Scope,
) -> Result<hawk_core::Graph> {
    let profiles: Vec<Option<&str>> = if opts.profile.is_empty() {
        vec![None]
    } else {
        opts.profile.iter().map(|p| Some(p.as_str())).collect()
    };
    let regions: Vec<Option<&str>> = if opts.region.is_empty() {
        vec![None]
    } else {
        opts.region.iter().map(|r| Some(r.as_str())).collect()
    };

    let mut merged = hawk_core::Graph::new();

    for profile in &profiles {
        for region in &regions {
            let ctx = hawk_aws::AwsCtx::new(*profile, *region).await?;
            let graph = hawk_aws::discover_all(&ctx, scope, *profile).await?;

            // Merge regions list
            for r in &graph.regions {
                if !merged.regions.contains(r) {
                    merged.regions.push(r.clone());
                }
            }

            merged.nodes.extend(graph.nodes);
            merged.edges.extend(graph.edges);
            merged.warnings.extend(graph.warnings);
        }
    }

    // Set profile info
    if !opts.profile.is_empty() {
        merged.profile = Some(opts.profile.join(","));
    }

    merged.dedupe_and_sort();
    merged.compute_stats();
    Ok(merged)
}

/// Parse a human-readable duration string like "5m", "30s", "1h".
fn parse_duration(s: &str) -> Result<std::time::Duration> {
    let s = s.trim();
    if let Some(val) = s.strip_suffix('s') {
        Ok(std::time::Duration::from_secs(val.parse()?))
    } else if let Some(val) = s.strip_suffix('m') {
        Ok(std::time::Duration::from_secs(val.parse::<u64>()? * 60))
    } else if let Some(val) = s.strip_suffix('h') {
        Ok(std::time::Duration::from_secs(val.parse::<u64>()? * 3600))
    } else {
        // Default: treat as seconds
        Ok(std::time::Duration::from_secs(s.parse()?))
    }
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
        for (name, count) in &graph.stats.top_fan_in {
            println!("  {name}: {count}");
        }
    }

    if !graph.stats.top_fan_out.is_empty() {
        println!();
        println!("Top fan-out (most connections):");
        for (name, count) in &graph.stats.top_fan_out {
            println!("  {name}: {count}");
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
