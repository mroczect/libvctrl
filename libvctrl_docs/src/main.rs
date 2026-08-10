use librawssg::markdown::PulldownMarkdown;
use librawssg::site::TeraRenderer;
use librawssg::site::context::{TeraFeedContextBuilder, TeraSitemapContextBuilder};
use librawssg::{RawssgError, SiteBuilder};
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "rawssg.yaml";
const TEMPLATES_DIR: &str = "templates";
const OUTPUT_DIR: &str = "dist";
const DEFAULT_PORT: u16 = 3000;

fn load_templates(tera: &mut TeraRenderer, dir: &Path) -> Result<(), RawssgError> {
    load_templates_recursive(tera, dir, dir)
}

fn load_templates_recursive(
    tera: &mut TeraRenderer,
    base: &Path,
    current: &Path,
) -> Result<(), RawssgError> {
    let entries = std::fs::read_dir(current).map_err(RawssgError::Io)?;

    for entry in entries {
        let entry = entry.map_err(RawssgError::Io)?;
        let path = entry.path();

        if path.is_dir() {
            load_templates_recursive(tera, base, &path)?;
        } else if path.is_file() {
            let name = path
                .strip_prefix(base)
                .map_err(|e| RawssgError::Template(format!("strip prefix: {}", e)))?
                .to_string_lossy()
                .to_string();

            let name = name.replace('\\', "/");

            let content = std::fs::read_to_string(&path).map_err(RawssgError::Io)?;
            tera.add_raw_template(&name, &content)?;

            tracing::debug!("Loaded template: {}", name);
        }
    }

    Ok(())
}

fn build_site() -> Result<(), RawssgError> {
    let start = std::time::Instant::now();

    tracing::info!("Building site...");

    let md_renderer = Box::new(PulldownMarkdown);

    let mut tera = TeraRenderer::new();
    load_templates(&mut tera, Path::new(TEMPLATES_DIR))?;
    tracing::info!("Templates loaded from {}", TEMPLATES_DIR);

    let site = SiteBuilder::new()
        .load_config(Path::new(CONFIG_FILE))?
        .with_markdown_renderer(md_renderer)
        .with_template_renderer(Box::new(tera))
        .with_feed_context_builder(Box::new(TeraFeedContextBuilder))
        .with_sitemap_context_builder(Box::new(TeraSitemapContextBuilder))
        .build()?;

    site.generate()?;

    let elapsed = start.elapsed();
    tracing::info!(
        "Site built successfully → {} ({:.2}s)",
        OUTPUT_DIR,
        elapsed.as_secs_f64()
    );

    Ok(())
}

fn serve_mode() -> Result<(), RawssgError> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    build_site()?;

    let watch_dirs: Vec<PathBuf> = vec![
        PathBuf::from("content"),
        PathBuf::from("templates"),
        PathBuf::from("static"),
        PathBuf::from(CONFIG_FILE),
    ];

    let watch_dirs: Vec<PathBuf> = watch_dirs.into_iter().filter(|p| p.exists()).collect();

    let _watcher = librawssg::serve::watch_dirs(&watch_dirs, || {
        tracing::info!("⚡ Change detected — rebuilding...");
        if let Err(e) = build_site() {
            tracing::error!("Rebuild failed: {}", e);
        }
    })
    .map_err(|e| RawssgError::Internal(format!("watcher error: {}", e)))?;

    tracing::info!("Watching: {:?}", watch_dirs);

    librawssg::serve::start_dev_server(Path::new(OUTPUT_DIR), port)?;

    Ok(())
}

fn print_usage() {
    eprintln!("libvctrl_docs — documentation builder");
    eprintln!();
    eprintln!("Usage: libvctrl_docs <command>");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  build   Build the static site (default)");
    eprintln!("  serve   Build + watch + serve on localhost:PORT");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  PORT    Port for dev server (default: {})", DEFAULT_PORT);
    eprintln!("  RUST_LOG  Tracing filter (default: info)");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("build");

    match cmd {
        "build" => {
            build_site()?;
        }
        "serve" => {
            serve_mode()?;
        }
        "-h" | "--help" | "help" => {
            print_usage();
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!();
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}
