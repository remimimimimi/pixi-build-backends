use std::{
    io::Read,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use fs_err::{create_dir_all, read_to_string};
use miette::{Context, IntoDiagnostic, miette};
use pixi_build_types::{
    BackendCapabilities, ChannelConfiguration, FrontendCapabilities, PlatformAndVirtualPackages,
    procedures::{
        conda_build_v0::CondaBuildParams,
        conda_build_v1::{CondaBuildV1Params, CondaBuildV1Result},
        conda_metadata::{CondaMetadataParams, CondaMetadataResult},
        initialize::InitializeParams,
        negotiate_capabilities::NegotiateCapabilitiesParams,
    },
};
use rattler_build::console_utils::{LoggingOutputHandler, get_default_env_filter};
use rattler_conda_types::{ChannelConfig, GenericVirtualPackage, Platform};
use rattler_virtual_packages::{VirtualPackage, VirtualPackageOverrides};
use tempfile::TempDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    consts,
    project::to_project_model,
    protocol::{Protocol, ProtocolInstantiator},
    server::Server,
};

#[allow(missing_docs)]
#[derive(Parser)]
pub struct App {
    /// The subcommand to run.
    #[clap(subcommand)]
    command: Option<Commands>,

    /// The port to expose the json-rpc server on. If not specified will
    /// communicate with stdin/stdout.
    #[clap(long)]
    http_port: Option<u16>,

    /// Enable verbose logging.
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get conda metadata for a recipe.
    GetCondaMetadata {
        #[clap(env, long, env = "PIXI_PROJECT_MANIFEST", default_value = consts::WORKSPACE_MANIFEST)]
        manifest_path: PathBuf,

        #[clap(long)]
        host_platform: Option<Platform>,
    },
    /// Build a conda package.
    CondaBuild {
        #[clap(env, long, env = "PIXI_PROJECT_MANIFEST", default_value = consts::WORKSPACE_MANIFEST)]
        manifest_path: PathBuf,
    },
    /// Build a conda package using the API v1 request format.
    CondaBuildV1 {
        #[clap(env, long, env = "PIXI_PROJECT_MANIFEST", default_value = consts::WORKSPACE_MANIFEST)]
        manifest_path: PathBuf,

        /// Path to a file containing the CondaBuildV1Params (YAML or JSON). Use '-' to read from stdin.
        #[clap(long)]
        params: PathBuf,

        /// Override the work directory provided in the params file.
        #[clap(long)]
        work_directory: Option<PathBuf>,

        /// Override the output directory provided in the params file.
        #[clap(long)]
        output_directory: Option<PathBuf>,
    },
    /// Get the capabilities of the backend.
    Capabilities,
}

/// Run the sever on the specified port or over stdin/stdout.
async fn run_server<T: ProtocolInstantiator>(port: Option<u16>, protocol: T) -> miette::Result<()> {
    let server = Server::new(protocol);
    if let Some(port) = port {
        server.run_over_http(port)
    } else {
        // running over stdin/stdout
        server.run().await
    }
}

/// The actual implementation of the main function that runs the CLI.
pub(crate) async fn main_impl<T: ProtocolInstantiator, F: FnOnce(LoggingOutputHandler) -> T>(
    factory: F,
    args: App,
) -> miette::Result<()> {
    // Setup logging
    let log_handler = LoggingOutputHandler::default();

    let registry = tracing_subscriber::registry()
        .with(get_default_env_filter(args.verbose.log_level_filter()).into_diagnostic()?);

    registry.with(log_handler.clone()).init();

    let factory = factory(log_handler);

    match args.command {
        None => run_server(args.http_port, factory).await,
        Some(Commands::Capabilities) => {
            let backend_capabilities = capabilities::<T>().await?;
            eprintln!(
                "Supports {}: {}",
                pixi_build_types::procedures::conda_metadata::METHOD_NAME,
                backend_capabilities
                    .provides_conda_metadata
                    .unwrap_or_default()
            );
            eprintln!(
                "Supports {}: {}",
                pixi_build_types::procedures::conda_outputs::METHOD_NAME,
                backend_capabilities
                    .provides_conda_outputs
                    .unwrap_or_default()
            );
            eprintln!(
                "Supports {}: {}",
                pixi_build_types::procedures::conda_build_v0::METHOD_NAME,
                backend_capabilities
                    .provides_conda_build
                    .unwrap_or_default()
            );
            eprintln!(
                "Supports {}: {}",
                pixi_build_types::procedures::conda_build_v1::METHOD_NAME,
                backend_capabilities
                    .provides_conda_build_v1
                    .unwrap_or_default()
            );
            eprintln!(
                "Highest project model: {}",
                backend_capabilities
                    .highest_supported_project_model
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| String::from("None"))
            );
            Ok(())
        }
        Some(Commands::CondaBuild { manifest_path }) => {
            let backend_capabilities = capabilities::<T>().await?;
            if backend_capabilities
                .provides_conda_build
                .unwrap_or_default()
            {
                build(factory, &manifest_path).await
            } else if backend_capabilities
                .provides_conda_build_v1
                .unwrap_or_default()
            {
                Err(miette!(
                    "Backend reports support for '{}' but not '{}'. Use 'conda-build-v1' instead.",
                    pixi_build_types::procedures::conda_build_v1::METHOD_NAME,
                    pixi_build_types::procedures::conda_build_v0::METHOD_NAME
                ))
            } else {
                Err(miette!(
                    "Backend does not report '{}' support.",
                    pixi_build_types::procedures::conda_build_v0::METHOD_NAME
                ))
            }
        }
        Some(Commands::CondaBuildV1 {
            manifest_path,
            params,
            work_directory,
            output_directory,
        }) => {
            let backend_capabilities = capabilities::<T>().await?;
            if !backend_capabilities
                .provides_conda_build_v1
                .unwrap_or_default()
            {
                return Err(miette!(
                    "Backend does not report '{}' support.",
                    pixi_build_types::procedures::conda_build_v1::METHOD_NAME
                ));
            }

            build_v1(
                factory,
                &manifest_path,
                &params,
                work_directory,
                output_directory,
            )
            .await
        }
        Some(Commands::GetCondaMetadata {
            manifest_path,
            host_platform,
        }) => {
            let metadata = conda_get_metadata(factory, &manifest_path, host_platform).await?;
            println!("{}", serde_yaml::to_string(&metadata).unwrap());
            Ok(())
        }
    }
}

/// The entry point for the CLI which should be called from the backends implementation.
pub async fn main<T: ProtocolInstantiator, F: FnOnce(LoggingOutputHandler) -> T>(
    factory: F,
) -> miette::Result<()> {
    let args = App::parse();
    main_impl(factory, args).await
}

/// The entry point for the CLI which should be called from the backends implementation.
pub async fn main_ext<T: ProtocolInstantiator, F: FnOnce(LoggingOutputHandler) -> T>(
    factory: F,
    args: Vec<String>,
) -> miette::Result<()> {
    let args = App::parse_from(args);
    main_impl(factory, args).await
}

/// Negotiate the capabilities of the backend and initialize the backend.
async fn initialize<T: ProtocolInstantiator>(
    factory: T,
    manifest_path: &Path,
) -> miette::Result<Box<dyn Protocol + Send + Sync + 'static>> {
    // Negotiate the capabilities of the backend.
    let capabilities = capabilities::<T>().await?;
    let channel_config = ChannelConfig::default_with_root_dir(
        manifest_path
            .parent()
            .expect("manifest should always reside in a directory")
            .to_path_buf(),
    );
    let project_model = to_project_model(
        manifest_path,
        &channel_config,
        capabilities.highest_supported_project_model,
    )?;

    // Check if the project model is required
    // and if it is not present, return an error.
    if capabilities.highest_supported_project_model.is_some() && project_model.is_none() {
        miette::bail!(
            "Could not extract 'project_model' from: {}, while it is required",
            manifest_path.display()
        );
    }

    // Initialize the backend
    let (protocol, _initialize_result) = factory
        .initialize(InitializeParams {
            workspace_root: None,
            source_dir: None,
            manifest_path: manifest_path.to_path_buf(),
            project_model,
            cache_directory: None,
            configuration: None,
            target_configuration: None,
        })
        .await?;
    Ok(protocol)
}

/// Frontend implementation for getting conda metadata.
async fn conda_get_metadata<T: ProtocolInstantiator>(
    factory: T,
    manifest_path: &Path,
    host_platform: Option<Platform>,
) -> miette::Result<CondaMetadataResult> {
    let channel_config = ChannelConfig::default_with_root_dir(
        manifest_path
            .parent()
            .expect("manifest should always reside in a directory")
            .to_path_buf(),
    );

    let protocol = initialize(factory, manifest_path).await?;

    let virtual_packages: Vec<_> = VirtualPackage::detect(&VirtualPackageOverrides::from_env())
        .into_diagnostic()?
        .into_iter()
        .map(GenericVirtualPackage::from)
        .collect();

    let tempdir = TempDir::new_in(".")
        .into_diagnostic()
        .context("failed to create a temporary directory in the current directory")?;

    protocol
        .conda_get_metadata(CondaMetadataParams {
            build_platform: None,
            host_platform: host_platform.map(|platform| PlatformAndVirtualPackages {
                platform,
                virtual_packages: Some(virtual_packages.clone()),
            }),
            channel_base_urls: None,
            channel_configuration: ChannelConfiguration {
                base_url: channel_config.channel_alias,
            },
            work_directory: tempdir.path().to_path_buf(),
            variant_configuration: None,
        })
        .await
}

/// Returns the capabilities of the backend.
async fn capabilities<Factory: ProtocolInstantiator>() -> miette::Result<BackendCapabilities> {
    let result = Factory::negotiate_capabilities(NegotiateCapabilitiesParams {
        capabilities: FrontendCapabilities {},
    })
    .await?;

    Ok(result.capabilities)
}

/// Frontend implementation for building a conda package.
async fn build<T: ProtocolInstantiator>(factory: T, manifest_path: &Path) -> miette::Result<()> {
    let channel_config = ChannelConfig::default_with_root_dir(
        manifest_path
            .parent()
            .expect("manifest should always reside in a directory")
            .to_path_buf(),
    );

    let protocol = initialize(factory, manifest_path).await?;
    let work_dir = TempDir::new_in(".")
        .into_diagnostic()
        .context("failed to create a temporary directory in the current directory")?;

    let result = protocol
        .conda_build_v0(CondaBuildParams {
            host_platform: None,
            build_platform_virtual_packages: None,
            channel_base_urls: None,
            channel_configuration: ChannelConfiguration {
                base_url: channel_config.channel_alias,
            },
            outputs: None,
            work_directory: work_dir.path().to_path_buf(),
            variant_configuration: None,
            editable: false,
        })
        .await?;

    for package in result.packages {
        eprintln!("Successfully build '{}'", package.output_file.display());
        eprintln!("Use following globs to revalidate: ");
        for glob in package.input_globs {
            eprintln!("  - {glob}");
        }
    }

    Ok(())
}

/// Build a package using the API v1 request format.
async fn build_v1<T: ProtocolInstantiator>(
    factory: T,
    manifest_path: &Path,
    params_path: &Path,
    work_directory: Option<PathBuf>,
    output_directory: Option<PathBuf>,
) -> miette::Result<()> {
    let mut params = load_conda_build_v1_params(params_path)?;

    if let Some(work_dir) = work_directory {
        create_dir_all(&work_dir)
            .into_diagnostic()
            .with_context(|| format!("failed to create work directory '{}'", work_dir.display()))?;
        params.work_directory = work_dir;
    } else {
        create_dir_all(&params.work_directory)
            .into_diagnostic()
            .with_context(|| {
                format!(
                    "failed to create work directory '{}'",
                    params.work_directory.display()
                )
            })?;
    }

    if let Some(out_dir) = output_directory {
        create_dir_all(&out_dir)
            .into_diagnostic()
            .with_context(|| {
                format!("failed to create output directory '{}'", out_dir.display())
            })?;
        params.output_directory = Some(out_dir);
    } else if let Some(out_dir) = params.output_directory.as_ref() {
        create_dir_all(out_dir).into_diagnostic().with_context(|| {
            format!("failed to create output directory '{}'", out_dir.display())
        })?;
    }

    let protocol = initialize(factory, manifest_path).await?;
    let result = protocol.conda_build_v1(params).await?;

    print_conda_build_v1_result(&result);

    println!(
        "{}",
        serde_yaml::to_string(&result)
            .into_diagnostic()
            .context("failed to serialize conda-build-v1 result")?
    );

    Ok(())
}

fn print_conda_build_v1_result(result: &CondaBuildV1Result) {
    eprintln!("Successfully built '{}'", result.output_file.display());
    eprintln!("Output metadata:");
    eprintln!("  name: {}", result.name);
    eprintln!("  version: {}", result.version);
    eprintln!("  build: {}", result.build);
    eprintln!("  subdir: {}", result.subdir);
    eprintln!("Use the following globs to revalidate:");
    for glob in &result.input_globs {
        eprintln!("  - {glob}");
    }
}

fn load_conda_build_v1_params(path: &Path) -> miette::Result<CondaBuildV1Params> {
    let (source, raw) = read_params_source(path)?;
    serde_yaml::from_str(&raw)
        .into_diagnostic()
        .with_context(|| format!("failed to parse CondaBuildV1Params from {source}"))
}

fn read_params_source(path: &Path) -> miette::Result<(String, String)> {
    if path == Path::new("-") {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .into_diagnostic()
            .context("failed to read CondaBuildV1Params from stdin")?;
        Ok((String::from("stdin"), buffer))
    } else {
        let text = read_to_string(path).into_diagnostic().with_context(|| {
            format!(
                "failed to read CondaBuildV1Params from '{}'",
                path.display()
            )
        })?;
        Ok((path.display().to_string(), text))
    }
}
