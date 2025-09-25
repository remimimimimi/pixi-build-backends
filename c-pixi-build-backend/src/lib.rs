//! Types:
//! - [x] Platform
//! - [X] Metadata provider
//! - [X] Project model
//! - [X] Python params
//! - [x] Config
//! - [x] Generated recipe
//!
//! Recipe stage 0:
//! - [ ] Conditional requirements
//! - [ ] Conditional
//! - [ ] Recipe
//! - [ ] Requirements
//!
//! Other:
//! - [ ] CLI
//! - [ ] Errors
use miette::{Diagnostic, IntoDiagnostic, WrapErr};
use pixi_build_backend::{
    NormalizedKey, Variable, generated_recipe,
    intermediate_backend::IntermediateBackendInstantiator,
};
use rattler_conda_types::{
    MatchSpec, ParseStrictness, StringMatcher, Version, VersionSpec, package::EntryPoint,
};
use rattler_digest::{Md5, Sha256, parse_digest_from_hex};
use recipe_stage0::matchspec::PackageDependency as StagePackageDependency;
use recipe_stage0::recipe::{
    About as StageAbout, Conditional as StageConditional, Extra as StageExtra,
    IntermediateRecipe as StageIntermediateRecipe, Item as StageItem,
    ListOrItem as StageListOrItem, NoArchKind as StageNoArchKind,
    PackageContents as StagePackageContents, PathSource as StagePathSource, Source as StageSource,
    Test as StageTest, UrlSource as StageUrlSource, Value as StageValue,
};
use safer_ffi::{boxed, prelude::*, slice::Ref as SliceRef};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value as SerdeValue, map::Entry as JsonEntry};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    ffi::CString,
    mem,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use url::Url;

// =======Platform=======
use rattler_conda_types::Platform as CondaPlatform;

/// A `struct` usable from both Rust and C
#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Platform {
    NoArch,
    Unknown,

    Linux32,
    Linux64,
    LinuxAarch64,
    LinuxArmV6l,
    LinuxArmV7l,
    LinuxLoong64,
    LinuxPpc64le,
    LinuxPpc64,
    LinuxPpc,
    LinuxS390X,
    LinuxRiscv32,
    LinuxRiscv64,

    Osx64,
    OsxArm64,

    Win32,
    Win64,
    WinArm64,

    EmscriptenWasm32,
    WasiWasm32,

    ZosZ,
}

impl From<CondaPlatform> for Platform {
    fn from(value: CondaPlatform) -> Self {
        match value {
            CondaPlatform::NoArch => Platform::NoArch,
            CondaPlatform::Unknown => Platform::Unknown,
            CondaPlatform::Linux32 => Platform::Linux32,
            CondaPlatform::Linux64 => Platform::Linux64,
            CondaPlatform::LinuxAarch64 => Platform::LinuxAarch64,
            CondaPlatform::LinuxArmV6l => Platform::LinuxArmV6l,
            CondaPlatform::LinuxArmV7l => Platform::LinuxArmV7l,
            CondaPlatform::LinuxLoong64 => Platform::LinuxLoong64,
            CondaPlatform::LinuxPpc64le => Platform::LinuxPpc64le,
            CondaPlatform::LinuxPpc64 => Platform::LinuxPpc64,
            CondaPlatform::LinuxPpc => Platform::LinuxPpc,
            CondaPlatform::LinuxS390X => Platform::LinuxS390X,
            CondaPlatform::LinuxRiscv32 => Platform::LinuxRiscv32,
            CondaPlatform::LinuxRiscv64 => Platform::LinuxRiscv64,
            CondaPlatform::Osx64 => Platform::Osx64,
            CondaPlatform::OsxArm64 => Platform::OsxArm64,
            CondaPlatform::Win32 => Platform::Win32,
            CondaPlatform::Win64 => Platform::Win64,
            CondaPlatform::WinArm64 => Platform::WinArm64,
            CondaPlatform::EmscriptenWasm32 => Platform::EmscriptenWasm32,
            CondaPlatform::WasiWasm32 => Platform::WasiWasm32,
            CondaPlatform::ZosZ => Platform::ZosZ,
            _ => Platform::Unknown,
        }
    }
}

impl From<Platform> for CondaPlatform {
    fn from(value: Platform) -> Self {
        match value {
            Platform::NoArch => CondaPlatform::NoArch,
            Platform::Unknown => CondaPlatform::Unknown,
            Platform::Linux32 => CondaPlatform::Linux32,
            Platform::Linux64 => CondaPlatform::Linux64,
            Platform::LinuxAarch64 => CondaPlatform::LinuxAarch64,
            Platform::LinuxArmV6l => CondaPlatform::LinuxArmV6l,
            Platform::LinuxArmV7l => CondaPlatform::LinuxArmV7l,
            Platform::LinuxLoong64 => CondaPlatform::LinuxLoong64,
            Platform::LinuxPpc64le => CondaPlatform::LinuxPpc64le,
            Platform::LinuxPpc64 => CondaPlatform::LinuxPpc64,
            Platform::LinuxPpc => CondaPlatform::LinuxPpc,
            Platform::LinuxS390X => CondaPlatform::LinuxS390X,
            Platform::LinuxRiscv32 => CondaPlatform::LinuxRiscv32,
            Platform::LinuxRiscv64 => CondaPlatform::LinuxRiscv64,
            Platform::Osx64 => CondaPlatform::Osx64,
            Platform::OsxArm64 => CondaPlatform::OsxArm64,
            Platform::Win32 => CondaPlatform::Win32,
            Platform::Win64 => CondaPlatform::Win64,
            Platform::WinArm64 => CondaPlatform::WinArm64,
            Platform::EmscriptenWasm32 => CondaPlatform::EmscriptenWasm32,
            Platform::WasiWasm32 => CondaPlatform::WasiWasm32,
            Platform::ZosZ => CondaPlatform::ZosZ,
            // _ => CondaPlatform::Unknown,
        }
    }
}

#[ffi_export]
fn ppb_platform_current() -> Platform {
    Platform::from(CondaPlatform::current())
}

/// Don't forget to free returned value.
#[ffi_export]
fn ppb_platform_name(p: &Platform) -> char_p::Box {
    CondaPlatform::from(*p).to_string().try_into().unwrap()
}

#[ffi_export]
fn ppb_platform_is_windows(p: &Platform) -> bool {
    CondaPlatform::from(*p).is_windows()
}

#[ffi_export]
fn ppb_platform_is_linux(p: &Platform) -> bool {
    CondaPlatform::from(*p).is_linux()
}

#[ffi_export]
fn ppb_platform_is_osx(p: &Platform) -> bool {
    CondaPlatform::from(*p).is_osx()
}

#[ffi_export]
fn ppb_platform_is_unix(p: &Platform) -> bool {
    CondaPlatform::from(*p).is_unix()
}

#[ffi_export]
fn ppb_platform_only_platform(p: &Platform) -> Option<char_p::Box> {
    CondaPlatform::from(*p)
        .only_platform()
        .map(|p| p.to_owned().try_into().unwrap())
}

// ===Metadata Provider==

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum MetadataProviderStatus {
    Ok,
    Err,
}

pub type ErrorMessage = Option<char_p::Box>;

#[derive_ReprC(dyn)]
pub trait CMetadataProvider {
    fn name(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn version(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn homepage(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn license(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn license_file(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn summary(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn description(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn documentation(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
    fn repository(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
}

#[derive_ReprC]
#[repr(C)]
pub struct MetadataProvider {
    provider: VirtualPtr<dyn CMetadataProvider>,
}

#[derive(Debug, Error, Diagnostic)]
#[error("{message}")]
pub struct ForeignMetadataProviderError {
    message: String,
}

impl ForeignMetadataProviderError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        let message = message.into();
        let message = if message.is_empty() {
            format!("{field} metadata callback failed")
        } else {
            format!("{field}: {message}")
        };
        Self { message }
    }
}

impl MetadataProvider {
    fn handle_result(
        field: &'static str,
        err: ErrorMessage,
        output: &mut Option<char_p::Box>,
    ) -> Result<Option<String>, ForeignMetadataProviderError> {
        match err {
            Some(msg) => Err(ForeignMetadataProviderError::new(field, msg.into_string())),
            None => {
                let value = mem::take(output);
                Ok(value.map(|s| s.into_string()))
            }
        }
    }
}

impl generated_recipe::MetadataProvider for MetadataProvider {
    type Error = ForeignMetadataProviderError;

    fn name(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.name(&mut output);
        Self::handle_result("name", result, &mut output)
    }

    fn version(&mut self) -> Result<Option<Version>, Self::Error> {
        let mut output = None;
        let result = self.provider.version(&mut output);
        let value = Self::handle_result("version", result, &mut output)?;
        match value {
            Some(version) => version
                .parse::<Version>()
                .map(Some)
                .map_err(|err| ForeignMetadataProviderError::new("version", err.to_string())),
            None => Ok(None),
        }
    }

    fn homepage(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.homepage(&mut output);
        Self::handle_result("homepage", result, &mut output)
    }

    fn license(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.license(&mut output);
        Self::handle_result("license", result, &mut output)
    }

    fn license_file(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.license_file(&mut output);
        Self::handle_result("license_file", result, &mut output)
    }

    fn summary(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.summary(&mut output);
        Self::handle_result("summary", result, &mut output)
    }

    fn description(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.description(&mut output);
        Self::handle_result("description", result, &mut output)
    }

    fn documentation(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.documentation(&mut output);
        Self::handle_result("documentation", result, &mut output)
    }

    fn repository(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = None;
        let result = self.provider.repository(&mut output);
        Self::handle_result("repository", result, &mut output)
    }
}

#[ffi_export]
pub fn ppb_metadata_provider_new(provider: VirtualPtr<dyn CMetadataProvider>) -> MetadataProvider {
    MetadataProvider { provider }
}

// ====Project model====

#[derive_ReprC]
#[repr(C)]
pub struct BinaryPackageSpecV1 {
    /// The version spec of the package (e.g. `1.2.3`, `>=1.2.3`, `1.2.*`)
    pub version: Option<char_p::Box>,
    /// The build string of the package (e.g. `py37_0`, `py37h6de7cb9_0`, `py*`)
    pub build: Option<char_p::Box>,
    /// The build number of the package
    pub build_number: Option<char_p::Box>,
    /// Match the specific filename of the package
    pub file_name: Option<char_p::Box>,
    /// The channel of the package
    pub channel: Option<char_p::Box>,
    /// The subdir of the channel
    pub subdir: Option<char_p::Box>,
    /// The md5 hash of the package
    pub md5: Option<char_p::Box>,
    /// The sha256 hash of the package
    pub sha256: Option<char_p::Box>,
    /// The URL of the package, if it is available
    pub url: Option<char_p::Box>,
    /// The license of the package
    pub license: Option<char_p::Box>,
}

#[derive(Debug, Error)]
pub enum BinaryPackageSpecV1ConversionError {
    #[error("invalid version spec '{input}': {message}")]
    InvalidVersionSpec { input: String, message: String },
    #[error("invalid build matcher '{input}': {message}")]
    InvalidBuildMatcher { input: String, message: String },
    #[error("invalid build number '{input}': {message}")]
    InvalidBuildNumber { input: String, message: String },
    #[error("invalid channel URL '{input}': {message}")]
    InvalidChannel { input: String, message: String },
    #[error("invalid md5 digest '{input}': {message}")]
    InvalidMd5 { input: String, message: String },
    #[error("invalid sha256 digest '{input}': {message}")]
    InvalidSha256 { input: String, message: String },
    #[error("invalid source URL '{input}': {message}")]
    InvalidUrl { input: String, message: String },
}

impl TryFrom<BinaryPackageSpecV1> for pixi_build_types::BinaryPackageSpecV1 {
    type Error = BinaryPackageSpecV1ConversionError;

    fn try_from(value: BinaryPackageSpecV1) -> Result<Self, Self::Error> {
        let version = value
            .version
            .map(|s| {
                let input = s.to_str().to_owned();
                VersionSpec::from_str(&input, ParseStrictness::Lenient).map_err(|err| {
                    BinaryPackageSpecV1ConversionError::InvalidVersionSpec {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let build = value
            .build
            .map(|s| {
                let input = s.to_str().to_owned();
                StringMatcher::from_str(&input).map_err(|err| {
                    BinaryPackageSpecV1ConversionError::InvalidBuildMatcher {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let build_number = value
            .build_number
            .map(|s| {
                let input = s.to_str().to_owned();
                rattler_conda_types::BuildNumberSpec::from_str(&input).map_err(|err| {
                    BinaryPackageSpecV1ConversionError::InvalidBuildNumber {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let file_name = value.file_name.map(|n| n.into_string());

        let channel = value
            .channel
            .map(|u| {
                let input = u.to_str().to_owned();
                Url::parse(&input).map_err(|err| {
                    BinaryPackageSpecV1ConversionError::InvalidChannel {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let subdir = value.subdir.map(|s| s.into_string());

        let md5 = value
            .md5
            .map(|md5| {
                let input = md5.to_str().to_owned();
                parse_digest_from_hex::<Md5>(&input)
                    .map(Into::into)
                    .ok_or_else(|| BinaryPackageSpecV1ConversionError::InvalidMd5 {
                        input,
                        message: "invalid hexadecimal digest".to_string(),
                    })
            })
            .transpose()?;

        let sha256 = value
            .sha256
            .map(|sha256| {
                let input = sha256.to_str().to_owned();
                parse_digest_from_hex::<Sha256>(&input)
                    .map(Into::into)
                    .ok_or_else(|| BinaryPackageSpecV1ConversionError::InvalidSha256 {
                        input,
                        message: "invalid hexadecimal digest".to_string(),
                    })
            })
            .transpose()?;

        let url = value
            .url
            .map(|u| {
                let input = u.to_str().to_owned();
                Url::parse(&input).map_err(|err| BinaryPackageSpecV1ConversionError::InvalidUrl {
                    input,
                    message: err.to_string(),
                })
            })
            .transpose()?;

        let license = value.license.map(|l| l.into_string());

        Ok(Self {
            version,
            build,
            build_number,
            file_name,
            channel,
            subdir,
            md5,
            sha256,
            url,
            license,
        })
    }
}

#[derive_ReprC]
#[repr(C)]
pub struct UrlSpecV1 {
    pub url: char_p::Box,
    pub md5: Option<char_p::Box>,
    pub sha256: Option<char_p::Box>,
}

#[derive(Debug, Error)]
pub enum UrlSpecV1ConversionError {
    #[error("invalid URL '{input}': {message}")]
    InvalidUrl { input: String, message: String },
    #[error("invalid md5 digest '{input}': {message}")]
    InvalidMd5 { input: String, message: String },
    #[error("invalid sha256 digest '{input}': {message}")]
    InvalidSha256 { input: String, message: String },
}

impl TryFrom<UrlSpecV1> for pixi_build_types::UrlSpecV1 {
    type Error = UrlSpecV1ConversionError;

    fn try_from(value: UrlSpecV1) -> Result<Self, Self::Error> {
        let url_input = value.url.to_str().to_owned();
        let url = Url::parse(&url_input).map_err(|err| UrlSpecV1ConversionError::InvalidUrl {
            input: url_input,
            message: err.to_string(),
        })?;

        let md5 = value
            .md5
            .map(|md5| {
                let input = md5.to_str().to_owned();
                parse_digest_from_hex::<Md5>(&input)
                    .map(Into::into)
                    .ok_or_else(|| UrlSpecV1ConversionError::InvalidMd5 {
                        input,
                        message: "invalid hexadecimal digest".to_string(),
                    })
            })
            .transpose()?;

        let sha256 = value
            .sha256
            .map(|sha256| {
                let input = sha256.to_str().to_owned();
                parse_digest_from_hex::<Sha256>(&input)
                    .map(Into::into)
                    .ok_or_else(|| UrlSpecV1ConversionError::InvalidSha256 {
                        input,
                        message: "invalid hexadecimal digest".to_string(),
                    })
            })
            .transpose()?;

        Ok(Self { url, md5, sha256 })
    }
}

/// A reference to a specific commit in a git repository.
#[derive_ReprC]
#[repr(opaque)]
pub struct GitReferenceV1 {
    inner: pixi_build_types::GitReferenceV1,
}

#[ffi_export]
pub fn pbb_git_reference_v1_branch_new(branch: char_p::Box) -> boxed::Box<GitReferenceV1> {
    let string = branch.into_string();
    Box::new(GitReferenceV1 {
        inner: pixi_build_types::GitReferenceV1::Branch(string),
    })
    .into()
}

#[ffi_export]
pub fn pbb_git_reference_v1_tag_new(tag: char_p::Box) -> boxed::Box<GitReferenceV1> {
    let string = tag.into_string();
    Box::new(GitReferenceV1 {
        inner: pixi_build_types::GitReferenceV1::Tag(string),
    })
    .into()
}

#[ffi_export]
pub fn pbb_git_reference_v1_rev_new(rev: char_p::Box) -> boxed::Box<GitReferenceV1> {
    let string = rev.into_string();
    Box::new(GitReferenceV1 {
        inner: pixi_build_types::GitReferenceV1::Rev(string),
    })
    .into()
}

#[ffi_export]
pub fn pbb_git_reference_v1_default_branch_new() -> boxed::Box<GitReferenceV1> {
    Box::new(GitReferenceV1 {
        inner: pixi_build_types::GitReferenceV1::DefaultBranch,
    })
    .into()
}

#[derive_ReprC]
#[repr(C)]
pub struct GitSpecV1 {
    /// The git url of the package which can contain git+ prefixes.
    pub git: char_p::Box,

    /// The git revision of the package
    pub rev: Option<boxed::Box<GitReferenceV1>>,

    /// The git subdirectory of the package
    pub subdirectory: Option<char_p::Box>,
}

#[derive(Debug, Error)]
pub enum GitSpecV1ConversionError {
    #[error("invalid git URL '{input}': {message}")]
    InvalidGitUrl { input: String, message: String },
}

impl TryFrom<GitSpecV1> for pixi_build_types::GitSpecV1 {
    type Error = GitSpecV1ConversionError;

    fn try_from(value: GitSpecV1) -> Result<Self, Self::Error> {
        let git_input = value.git.to_str().to_owned();
        let git =
            Url::parse(&git_input).map_err(|err| GitSpecV1ConversionError::InvalidGitUrl {
                input: git_input,
                message: err.to_string(),
            })?;

        Ok(Self {
            git,
            rev: value.rev.map(|r| r.into().inner),
            subdirectory: value.subdirectory.map(|subdir| subdir.into_string()),
        })
    }
}

#[derive_ReprC]
#[repr(C)]
pub struct PathSpecV1 {
    pub path: char_p::Box,
}

// TODO: Error handling
impl From<PathSpecV1> for pixi_build_types::PathSpecV1 {
    fn from(value: PathSpecV1) -> Self {
        Self {
            path: value.path.into_string(),
        }
    }
}

#[derive_ReprC]
#[repr(opaque)]
pub struct SourcePackageSpecV1 {
    inner: pixi_build_types::SourcePackageSpecV1,
}

#[ffi_export]
pub fn ppb_source_package_spec_v1_url_new(
    spec: UrlSpecV1,
    output: &mut Option<boxed::Box<SourcePackageSpecV1>>,
) -> ErrorMessage {
    match pixi_build_types::UrlSpecV1::try_from(spec) {
        Ok(url_spec) => {
            *output = Some(
                Box::new(SourcePackageSpecV1 {
                    inner: pixi_build_types::SourcePackageSpecV1::Url(url_spec),
                })
                .into(),
            );
            None
        }
        Err(err) => {
            *output = None;
            error_message(&err.to_string())
        }
    }
}

#[ffi_export]
pub fn ppb_source_package_spec_v1_git_new(
    spec: GitSpecV1,
    output: &mut Option<boxed::Box<SourcePackageSpecV1>>,
) -> ErrorMessage {
    match pixi_build_types::GitSpecV1::try_from(spec) {
        Ok(git_spec) => {
            *output = Some(
                Box::new(SourcePackageSpecV1 {
                    inner: pixi_build_types::SourcePackageSpecV1::Git(git_spec),
                })
                .into(),
            );
            None
        }
        Err(err) => {
            *output = None;
            error_message(&err.to_string())
        }
    }
}

// TODO: Error handling
#[ffi_export]
pub fn ppb_source_package_spec_v1_path_new(spec: PathSpecV1) -> boxed::Box<SourcePackageSpecV1> {
    Box::new(SourcePackageSpecV1 {
        inner: pixi_build_types::SourcePackageSpecV1::Path(spec.into()),
    })
    .into()
}

#[derive_ReprC]
#[repr(opaque)]
#[derive(Clone)]
pub struct PackageSpecV1 {
    inner: pixi_build_types::PackageSpecV1,
}

#[ffi_export]
pub fn ppb_package_spec_v1_binary_new(
    spec: BinaryPackageSpecV1,
    output: &mut Option<boxed::Box<PackageSpecV1>>,
) -> ErrorMessage {
    match pixi_build_types::BinaryPackageSpecV1::try_from(spec) {
        Ok(binary_spec) => {
            *output = Some(
                Box::new(PackageSpecV1 {
                    inner: pixi_build_types::PackageSpecV1::Binary(Box::new(binary_spec)),
                })
                .into(),
            );
            None
        }
        Err(err) => {
            *output = None;
            error_message(&err.to_string())
        }
    }
}

#[ffi_export]
pub fn ppb_package_spec_v1_source_new(
    spec: boxed::Box<SourcePackageSpecV1>,
    output: &mut Option<boxed::Box<PackageSpecV1>>,
) -> ErrorMessage {
    let spec_box: Box<SourcePackageSpecV1> = spec.into();
    *output = Some(
        Box::new(PackageSpecV1 {
            inner: pixi_build_types::PackageSpecV1::Source(spec_box.inner),
        })
        .into(),
    );
    None
}

type SourcePackageName = char_p::Box;

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct PackageMap {
    name: SourcePackageName,
    spec: boxed::Box<PackageSpecV1>,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct TargetV1 {
    /// Host dependencies of the project
    pub host_dependencies: Option<safer_ffi::Vec<PackageMap>>,

    /// Build dependencies of the project
    pub build_dependencies: Option<safer_ffi::Vec<PackageMap>>,

    /// Run dependencies of the project
    pub run_dependencies: Option<safer_ffi::Vec<PackageMap>>,
}

impl From<TargetV1> for pixi_build_types::TargetV1 {
    fn from(value: TargetV1) -> Self {
        Self {
            host_dependencies: value.host_dependencies.map(|d| {
                d.into_iter()
                    .map(|p| {
                        let p = p.clone();
                        let name = p.name.into_string();
                        let spec = p.spec.into().inner;
                        (name, spec)
                    })
                    .collect()
            }),
            build_dependencies: value.build_dependencies.map(|d| {
                d.into_iter()
                    .map(|p| {
                        let p = p.clone();
                        let name = p.name.into_string();
                        let spec = p.spec.into().inner;
                        (name, spec)
                    })
                    .collect()
            }),
            run_dependencies: value.run_dependencies.map(|d| {
                d.into_iter()
                    .map(|p| {
                        let p = p.clone();
                        let name = p.name.into_string();
                        let spec = p.spec.into().inner;
                        (name, spec)
                    })
                    .collect()
            }),
        }
    }
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone)]
pub enum TargetSelectorV1Kind {
    // Platform specific configuration
    Unix,
    Linux,
    Win,
    MacOs,
    Platform,
    // TODO: Add minijinja coolness here.
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct TargetSelectorV1 {
    kind: TargetSelectorV1Kind,
    // Non null only if kind is `Platform`.
    platform: Option<char_p::Box>,
}

#[derive(Debug, Error)]
pub enum TargetSelectorV1ConversionError {
    #[error("platform selector requires a platform value")]
    MissingPlatform,
}

impl TryFrom<TargetSelectorV1> for pixi_build_types::TargetSelectorV1 {
    type Error = TargetSelectorV1ConversionError;

    fn try_from(value: TargetSelectorV1) -> Result<Self, Self::Error> {
        Ok(match value.kind {
            TargetSelectorV1Kind::Unix => Self::Unix,
            TargetSelectorV1Kind::Linux => Self::Linux,
            TargetSelectorV1Kind::Win => Self::Win,
            TargetSelectorV1Kind::MacOs => Self::MacOs,
            TargetSelectorV1Kind::Platform => {
                let platform = value
                    .platform
                    .ok_or(TargetSelectorV1ConversionError::MissingPlatform)?;
                Self::Platform(platform.into_string())
            }
        })
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct TargetMap {
    selector: TargetSelectorV1,
    // Non null only if kind is `Platform`.
    target: boxed::Box<TargetV1>,
}

#[derive_ReprC]
#[repr(C)]
pub struct TargetsV1 {
    pub default_target: Option<safer_ffi::boxed::Box<TargetV1>>,

    /// We use an [`OrderMap`] to preserve the order in which the items where
    /// defined in the manifest.
    pub targets: Option<safer_ffi::Vec<TargetMap>>,
}

#[derive(Debug, Error)]
pub enum TargetsV1ConversionError {
    #[error("failed to convert target selector at index {index}: {source}")]
    Selector {
        index: usize,
        #[source]
        source: TargetSelectorV1ConversionError,
    },
}

impl TryFrom<TargetsV1> for pixi_build_types::TargetsV1 {
    type Error = TargetsV1ConversionError;

    fn try_from(value: TargetsV1) -> Result<Self, Self::Error> {
        let default_target = value.default_target.map(|t| (*t.into()).into());

        let targets = value
            .targets
            .map(|targets| {
                targets
                    .into_iter()
                    .enumerate()
                    .map(|(index, map)| {
                        let map = map.clone();
                        let selector = map.selector.try_into().map_err(|source| {
                            TargetsV1ConversionError::Selector { index, source }
                        })?;
                        let target = (*map.target.into()).into();
                        Ok((selector, target))
                    })
                    .collect::<Result<_, TargetsV1ConversionError>>()
            })
            .transpose()?;

        Ok(Self {
            default_target,
            targets,
        })
    }
}

#[derive_ReprC]
#[repr(C)]
pub struct ProjectModelV1 {
    /// The name of the project
    pub name: Option<char_p::Box>,

    /// The version of the project
    pub version: Option<char_p::Box>,

    /// An optional project description
    pub description: Option<char_p::Box>,

    /// Optional authors
    pub authors: Option<safer_ffi::Vec<char_p::Box>>,

    /// The license as a valid SPDX string (e.g. MIT AND Apache-2.0)
    pub license: Option<char_p::Box>,

    /// The license file (relative to the project root)
    pub license_file: Option<char_p::Box>,

    /// Path to the README file of the project (relative to the project root)
    pub readme: Option<char_p::Box>,

    /// URL of the project homepage
    pub homepage: Option<char_p::Box>,

    /// URL of the project source repository
    pub repository: Option<char_p::Box>,

    /// URL of the project documentation
    pub documentation: Option<char_p::Box>,

    /// The target of the project, this may contain
    /// platform specific configurations.
    pub targets: Option<safer_ffi::boxed::Box<TargetsV1>>,
}

#[derive(Debug, Error)]
pub enum ProjectModelV1ConversionError {
    #[error("invalid version '{input}': {message}")]
    InvalidVersion { input: String, message: String },
    #[error("invalid homepage URL '{input}': {message}")]
    InvalidHomepage { input: String, message: String },
    #[error("invalid repository URL '{input}': {message}")]
    InvalidRepository { input: String, message: String },
    #[error("invalid documentation URL '{input}': {message}")]
    InvalidDocumentation { input: String, message: String },
    #[error("failed to convert targets: {source}")]
    Targets {
        #[from]
        source: TargetsV1ConversionError,
    },
}

impl TryFrom<ProjectModelV1> for pixi_build_types::ProjectModelV1 {
    type Error = ProjectModelV1ConversionError;

    fn try_from(value: ProjectModelV1) -> Result<Self, Self::Error> {
        let name = value.name.map(|s| s.into_string());

        let version = value
            .version
            .map(|s| {
                let input = s.to_str().to_owned();
                Version::from_str(&input).map_err(|err| {
                    ProjectModelV1ConversionError::InvalidVersion {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let description = value.description.map(|s| s.into_string());

        let authors = value.authors.map(|authors| {
            authors
                .into_iter()
                .map(|author| author.clone().into_string())
                .collect()
        });

        let license = value.license.map(|s| s.into_string());

        let license_file = value.license_file.map(|s| PathBuf::from(s.into_string()));

        let readme = value.readme.map(|s| PathBuf::from(s.into_string()));

        let homepage = value
            .homepage
            .map(|s| {
                let input = s.to_str().to_owned();
                Url::parse(&input).map_err(|err| ProjectModelV1ConversionError::InvalidHomepage {
                    input,
                    message: err.to_string(),
                })
            })
            .transpose()?;

        let repository = value
            .repository
            .map(|s| {
                let input = s.to_str().to_owned();
                Url::parse(&input).map_err(|err| ProjectModelV1ConversionError::InvalidRepository {
                    input,
                    message: err.to_string(),
                })
            })
            .transpose()?;

        let documentation = value
            .documentation
            .map(|s| {
                let input = s.to_str().to_owned();
                Url::parse(&input).map_err(|err| {
                    ProjectModelV1ConversionError::InvalidDocumentation {
                        input,
                        message: err.to_string(),
                    }
                })
            })
            .transpose()?;

        let targets = value
            .targets
            .map(|t| pixi_build_types::TargetsV1::try_from(*t.into()))
            .transpose()?;

        Ok(Self {
            name,
            version,
            description,
            authors,
            license,
            license_file,
            readme,
            homepage,
            repository,
            documentation,
            targets,
        })
    }
}

// ====Python Params====
#[derive_ReprC]
#[repr(C)]
pub struct PythonParams {
    // Returns whether the build is editable or not.
    // Default to false
    pub editable: bool,
}

impl From<PythonParams> for pixi_build_backend::generated_recipe::PythonParams {
    fn from(value: PythonParams) -> Self {
        Self {
            editable: value.editable,
        }
    }
}

// =======Config========
#[derive_ReprC]
#[repr(opaque)]
#[derive(Clone)]
pub struct CBackendConfig {
    raw: SerdeValue,
    debug_dir: Option<PathBuf>,
}

impl<'de> Deserialize<'de> for CBackendConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = SerdeValue::deserialize(deserializer)?;
        let debug_dir = value
            .get("debug_dir")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        Ok(Self {
            raw: value,
            debug_dir,
        })
    }
}

impl generated_recipe::BackendConfig for CBackendConfig {
    fn debug_dir(&self) -> Option<&Path> {
        self.debug_dir.as_deref()
    }

    fn merge_with_target_config(&self, target_config: &Self) -> miette::Result<Self> {
        if target_config.debug_dir.is_some() {
            miette::bail!("`debug_dir` cannot have a target specific value");
        }

        let merged_raw = merge_config_values(&self.raw, &target_config.raw);

        Ok(Self {
            raw: merged_raw,
            debug_dir: self.debug_dir.clone(),
        })
    }
}

fn merge_config_values(base: &SerdeValue, target: &SerdeValue) -> SerdeValue {
    match (base, target) {
        (SerdeValue::Object(base_map), SerdeValue::Object(target_map)) => {
            let mut merged = base_map.clone();
            for (key, target_value) in target_map {
                match merged.entry(key.clone()) {
                    JsonEntry::Occupied(mut entry) => {
                        let combined = merge_config_values(entry.get(), target_value);
                        entry.insert(combined);
                    }
                    JsonEntry::Vacant(entry) => {
                        entry.insert(target_value.clone());
                    }
                }
            }
            SerdeValue::Object(merged)
        }
        (_, target_value) => target_value.clone(),
    }
}

#[derive_ReprC]
#[repr(opaque)]
#[derive(Clone)]
pub struct ConfigValue {
    inner: SerdeValue,
}

impl From<SerdeValue> for ConfigValue {
    fn from(inner: SerdeValue) -> Self {
        Self { inner }
    }
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ConfigValueKind {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

#[ffi_export]
pub fn pbb_backend_config_debug_dir(config: &CBackendConfig) -> Option<char_p::Box> {
    config
        .debug_dir
        .as_ref()
        .and_then(|path| path.to_string_lossy().to_string().try_into().ok())
}

#[ffi_export]
pub fn pbb_backend_config_raw_value(config: &CBackendConfig) -> boxed::Box<ConfigValue> {
    Box::new(ConfigValue::from(config.raw.clone())).into()
}

#[ffi_export]
pub fn pbb_backend_config_merge_with_target(
    base: &CBackendConfig,
    target: &CBackendConfig,
    output: &mut Option<boxed::Box<CBackendConfig>>,
) -> ErrorMessage {
    match generated_recipe::BackendConfig::merge_with_target_config(base, target) {
        Ok(merged) => {
            *output = Some(Box::new(merged).into());
            None
        }
        Err(err) => {
            *output = None;
            error_message(&err.to_string())
        }
    }
}

#[ffi_export]
pub fn pbb_config_value_kind(value: &ConfigValue) -> ConfigValueKind {
    match &value.inner {
        SerdeValue::Null => ConfigValueKind::Null,
        SerdeValue::Bool(_) => ConfigValueKind::Bool,
        SerdeValue::Number(_) => ConfigValueKind::Number,
        SerdeValue::String(_) => ConfigValueKind::String,
        SerdeValue::Array(_) => ConfigValueKind::Array,
        SerdeValue::Object(_) => ConfigValueKind::Object,
    }
}

#[ffi_export]
pub fn pbb_config_value_as_bool(value: &ConfigValue, output: &mut bool) -> bool {
    if let SerdeValue::Bool(inner) = &value.inner {
        *output = *inner;
        true
    } else {
        false
    }
}

#[ffi_export]
pub fn pbb_config_value_as_i64(value: &ConfigValue, output: &mut i64) -> bool {
    if let SerdeValue::Number(number) = &value.inner {
        number
            .as_i64()
            .map(|v| {
                *output = v;
                true
            })
            .unwrap_or(false)
    } else {
        false
    }
}

#[ffi_export]
pub fn pbb_config_value_as_u64(value: &ConfigValue, output: &mut u64) -> bool {
    if let SerdeValue::Number(number) = &value.inner {
        number
            .as_u64()
            .map(|v| {
                *output = v;
                true
            })
            .unwrap_or(false)
    } else {
        false
    }
}

#[ffi_export]
pub fn pbb_config_value_as_f64(value: &ConfigValue, output: &mut f64) -> bool {
    if let SerdeValue::Number(number) = &value.inner {
        number
            .as_f64()
            .map(|v| {
                *output = v;
                true
            })
            .unwrap_or(false)
    } else {
        false
    }
}

#[ffi_export]
pub fn pbb_config_value_as_string(value: &ConfigValue) -> Option<char_p::Box> {
    if let SerdeValue::String(string) = &value.inner {
        string.clone().try_into().ok()
    } else {
        None
    }
}

#[ffi_export]
pub fn pbb_config_value_array_len(value: &ConfigValue, output: &mut usize) -> bool {
    if let Some(array) = value.inner.as_array() {
        *output = array.len();
        true
    } else {
        false
    }
}

#[ffi_export]
pub fn pbb_config_value_array_get(
    value: &ConfigValue,
    index: usize,
) -> Option<boxed::Box<ConfigValue>> {
    value
        .inner
        .as_array()
        .and_then(|array| array.get(index))
        .map(|item| Box::new(ConfigValue::from(item.clone())).into())
}

#[ffi_export]
pub fn pbb_config_value_object_get(
    value: &ConfigValue,
    key: char_p::Ref<'_>,
) -> Option<boxed::Box<ConfigValue>> {
    value
        .inner
        .as_object()
        .and_then(|map| map.get(key.to_str()))
        .map(|item| Box::new(ConfigValue::from(item.clone())).into())
}

#[ffi_export]
pub fn pbb_config_value_to_json(value: &ConfigValue) -> Option<char_p::Box> {
    serde_json::to_string(&value.inner)
        .ok()
        .and_then(|json| json.try_into().ok())
}

#[ffi_export]
pub fn pbb_config_value_free(value: Option<boxed::Box<ConfigValue>>) {
    drop(value);
}

// ===Generated Recipe===

fn string_to_char_box(value: &str) -> miette::Result<char_p::Box> {
    let c_string =
        CString::new(value).map_err(|_| miette::miette!("failed to convert string to C string"))?;
    Ok(char_p::Box::from(c_string))
}

fn handle_callback_error(context: &'static str, error: ErrorMessage) -> miette::Result<()> {
    if let Some(message) = error {
        let message = message.into_string();
        if message.is_empty() {
            miette::bail!("{context} callback failed");
        } else {
            miette::bail!("{context}: {message}");
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct GeneratedRecipeJson {
    recipe: StageIntermediateRecipe,
    #[serde(default)]
    metadata_input_globs: BTreeSet<String>,
    #[serde(default)]
    build_input_globs: BTreeSet<String>,
}

impl From<generated_recipe::GeneratedRecipe> for GeneratedRecipeJson {
    fn from(value: generated_recipe::GeneratedRecipe) -> Self {
        Self {
            recipe: value.recipe,
            metadata_input_globs: value.metadata_input_globs,
            build_input_globs: value.build_input_globs,
        }
    }
}

impl From<GeneratedRecipeJson> for generated_recipe::GeneratedRecipe {
    fn from(value: GeneratedRecipeJson) -> Self {
        Self {
            recipe: value.recipe,
            metadata_input_globs: value.metadata_input_globs,
            build_input_globs: value.build_input_globs,
        }
    }
}

#[derive_ReprC]
#[repr(opaque)]
pub struct GeneratedRecipeHandle {
    recipe: generated_recipe::GeneratedRecipe,
}

impl GeneratedRecipeHandle {
    fn new(recipe: generated_recipe::GeneratedRecipe) -> Self {
        Self { recipe }
    }

    fn as_inner(&self) -> &generated_recipe::GeneratedRecipe {
        &self.recipe
    }

    fn as_inner_mut(&mut self) -> &mut generated_recipe::GeneratedRecipe {
        &mut self.recipe
    }
}

#[ffi_export]
pub fn pbb_generated_recipe_new_empty() -> boxed::Box<GeneratedRecipeHandle> {
    Box::new(GeneratedRecipeHandle::new(
        generated_recipe::GeneratedRecipe::default(),
    ))
    .into()
}

#[ffi_export]
pub fn pbb_generated_recipe_clone(
    recipe: &GeneratedRecipeHandle,
) -> boxed::Box<GeneratedRecipeHandle> {
    Box::new(GeneratedRecipeHandle::new(recipe.as_inner().clone())).into()
}

#[ffi_export]
pub fn pbb_generated_recipe_release(recipe: Option<boxed::Box<GeneratedRecipeHandle>>) {
    drop(recipe);
}

#[derive_ReprC]
#[repr(opaque)]
pub struct IntermediateRecipeHandle {
    inner: StageIntermediateRecipe,
}

impl IntermediateRecipeHandle {
    fn new(inner: StageIntermediateRecipe) -> Self {
        Self { inner }
    }

    fn as_inner(&self) -> &StageIntermediateRecipe {
        &self.inner
    }

    fn as_inner_mut(&mut self) -> &mut StageIntermediateRecipe {
        &mut self.inner
    }
}

fn value_from_input(input: char_p::Ref<'_>, is_template: bool) -> StageValue<String> {
    if is_template {
        StageValue::Template(input.to_str().to_string())
    } else {
        StageValue::Concrete(input.to_str().to_string())
    }
}

fn optional_value_from_input(
    input: Option<char_p::Ref<'_>>,
    is_template: bool,
) -> Option<StageValue<String>> {
    input.map(|value| value_from_input(value, is_template))
}

fn value_from_boxed(input: &char_p::Box, is_template: bool) -> StageValue<String> {
    if is_template {
        StageValue::Template(input.to_str().to_string())
    } else {
        StageValue::Concrete(input.to_str().to_string())
    }
}

fn optional_value_from_boxed(
    input: &Option<char_p::Box>,
    is_template: bool,
) -> Option<StageValue<String>> {
    input
        .as_ref()
        .map(|value| value_from_boxed(value, is_template))
}

fn add_requirement(
    list: &mut Vec<StageItem<StagePackageDependency>>,
    spec: char_p::Ref<'_>,
    is_template: bool,
) -> Result<(), String> {
    if is_template {
        list.push(StageItem::Value(StageValue::Template(
            spec.to_str().to_string(),
        )));
        Ok(())
    } else {
        let dependency = parse_dependency(spec.to_str())?;
        list.push(StageItem::Value(StageValue::Concrete(dependency)));
        Ok(())
    }
}

fn add_requirement_conditional(
    list: &mut Vec<StageItem<StagePackageDependency>>,
    condition: &str,
    then_specs: SliceRef<'_, RequirementInput>,
    else_specs: SliceRef<'_, RequirementInput>,
) -> Result<(), String> {
    let then = parse_requirement_inputs(then_specs)?;
    let else_branch = parse_requirement_inputs(else_specs)?;
    list.push(StageItem::Conditional(StageConditional {
        condition: condition.to_string(),
        then: StageListOrItem(then),
        else_value: StageListOrItem(else_branch),
    }));
    Ok(())
}

fn assign_string_to_output(value: String, output: &mut Option<char_p::Box>) -> Result<(), String> {
    char_p::Box::try_from(value)
        .map(|owned| {
            *output = Some(owned);
        })
        .map_err(|err| err.to_string())
}

fn set_output_from_string(value: String, output: &mut Option<char_p::Box>) -> ErrorMessage {
    match assign_string_to_output(value, output) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

fn parse_dependency(spec: &str) -> Result<StagePackageDependency, String> {
    let match_spec =
        MatchSpec::from_str(spec, ParseStrictness::Strict).map_err(|err| err.to_string())?;
    Ok(StagePackageDependency::Binary(match_spec))
}

fn parse_requirement_inputs(
    inputs: SliceRef<'_, RequirementInput>,
) -> Result<Vec<StagePackageDependency>, String> {
    inputs
        .iter()
        .map(|input| {
            if input.is_template {
                Err(
                    "template requirements are not supported inside conditional branches"
                        .to_string(),
                )
            } else {
                parse_dependency(input.spec.to_str())
            }
        })
        .collect()
}

fn modify_python_entry_points(
    recipe: &mut IntermediateRecipeHandle,
    f: impl FnOnce(&mut Vec<EntryPoint>) -> Result<(), String>,
) -> ErrorMessage {
    let entries = &mut recipe.as_inner_mut().build.python.entry_points;
    match f(entries) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

fn ensure_about(recipe: &mut StageIntermediateRecipe) -> &mut StageAbout {
    if recipe.about.is_none() {
        recipe.about = Some(StageAbout::default());
    }
    recipe.about.as_mut().unwrap()
}

fn ensure_extra(recipe: &mut StageIntermediateRecipe) -> &mut StageExtra {
    if recipe.extra.is_none() {
        recipe.extra = Some(StageExtra::default());
    }
    recipe.extra.as_mut().unwrap()
}

fn error_message(message: &str) -> ErrorMessage {
    CString::new(message)
        .ok()
        .and_then(|c_string| char_p::Box::try_from(c_string).ok())
}

fn get_test_mut<'a>(
    recipe: &'a mut IntermediateRecipeHandle,
    index: usize,
) -> Result<&'a mut StageTest, String> {
    recipe
        .as_inner_mut()
        .tests
        .get_mut(index)
        .ok_or_else(|| "test index out of bounds".to_string())
}

fn ensure_package_contents(test: &mut StageTest) -> &mut StagePackageContents {
    if test.package_contents.is_none() {
        test.package_contents = Some(StagePackageContents::default());
    }
    test.package_contents.as_mut().unwrap()
}

fn ensure_string_items(list: &mut Option<Vec<StageItem<String>>>) -> &mut Vec<StageItem<String>> {
    if list.is_none() {
        *list = Some(Vec::new());
    }
    list.as_mut().unwrap()
}

fn add_string_item(
    list: &mut Option<Vec<StageItem<String>>>,
    value: char_p::Ref<'_>,
    is_template: bool,
) {
    ensure_string_items(list).push(StageItem::Value(value_from_input(value, is_template)));
}

fn string_from_item(item: &StageItem<String>) -> Result<String, String> {
    match item {
        StageItem::Value(value) => Ok(value.to_string()),
        StageItem::Conditional(_) => Err("conditional values are not yet supported".to_string()),
    }
}

fn string_from_dependency(item: &StageItem<StagePackageDependency>) -> Result<String, String> {
    match item {
        StageItem::Value(value) => match value {
            StageValue::Concrete(dep) => match dep {
                StagePackageDependency::Binary(spec) => Ok(spec.to_string()),
                StagePackageDependency::Source(source_spec) => {
                    Ok(format!("{} @ {}", source_spec.spec, source_spec.location))
                }
            },
            StageValue::Template(template) => Ok(template.clone()),
        },
        StageItem::Conditional(cond) => {
            let then = cond
                .then
                .0
                .iter()
                .map(|dep| {
                    string_from_dependency(&StageItem::Value(StageValue::Concrete(dep.clone())))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let else_values = cond
                .else_value
                .0
                .iter()
                .map(|dep| {
                    string_from_dependency(&StageItem::Value(StageValue::Concrete(dep.clone())))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!(
                "if {} then [{}] else [{}]",
                cond.condition,
                then.join(", "),
                else_values.join(", ")
            ))
        }
    }
}

fn source_parts(source: StageSource) -> (SourceKind, String, Option<String>) {
    match source {
        StageSource::Url(url) => (
            SourceKind::Url,
            url.url.to_string(),
            url.sha256.map(|sha| sha.to_string()),
        ),
        StageSource::Path(path) => (
            SourceKind::Path,
            path.path.to_string(),
            path.sha256.map(|sha| sha.to_string()),
        ),
    }
}

fn requirement_branch<'a, T>(cond: &'a StageConditional<T>, branch: ConditionalBranch) -> &'a [T] {
    match branch {
        ConditionalBranch::Then => &cond.then.0,
        ConditionalBranch::Else => &cond.else_value.0,
    }
}

fn source_conditional<'a>(
    recipe: &'a IntermediateRecipeHandle,
    index: usize,
) -> Result<&'a StageConditional<StageSource>, String> {
    let sources = &recipe.as_inner().source;
    let Some(item) = sources.get(index) else {
        return Err("source index out of bounds".to_string());
    };
    match item {
        StageItem::Conditional(cond) => Ok(cond),
        _ => Err("source entry is not conditional".to_string()),
    }
}

fn source_conditional_mut<'a>(
    recipe: &'a mut IntermediateRecipeHandle,
    index: usize,
) -> Result<&'a mut StageConditional<StageSource>, String> {
    let sources = &mut recipe.as_inner_mut().source;
    let Some(item) = sources.get_mut(index) else {
        return Err("source index out of bounds".to_string());
    };
    match item {
        StageItem::Conditional(cond) => Ok(cond),
        _ => Err("source entry is not conditional".to_string()),
    }
}

fn source_branch_mut<'a>(
    cond: &'a mut StageConditional<StageSource>,
    branch: ConditionalBranch,
) -> &'a mut Vec<StageSource> {
    match branch {
        ConditionalBranch::Then => &mut cond.then.0,
        ConditionalBranch::Else => &mut cond.else_value.0,
    }
}

fn source_from_input(input: &SourceInput) -> Result<StageSource, String> {
    match input.kind {
        SourceKind::Url => Ok(StageSource::Url(StageUrlSource {
            url: value_from_boxed(&input.value, input.value_is_template),
            sha256: optional_value_from_boxed(&input.sha256, input.sha256_is_template),
        })),
        SourceKind::Path => Ok(StageSource::Path(StagePathSource {
            path: value_from_boxed(&input.value, input.value_is_template),
            sha256: optional_value_from_boxed(&input.sha256, input.sha256_is_template),
        })),
        SourceKind::Conditional => Err("invalid source kind for standalone entry".to_string()),
    }
}

fn parse_sources(inputs: SliceRef<'_, SourceInput>) -> Result<Vec<StageSource>, String> {
    inputs.iter().map(source_from_input).collect()
}

fn about_getter(
    recipe: &IntermediateRecipeHandle,
    f: impl Fn(&StageAbout) -> Option<&StageValue<String>>,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let Some(about) = recipe.as_inner().about.as_ref() else {
        *output = None;
        return None;
    };
    match f(about) {
        Some(value) => set_output_from_string(value.to_string(), output),
        None => {
            *output = None;
            None
        }
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_new() -> boxed::Box<IntermediateRecipeHandle> {
    Box::new(IntermediateRecipeHandle::new(
        StageIntermediateRecipe::default(),
    ))
    .into()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_clone(
    recipe: &IntermediateRecipeHandle,
) -> boxed::Box<IntermediateRecipeHandle> {
    Box::new(IntermediateRecipeHandle::new(recipe.as_inner().clone())).into()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_release(recipe: Option<boxed::Box<IntermediateRecipeHandle>>) {
    drop(recipe);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_from_yaml(
    yaml: char_p::Ref<'_>,
    output: &mut Option<boxed::Box<IntermediateRecipeHandle>>,
) -> ErrorMessage {
    match StageIntermediateRecipe::from_yaml(yaml.to_str()) {
        Ok(recipe) => {
            *output = Some(Box::new(IntermediateRecipeHandle::new(recipe)).into());
            None
        }
        Err(err) => error_message(&err.to_string()),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_to_yaml(recipe: &IntermediateRecipeHandle) -> Option<char_p::Box> {
    recipe
        .as_inner()
        .to_yaml()
        .ok()
        .and_then(|yaml| yaml.try_into().ok())
}

#[ffi_export]
pub fn pbb_generated_recipe_get_intermediate(
    recipe: &mut GeneratedRecipeHandle,
) -> boxed::Box<IntermediateRecipeHandle> {
    Box::new(IntermediateRecipeHandle::new(recipe.recipe.recipe.clone())).into()
}

#[ffi_export]
pub fn pbb_generated_recipe_set_intermediate(
    recipe: &mut GeneratedRecipeHandle,
    intermediate: boxed::Box<IntermediateRecipeHandle>,
) {
    let handle: Box<IntermediateRecipeHandle> = intermediate.into();
    recipe.recipe.recipe = handle.inner;
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_package(
    recipe: &mut IntermediateRecipeHandle,
    name: char_p::Ref<'_>,
    name_is_template: bool,
    version: char_p::Ref<'_>,
    version_is_template: bool,
) {
    let package = &mut recipe.as_inner_mut().package;
    package.name = value_from_input(name, name_is_template);
    package.version = value_from_input(version, version_is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_package(
    recipe: &IntermediateRecipeHandle,
    out_name: &mut Option<char_p::Box>,
    out_version: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let package = &recipe.as_inner().package;
    if let Some(err) = set_output_from_string(package.name.to_string(), out_name) {
        return Some(err);
    }
    if let Some(err) = set_output_from_string(package.version.to_string(), out_version) {
        return Some(err);
    }
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_clear_sources(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().source.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_source_url(
    recipe: &mut IntermediateRecipeHandle,
    url: char_p::Ref<'_>,
    url_is_template: bool,
    sha256: Option<char_p::Ref<'_>>,
    sha256_is_template: bool,
) {
    let source = StageSource::Url(StageUrlSource {
        url: value_from_input(url, url_is_template),
        sha256: optional_value_from_input(sha256, sha256_is_template),
    });
    recipe
        .as_inner_mut()
        .source
        .push(StageItem::Value(StageValue::Concrete(source)));
}

fn add_source_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: &str,
    then_inputs: SliceRef<'_, SourceInput>,
    else_inputs: SliceRef<'_, SourceInput>,
) -> Result<(), String> {
    let then_sources = parse_sources(then_inputs)?;
    let else_sources = parse_sources(else_inputs)?;
    recipe
        .as_inner_mut()
        .source
        .push(StageItem::Conditional(StageConditional {
            condition: condition.to_string(),
            then: StageListOrItem(then_sources),
            else_value: StageListOrItem(else_sources),
        }));
    Ok(())
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_source_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: char_p::Ref<'_>,
    then_sources: SliceRef<'_, SourceInput>,
    else_sources: SliceRef<'_, SourceInput>,
) -> ErrorMessage {
    match add_source_conditional(recipe, condition.to_str(), then_sources, else_sources) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_len(recipe: &IntermediateRecipeHandle) -> usize {
    recipe.as_inner().source.len()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_remove_at(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    let sources = &mut recipe.as_inner_mut().source;
    if index >= sources.len() {
        return error_message("source index out of bounds");
    }
    sources.remove(index);
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_kind: &mut SourceKind,
    out_value: &mut Option<char_p::Box>,
    out_sha256: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let sources = &recipe.as_inner().source;
    let Some(item) = sources.get(index) else {
        return error_message("source index out of bounds");
    };
    match item {
        StageItem::Value(value) => match value {
            StageValue::Concrete(source) => {
                let (kind_enum, val, sha) = source_parts(source.clone());
                *out_kind = kind_enum;
                if let Some(err) = set_output_from_string(val, out_value) {
                    return Some(err);
                }
                match sha {
                    Some(sha) => {
                        if let Some(err) = set_output_from_string(sha, out_sha256) {
                            return Some(err);
                        }
                    }
                    None => *out_sha256 = None,
                }
                None
            }
            StageValue::Template(template) => {
                error_message(&format!("template sources not supported: {}", template))
            }
        },
        StageItem::Conditional(cond) => {
            *out_kind = SourceKind::Conditional;
            if let Some(err) = set_output_from_string(cond.condition.clone(), out_value) {
                return Some(err);
            }
            *out_sha256 = None;
            None
        }
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_conditional_len(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    out_len: &mut usize,
) -> ErrorMessage {
    match source_conditional(recipe, index) {
        Ok(cond) => {
            *out_len = requirement_branch(cond, branch).len();
            None
        }
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_conditional_info(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    match source_conditional(recipe, index) {
        Ok(cond) => {
            if let Some(err) = set_output_from_string(cond.condition.clone(), out_condition) {
                return Some(err);
            }
            *out_then_len = cond.then.0.len();
            *out_else_len = cond.else_value.0.len();
            None
        }
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_conditional_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    out_kind: &mut SourceKind,
    out_value: &mut Option<char_p::Box>,
    out_sha256: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let cond = match source_conditional(recipe, index) {
        Ok(cond) => cond,
        Err(err) => return error_message(&err),
    };
    let list = requirement_branch(cond, branch);
    let Some(source) = list.get(branch_index) else {
        return error_message("source branch index out of bounds");
    };
    let (kind, value, sha) = source_parts(source.clone());
    *out_kind = kind;
    if let Some(err) = set_output_from_string(value, out_value) {
        return Some(err);
    }
    match sha {
        Some(sha) => {
            if let Some(err) = set_output_from_string(sha, out_sha256) {
                return Some(err);
            }
        }
        None => *out_sha256 = None,
    }
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_conditional_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> ErrorMessage {
    let cond = match source_conditional_mut(recipe, index) {
        Ok(cond) => cond,
        Err(err) => return error_message(&err),
    };
    let branch_list = source_branch_mut(cond, branch);
    if branch_index >= branch_list.len() {
        return error_message("source branch index out of bounds");
    }
    branch_list.remove(branch_index);
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_sources_conditional_add(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    source: SourceInput,
) -> ErrorMessage {
    let cond = match source_conditional_mut(recipe, index) {
        Ok(cond) => cond,
        Err(err) => return error_message(&err),
    };
    match source_from_input(&source) {
        Ok(stage_source) => {
            source_branch_mut(cond, branch).push(stage_source);
            None
        }
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_source_path(
    recipe: &mut IntermediateRecipeHandle,
    path: char_p::Ref<'_>,
    path_is_template: bool,
    sha256: Option<char_p::Ref<'_>>,
    sha256_is_template: bool,
) {
    let source = StageSource::Path(StagePathSource {
        path: value_from_input(path, path_is_template),
        sha256: optional_value_from_input(sha256, sha256_is_template),
    });
    recipe
        .as_inner_mut()
        .source
        .push(StageItem::Value(StageValue::Concrete(source)));
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum NoArchKind {
    None,
    Python,
    Generic,
}

fn convert_noarch_kind(kind: NoArchKind) -> Option<StageNoArchKind> {
    match kind {
        NoArchKind::None => None,
        NoArchKind::Python => Some(StageNoArchKind::Python),
        NoArchKind::Generic => Some(StageNoArchKind::Generic),
    }
}

fn noarch_from_option(value: &Option<StageNoArchKind>) -> NoArchKind {
    match value {
        Some(StageNoArchKind::Python) => NoArchKind::Python,
        Some(StageNoArchKind::Generic) => NoArchKind::Generic,
        None => NoArchKind::None,
    }
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum SourceKind {
    Url,
    Path,
    Conditional,
}

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ConditionalBranch {
    Then,
    Else,
}

#[derive_ReprC]
#[repr(C)]
pub struct SourceInput {
    pub kind: SourceKind,
    pub value: char_p::Box,
    pub value_is_template: bool,
    pub sha256: Option<char_p::Box>,
    pub sha256_is_template: bool,
}

#[derive_ReprC]
#[repr(C)]
pub struct RequirementInput {
    pub spec: char_p::Box,
    pub is_template: bool,
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_noarch(recipe: &mut IntermediateRecipeHandle, kind: NoArchKind) {
    recipe.as_inner_mut().build.noarch = convert_noarch_kind(kind);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_noarch(recipe: &IntermediateRecipeHandle) -> NoArchKind {
    noarch_from_option(&recipe.as_inner().build.noarch)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_build_number(
    recipe: &mut IntermediateRecipeHandle,
    has_number: bool,
    number: u64,
) {
    recipe.as_inner_mut().build.number = if has_number {
        Some(StageValue::Concrete(number))
    } else {
        None
    };
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_build_number(
    recipe: &IntermediateRecipeHandle,
    out_present: &mut bool,
    out_value: &mut u64,
) -> ErrorMessage {
    match &recipe.as_inner().build.number {
        Some(StageValue::Concrete(value)) => {
            *out_present = true;
            *out_value = *value;
            None
        }
        Some(StageValue::Template(template)) => {
            error_message(&format!("build number is a template: {}", template))
        }
        None => {
            *out_present = false;
            *out_value = 0;
            None
        }
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_build_script(
    recipe: &mut IntermediateRecipeHandle,
    script: char_p::Ref<'_>,
) {
    let build = &mut recipe.as_inner_mut().build;
    build.script.content = script.to_str().to_string();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_build_script(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    set_output_from_string(recipe.as_inner().build.script.content.clone(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_clear_env(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().build.script.env.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_set_env(
    recipe: &mut IntermediateRecipeHandle,
    key: char_p::Ref<'_>,
    value: char_p::Ref<'_>,
) {
    recipe
        .as_inner_mut()
        .build
        .script
        .env
        .insert(key.to_str().to_string(), value.to_str().to_string());
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_env_len(recipe: &IntermediateRecipeHandle) -> usize {
    recipe.as_inner().build.script.env.len()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_env_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_key: &mut Option<char_p::Box>,
    out_value: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let env = &recipe.as_inner().build.script.env;
    let Some((key, value)) = env.get_index(index) else {
        return error_message("env index out of bounds");
    };
    if let Some(err) = set_output_from_string(key.clone(), out_key) {
        return Some(err);
    }
    set_output_from_string(value.clone(), out_value)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_remove_env(
    recipe: &mut IntermediateRecipeHandle,
    key: char_p::Ref<'_>,
) {
    recipe
        .as_inner_mut()
        .build
        .script
        .env
        .shift_remove(key.to_str());
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_clear_secrets(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().build.script.secrets.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_add_secret(
    recipe: &mut IntermediateRecipeHandle,
    secret: char_p::Ref<'_>,
) {
    recipe
        .as_inner_mut()
        .build
        .script
        .secrets
        .push(secret.to_str().to_string());
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_secrets_len(
    recipe: &IntermediateRecipeHandle,
) -> usize {
    recipe.as_inner().build.script.secrets.len()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_build_script_secret_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let secrets = &recipe.as_inner().build.script.secrets;
    let Some(secret) = secrets.get(index) else {
        return error_message("secret index out of bounds");
    };
    set_output_from_string(secret.clone(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_clear_entry_points(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().build.python.entry_points.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_add_entry_point(
    recipe: &mut IntermediateRecipeHandle,
    command: char_p::Ref<'_>,
    module: char_p::Ref<'_>,
    function: char_p::Ref<'_>,
) {
    recipe
        .as_inner_mut()
        .build
        .python
        .entry_points
        .push(EntryPoint {
            command: command.to_str().to_string(),
            module: module.to_str().to_string(),
            function: function.to_str().to_string(),
        });
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_entry_points_len(recipe: &IntermediateRecipeHandle) -> usize {
    recipe.as_inner().build.python.entry_points.len()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_entry_point_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_command: &mut Option<char_p::Box>,
    out_module: &mut Option<char_p::Box>,
    out_function: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let entries = &recipe.as_inner().build.python.entry_points;
    let Some(entry) = entries.get(index) else {
        return error_message("entry point index out of bounds");
    };

    if let Err(err) = assign_string_to_output(entry.command.clone(), out_command) {
        return error_message(&err);
    }
    if let Err(err) = assign_string_to_output(entry.module.clone(), out_module) {
        return error_message(&err);
    }
    if let Err(err) = assign_string_to_output(entry.function.clone(), out_function) {
        return error_message(&err);
    }

    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_remove_entry_point_at(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    modify_python_entry_points(recipe, |entries| {
        if index >= entries.len() {
            return Err("entry point index out of bounds".to_string());
        }
        entries.remove(index);
        Ok(())
    })
}

#[ffi_export]
pub fn pbb_intermediate_recipe_python_is_default(recipe: &IntermediateRecipeHandle) -> bool {
    recipe.as_inner().build.python.is_default()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_clear_build(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().requirements.build.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_clear_host(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().requirements.host.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_clear_run(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().requirements.run.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_clear_run_constraints(
    recipe: &mut IntermediateRecipeHandle,
) {
    recipe.as_inner_mut().requirements.run_constraints.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_build_requirement(
    recipe: &mut IntermediateRecipeHandle,
    spec: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match add_requirement(
        &mut recipe.as_inner_mut().requirements.build,
        spec,
        is_template,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_host_requirement(
    recipe: &mut IntermediateRecipeHandle,
    spec: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match add_requirement(
        &mut recipe.as_inner_mut().requirements.host,
        spec,
        is_template,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_run_requirement(
    recipe: &mut IntermediateRecipeHandle,
    spec: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match add_requirement(
        &mut recipe.as_inner_mut().requirements.run,
        spec,
        is_template,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_run_constraint(
    recipe: &mut IntermediateRecipeHandle,
    spec: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match add_requirement(
        &mut recipe.as_inner_mut().requirements.run_constraints,
        spec,
        is_template,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_build_requirement_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: char_p::Ref<'_>,
    then_specs: SliceRef<'_, RequirementInput>,
    else_specs: SliceRef<'_, RequirementInput>,
) -> ErrorMessage {
    match add_requirement_conditional(
        &mut recipe.as_inner_mut().requirements.build,
        condition.to_str(),
        then_specs,
        else_specs,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_host_requirement_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: char_p::Ref<'_>,
    then_specs: SliceRef<'_, RequirementInput>,
    else_specs: SliceRef<'_, RequirementInput>,
) -> ErrorMessage {
    match add_requirement_conditional(
        &mut recipe.as_inner_mut().requirements.host,
        condition.to_str(),
        then_specs,
        else_specs,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_run_requirement_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: char_p::Ref<'_>,
    then_specs: SliceRef<'_, RequirementInput>,
    else_specs: SliceRef<'_, RequirementInput>,
) -> ErrorMessage {
    match add_requirement_conditional(
        &mut recipe.as_inner_mut().requirements.run,
        condition.to_str(),
        then_specs,
        else_specs,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_add_run_constraint_conditional(
    recipe: &mut IntermediateRecipeHandle,
    condition: char_p::Ref<'_>,
    then_specs: SliceRef<'_, RequirementInput>,
    else_specs: SliceRef<'_, RequirementInput>,
) -> ErrorMessage {
    match add_requirement_conditional(
        &mut recipe.as_inner_mut().requirements.run_constraints,
        condition.to_str(),
        then_specs,
        else_specs,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

fn requirement_list_len(list: &Vec<StageItem<StagePackageDependency>>) -> usize {
    list.len()
}

fn requirement_list_at(
    list: &Vec<StageItem<StagePackageDependency>>,
    index: usize,
) -> Result<String, String> {
    list.get(index)
        .ok_or_else(|| "requirement index out of bounds".to_string())
        .and_then(string_from_dependency)
}

fn requirement_conditional<'a>(
    list: &'a [StageItem<StagePackageDependency>],
    index: usize,
) -> Result<&'a StageConditional<StagePackageDependency>, String> {
    let Some(item) = list.get(index) else {
        return Err("requirement index out of bounds".to_string());
    };
    match item {
        StageItem::Conditional(cond) => Ok(cond),
        _ => Err("requirement is not conditional".to_string()),
    }
}

fn requirement_conditional_mut<'a>(
    list: &'a mut [StageItem<StagePackageDependency>],
    index: usize,
) -> Result<&'a mut StageConditional<StagePackageDependency>, String> {
    let Some(item) = list.get_mut(index) else {
        return Err("requirement index out of bounds".to_string());
    };
    match item {
        StageItem::Conditional(cond) => Ok(cond),
        _ => Err("requirement is not conditional".to_string()),
    }
}

fn requirement_branch_mut(
    cond: &mut StageConditional<StagePackageDependency>,
    branch: ConditionalBranch,
) -> &mut Vec<StagePackageDependency> {
    match branch {
        ConditionalBranch::Then => &mut cond.then.0,
        ConditionalBranch::Else => &mut cond.else_value.0,
    }
}

fn requirement_conditional_remove(
    list: &mut Vec<StageItem<StagePackageDependency>>,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> Result<(), String> {
    let cond = requirement_conditional_mut(list, index)?;
    let branch_list = requirement_branch_mut(cond, branch);
    if branch_index >= branch_list.len() {
        return Err("requirement branch index out of bounds".to_string());
    }
    branch_list.remove(branch_index);
    Ok(())
}

fn requirement_conditional_add(
    list: &mut Vec<StageItem<StagePackageDependency>>,
    index: usize,
    branch: ConditionalBranch,
    spec: char_p::Ref<'_>,
) -> Result<(), String> {
    let cond = requirement_conditional_mut(list, index)?;
    let dep = parse_dependency(spec.to_str())?;
    requirement_branch_mut(cond, branch).push(dep);
    Ok(())
}

fn requirement_conditional_info(
    list: &[StageItem<StagePackageDependency>],
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    match requirement_conditional(list, index) {
        Ok(cond) => {
            if let Some(err) = set_output_from_string(cond.condition.clone(), out_condition) {
                return Some(err);
            }
            *out_then_len = cond.then.0.len();
            *out_else_len = cond.else_value.0.len();
            None
        }
        Err(err) => error_message(&err),
    }
}

fn requirement_conditional_at(
    list: &[StageItem<StagePackageDependency>],
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let cond = match requirement_conditional(list, index) {
        Ok(cond) => cond,
        Err(err) => return error_message(&err),
    };
    let branch_items = requirement_branch(cond, branch);
    let Some(dep) = branch_items.get(branch_index) else {
        return error_message("requirement branch index out of bounds");
    };
    match string_from_dependency(&StageItem::Value(StageValue::Concrete(dep.clone()))) {
        Ok(value) => set_output_from_string(value, output),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_len(recipe: &IntermediateRecipeHandle) -> usize {
    requirement_list_len(&recipe.as_inner().requirements.build)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match requirement_list_at(&recipe.as_inner().requirements.build, index) {
        Ok(value) => set_output_from_string(value, output),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_len(recipe: &IntermediateRecipeHandle) -> usize {
    requirement_list_len(&recipe.as_inner().requirements.host)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match requirement_list_at(&recipe.as_inner().requirements.host, index) {
        Ok(value) => set_output_from_string(value, output),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_len(recipe: &IntermediateRecipeHandle) -> usize {
    requirement_list_len(&recipe.as_inner().requirements.run)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match requirement_list_at(&recipe.as_inner().requirements.run, index) {
        Ok(value) => set_output_from_string(value, output),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_len(
    recipe: &IntermediateRecipeHandle,
) -> usize {
    requirement_list_len(&recipe.as_inner().requirements.run_constraints)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match requirement_list_at(&recipe.as_inner().requirements.run_constraints, index) {
        Ok(value) => set_output_from_string(value, output),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match requirement_list_remove(&mut recipe.as_inner_mut().requirements.build, index) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match requirement_list_remove(&mut recipe.as_inner_mut().requirements.host, index) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match requirement_list_remove(&mut recipe.as_inner_mut().requirements.run, index) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match requirement_list_remove(
        &mut recipe.as_inner_mut().requirements.run_constraints,
        index,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_conditional_info(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    requirement_conditional_info(
        &recipe.as_inner().requirements.build,
        index,
        out_condition,
        out_then_len,
        out_else_len,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_conditional_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    requirement_conditional_at(
        &recipe.as_inner().requirements.build,
        index,
        branch,
        branch_index,
        output,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_conditional_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> ErrorMessage {
    match requirement_conditional_remove(
        &mut recipe.as_inner_mut().requirements.build,
        index,
        branch,
        branch_index,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_build_conditional_add(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    spec: char_p::Ref<'_>,
) -> ErrorMessage {
    match requirement_conditional_add(
        &mut recipe.as_inner_mut().requirements.build,
        index,
        branch,
        spec,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_conditional_info(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    requirement_conditional_info(
        &recipe.as_inner().requirements.host,
        index,
        out_condition,
        out_then_len,
        out_else_len,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_conditional_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    requirement_conditional_at(
        &recipe.as_inner().requirements.host,
        index,
        branch,
        branch_index,
        output,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_conditional_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> ErrorMessage {
    match requirement_conditional_remove(
        &mut recipe.as_inner_mut().requirements.host,
        index,
        branch,
        branch_index,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_host_conditional_add(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    spec: char_p::Ref<'_>,
) -> ErrorMessage {
    match requirement_conditional_add(
        &mut recipe.as_inner_mut().requirements.host,
        index,
        branch,
        spec,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_conditional_info(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    requirement_conditional_info(
        &recipe.as_inner().requirements.run,
        index,
        out_condition,
        out_then_len,
        out_else_len,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_conditional_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    requirement_conditional_at(
        &recipe.as_inner().requirements.run,
        index,
        branch,
        branch_index,
        output,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_conditional_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> ErrorMessage {
    match requirement_conditional_remove(
        &mut recipe.as_inner_mut().requirements.run,
        index,
        branch,
        branch_index,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_conditional_add(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    spec: char_p::Ref<'_>,
) -> ErrorMessage {
    match requirement_conditional_add(
        &mut recipe.as_inner_mut().requirements.run,
        index,
        branch,
        spec,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_conditional_info(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_condition: &mut Option<char_p::Box>,
    out_then_len: &mut usize,
    out_else_len: &mut usize,
) -> ErrorMessage {
    requirement_conditional_info(
        &recipe.as_inner().requirements.run_constraints,
        index,
        out_condition,
        out_then_len,
        out_else_len,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_conditional_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    requirement_conditional_at(
        &recipe.as_inner().requirements.run_constraints,
        index,
        branch,
        branch_index,
        output,
    )
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_conditional_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    branch_index: usize,
) -> ErrorMessage {
    match requirement_conditional_remove(
        &mut recipe.as_inner_mut().requirements.run_constraints,
        index,
        branch,
        branch_index,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_requirements_run_constraints_conditional_add(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    branch: ConditionalBranch,
    spec: char_p::Ref<'_>,
) -> ErrorMessage {
    match requirement_conditional_add(
        &mut recipe.as_inner_mut().requirements.run_constraints,
        index,
        branch,
        spec,
    ) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_tests_len(recipe: &IntermediateRecipeHandle) -> usize {
    recipe.as_inner().tests.len()
}

#[ffi_export]
pub fn pbb_intermediate_recipe_tests_clear(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().tests.clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_tests_push(recipe: &mut IntermediateRecipeHandle) {
    recipe.as_inner_mut().tests.push(StageTest::default());
}

#[ffi_export]
pub fn pbb_intermediate_recipe_tests_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    if index >= recipe.as_inner().tests.len() {
        return error_message("test index out of bounds");
    }
    recipe.as_inner_mut().tests.remove(index);
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_has_package_contents(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_present: &mut bool,
) -> ErrorMessage {
    let tests = &recipe.as_inner().tests;
    let Some(test) = tests.get(index) else {
        return error_message("test index out of bounds");
    };
    *out_present = test.package_contents.is_some();
    None
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_set_package_contents_present(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    present: bool,
) -> ErrorMessage {
    match get_test_mut(recipe, index) {
        Ok(test) => {
            if present {
                ensure_package_contents(test);
            } else {
                test.package_contents = None;
            }
            None
        }
        Err(err) => error_message(&err),
    }
}

fn with_package_contents_mut<F>(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    f: F,
) -> Result<(), String>
where
    F: FnOnce(&mut StagePackageContents) -> Result<(), String>,
{
    let test = get_test_mut(recipe, index)?;
    let contents = ensure_package_contents(test);
    f(contents)
}

fn map_package_contents<F, R>(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    f: F,
) -> Result<R, String>
where
    F: FnOnce(&StagePackageContents) -> R,
{
    let tests = &recipe.as_inner().tests;
    let test = tests
        .get(index)
        .ok_or_else(|| "test index out of bounds".to_string())?;
    let contents = test
        .package_contents
        .as_ref()
        .ok_or_else(|| "package contents not present".to_string())?;
    Ok(f(contents))
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_clear_include(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        contents.include = None;
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_clear_files(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        contents.files = None;
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_add_include(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    value: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        add_string_item(&mut contents.include, value, is_template);
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_add_file(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    value: char_p::Ref<'_>,
    is_template: bool,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        add_string_item(&mut contents.files, value, is_template);
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

fn string_list_len(list: &Option<Vec<StageItem<String>>>) -> usize {
    list.as_ref().map(|items| items.len()).unwrap_or(0)
}

fn string_list_at(list: &Option<Vec<StageItem<String>>>, index: usize) -> Result<String, String> {
    let items = list.as_ref().ok_or_else(|| "list is empty".to_string())?;
    let item = items
        .get(index)
        .ok_or_else(|| "index out of bounds".to_string())?;
    string_from_item(item)
}

fn string_list_remove(
    list: &mut Option<Vec<StageItem<String>>>,
    index: usize,
) -> Result<(), String> {
    let items = ensure_string_items(list);
    if index >= items.len() {
        return Err("index out of bounds".to_string());
    }
    items.remove(index);
    Ok(())
}

fn requirement_list_remove(
    list: &mut Vec<StageItem<StagePackageDependency>>,
    index: usize,
) -> Result<(), String> {
    if index >= list.len() {
        return Err("requirement index out of bounds".to_string());
    }
    list.remove(index);
    Ok(())
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_include_len(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_len: &mut usize,
) -> ErrorMessage {
    match map_package_contents(recipe, index, |contents| string_list_len(&contents.include)) {
        Ok(len) => {
            *out_len = len;
            None
        }
        Err(err) => {
            if err == "package contents not present" {
                *out_len = 0;
                None
            } else {
                error_message(&err)
            }
        }
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_include_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    entry_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match map_package_contents(recipe, index, |contents| {
        string_list_at(&contents.include, entry_index)
    }) {
        Ok(value) => match value {
            Ok(value) => set_output_from_string(value, output),
            Err(err) => error_message(&err),
        },
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_include_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    entry_index: usize,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        string_list_remove(&mut contents.include, entry_index)?;
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_files_len(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    out_len: &mut usize,
) -> ErrorMessage {
    match map_package_contents(recipe, index, |contents| string_list_len(&contents.files)) {
        Ok(len) => {
            *out_len = len;
            None
        }
        Err(err) => {
            if err == "package contents not present" {
                *out_len = 0;
                None
            } else {
                error_message(&err)
            }
        }
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_files_at(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    entry_index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    match map_package_contents(recipe, index, |contents| {
        string_list_at(&contents.files, entry_index)
    }) {
        Ok(value) => match value {
            Ok(value) => set_output_from_string(value, output),
            Err(err) => error_message(&err),
        },
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_test_package_contents_files_remove(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
    entry_index: usize,
) -> ErrorMessage {
    match with_package_contents_mut(recipe, index, |contents| {
        string_list_remove(&mut contents.files, entry_index)?;
        Ok(())
    }) {
        Ok(_) => None,
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_homepage(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).homepage = optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_homepage(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.homepage.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_license(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).license = optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_license(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.license.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_license_file(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).license_file =
        optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_license_file(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.license_file.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_summary(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).summary = optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_description(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).description = optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_description(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.description.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_documentation(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).documentation =
        optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_documentation(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.documentation.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_set_about_repository(
    recipe: &mut IntermediateRecipeHandle,
    value: Option<char_p::Ref<'_>>,
    is_template: bool,
) {
    ensure_about(recipe.as_inner_mut()).repository = optional_value_from_input(value, is_template);
}

#[ffi_export]
pub fn pbb_intermediate_recipe_get_about_repository(
    recipe: &IntermediateRecipeHandle,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    about_getter(recipe, |about| about.repository.as_ref(), output)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_extra_clear_maintainers(recipe: &mut IntermediateRecipeHandle) {
    ensure_extra(recipe.as_inner_mut())
        .recipe_maintainers
        .clear();
}

#[ffi_export]
pub fn pbb_intermediate_recipe_extra_add_maintainer(
    recipe: &mut IntermediateRecipeHandle,
    maintainer: char_p::Ref<'_>,
) {
    ensure_extra(recipe.as_inner_mut())
        .recipe_maintainers
        .push(StageItem::Value(StageValue::Concrete(
            maintainer.to_str().to_string(),
        )));
}

#[ffi_export]
pub fn pbb_intermediate_recipe_extra_maintainers_len(recipe: &IntermediateRecipeHandle) -> usize {
    recipe
        .as_inner()
        .extra
        .as_ref()
        .map(|extra| extra.recipe_maintainers.len())
        .unwrap_or(0)
}

#[ffi_export]
pub fn pbb_intermediate_recipe_extra_get_maintainer(
    recipe: &IntermediateRecipeHandle,
    index: usize,
    output: &mut Option<char_p::Box>,
) -> ErrorMessage {
    let Some(extra) = recipe.as_inner().extra.as_ref() else {
        *output = None;
        return None;
    };
    let Some(item) = extra.recipe_maintainers.get(index) else {
        return error_message("maintainer index out of bounds");
    };
    match string_from_item(item) {
        Ok(value) => assign_string_to_output(value, output)
            .err()
            .and_then(|err| error_message(&err)),
        Err(err) => error_message(&err),
    }
}

#[ffi_export]
pub fn pbb_intermediate_recipe_extra_remove_maintainer(
    recipe: &mut IntermediateRecipeHandle,
    index: usize,
) -> ErrorMessage {
    let extra = ensure_extra(recipe.as_inner_mut());
    if index >= extra.recipe_maintainers.len() {
        return error_message("maintainer index out of bounds");
    }
    extra.recipe_maintainers.remove(index);
    None
}

#[ffi_export]
pub fn pbb_generated_recipe_from_json(
    json: char_p::Ref<'_>,
    output: &mut Option<boxed::Box<GeneratedRecipeHandle>>,
) -> ErrorMessage {
    match serde_json::from_str::<GeneratedRecipeJson>(json.to_str()) {
        Ok(value) => {
            *output = Some(Box::new(GeneratedRecipeHandle::new(value.into())).into());
            None
        }
        Err(err) => error_message(&err.to_string()),
    }
}

#[ffi_export]
pub fn pbb_generated_recipe_to_json(recipe: &GeneratedRecipeHandle) -> Option<char_p::Box> {
    let json = serde_json::to_string(&GeneratedRecipeJson::from(recipe.as_inner().clone())).ok()?;
    json.try_into().ok()
}

#[ffi_export]
pub fn pbb_generated_recipe_metadata_glob_count(recipe: &GeneratedRecipeHandle) -> usize {
    recipe.as_inner().metadata_input_globs.len()
}

#[ffi_export]
pub fn pbb_generated_recipe_metadata_glob_at(
    recipe: &GeneratedRecipeHandle,
    index: usize,
) -> Option<char_p::Box> {
    recipe
        .as_inner()
        .metadata_input_globs
        .iter()
        .nth(index)
        .and_then(|glob| glob.clone().try_into().ok())
}

#[ffi_export]
pub fn pbb_generated_recipe_add_metadata_glob(
    recipe: &mut GeneratedRecipeHandle,
    glob: char_p::Ref<'_>,
) -> ErrorMessage {
    recipe
        .as_inner_mut()
        .metadata_input_globs
        .insert(glob.to_str().to_string());
    None
}

#[ffi_export]
pub fn pbb_generated_recipe_clear_metadata_globs(recipe: &mut GeneratedRecipeHandle) {
    recipe.as_inner_mut().metadata_input_globs.clear();
}

#[ffi_export]
pub fn pbb_generated_recipe_build_glob_count(recipe: &GeneratedRecipeHandle) -> usize {
    recipe.as_inner().build_input_globs.len()
}

#[ffi_export]
pub fn pbb_generated_recipe_build_glob_at(
    recipe: &GeneratedRecipeHandle,
    index: usize,
) -> Option<char_p::Box> {
    recipe
        .as_inner()
        .build_input_globs
        .iter()
        .nth(index)
        .and_then(|glob| glob.clone().try_into().ok())
}

#[ffi_export]
pub fn pbb_generated_recipe_add_build_glob(
    recipe: &mut GeneratedRecipeHandle,
    glob: char_p::Ref<'_>,
) -> ErrorMessage {
    recipe
        .as_inner_mut()
        .build_input_globs
        .insert(glob.to_str().to_string());
    None
}

#[ffi_export]
pub fn pbb_generated_recipe_clear_build_globs(recipe: &mut GeneratedRecipeHandle) {
    recipe.as_inner_mut().build_input_globs.clear();
}

#[derive_ReprC(dyn)]
pub trait CGenerator: Send {
    fn generate_recipe(
        &mut self,
        project_model_json: char_p::Ref<'_>,
        config_json: char_p::Ref<'_>,
        manifest_path: char_p::Ref<'_>,
        host_platform: char_p::Ref<'_>,
        editable: bool,
        variants_json: char_p::Ref<'_>,
        output: &mut Option<boxed::Box<GeneratedRecipeHandle>>,
    ) -> ErrorMessage;

    fn extract_input_globs_from_build(
        &mut self,
        config_json: char_p::Ref<'_>,
        workdir: char_p::Ref<'_>,
        editable: bool,
        output: &mut Option<char_p::Box>,
    ) -> ErrorMessage;

    fn default_variants(
        &mut self,
        host_platform: char_p::Ref<'_>,
        output: &mut Option<char_p::Box>,
    ) -> ErrorMessage;
}

#[derive_ReprC]
#[repr(opaque)]
#[derive(Clone)]
pub struct Generator {
    generator: Arc<Mutex<VirtualPtr<dyn CGenerator>>>,
}

#[ffi_export]
pub fn pbb_generator_new(generator: VirtualPtr<dyn CGenerator>) -> boxed::Box<Generator> {
    Box::new(Generator {
        generator: Arc::new(Mutex::new(generator)),
    })
    .into()
}

fn parse_variants_json(json: &str) -> miette::Result<BTreeMap<NormalizedKey, Vec<Variable>>> {
    let map: BTreeMap<String, Vec<String>> = serde_json::from_str(json)
        .into_diagnostic()
        .wrap_err("default_variants callback returned invalid JSON")?;

    let mut result = BTreeMap::new();
    for (key, values) in map {
        result.insert(
            NormalizedKey::from(key),
            values.into_iter().map(Variable::from).collect(),
        );
    }

    Ok(result)
}

fn parse_globs_json(json: &str, context: &'static str) -> miette::Result<BTreeSet<String>> {
    let list: Vec<String> = serde_json::from_str(json)
        .into_diagnostic()
        .wrap_err(format!("{context} callback returned invalid JSON"))?;
    Ok(list.into_iter().collect())
}

fn collect_cli_args(args: SliceRef<'_, char_p::Ref<'_>>) -> Vec<String> {
    if args.len() == 0 {
        return vec!["pixi-build-backend".to_string()];
    }

    args.iter().map(|arg| arg.to_str().to_string()).collect()
}

#[ffi_export]
pub fn pbb_cli_run(generator: &Generator, args: SliceRef<'_, char_p::Ref<'_>>) -> ErrorMessage {
    let argv = collect_cli_args(args);
    let generator_arc = Arc::new(generator.clone());

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(err) => return error_message(&format!("failed to create runtime: {err}")),
    };

    let result = runtime.block_on(async move {
        pixi_build_backend::cli::main_ext(
            move |log| IntermediateBackendInstantiator::new(log, Arc::clone(&generator_arc)),
            argv,
        )
        .await
    });

    match result {
        Ok(_) => None,
        Err(err) => error_message(&err.to_string()),
    }
}

impl generated_recipe::GenerateRecipe for Generator {
    type Config = CBackendConfig;

    fn generate_recipe(
        &self,
        model: &pixi_build_types::ProjectModelV1,
        config: &Self::Config,
        manifest_path: PathBuf,
        host_platform: rattler_conda_types::Platform,
        python_params: Option<generated_recipe::PythonParams>,
        variants: &HashSet<NormalizedKey>,
    ) -> miette::Result<generated_recipe::GeneratedRecipe> {
        let project_model_json = serde_json::to_string(model).into_diagnostic()?;
        let config_json = serde_json::to_string(&config.raw).into_diagnostic()?;
        let manifest_path_str = manifest_path.display().to_string();
        let host_platform_str = host_platform.to_string();
        let editable = python_params.map(|params| params.editable).unwrap_or(false);
        let variants_vec: Vec<String> = variants.iter().map(|v| v.0.clone()).collect();
        let variants_json = serde_json::to_string(&variants_vec).into_diagnostic()?;

        let project_model_c = string_to_char_box(&project_model_json)?;
        let config_c = string_to_char_box(&config_json)?;
        let manifest_c = string_to_char_box(&manifest_path_str)?;
        let host_platform_c = string_to_char_box(&host_platform_str)?;
        let variants_c = string_to_char_box(&variants_json)?;

        let mut output = None;
        let mut generator = self
            .generator
            .lock()
            .map_err(|_| miette::miette!("generator callback panicked"))?;

        let error = generator.generate_recipe(
            project_model_c.as_ref(),
            config_c.as_ref(),
            manifest_c.as_ref(),
            host_platform_c.as_ref(),
            editable,
            variants_c.as_ref(),
            &mut output,
        );

        handle_callback_error("generate_recipe", error)?;

        let Some(handle) = output.take() else {
            miette::bail!("generate_recipe callback did not provide a recipe handle");
        };

        let mut handle_box: Box<GeneratedRecipeHandle> = handle.into();
        let recipe = mem::replace(
            &mut handle_box.recipe,
            generated_recipe::GeneratedRecipe::default(),
        );
        Ok(recipe)
    }

    fn extract_input_globs_from_build(
        &self,
        config: &Self::Config,
        workdir: impl AsRef<Path>,
        editable: bool,
    ) -> miette::Result<BTreeSet<String>> {
        let config_json = serde_json::to_string(&config.raw).into_diagnostic()?;
        let workdir_str = workdir.as_ref().display().to_string();

        let config_c = string_to_char_box(&config_json)?;
        let workdir_c = string_to_char_box(&workdir_str)?;

        let mut output = None;
        let mut generator = self
            .generator
            .lock()
            .map_err(|_| miette::miette!("generator callback panicked"))?;

        let error = generator.extract_input_globs_from_build(
            config_c.as_ref(),
            workdir_c.as_ref(),
            editable,
            &mut output,
        );

        handle_callback_error("extract_input_globs_from_build", error)?;

        let Some(json) = output.map(|s| s.into_string()) else {
            return Ok(BTreeSet::new());
        };

        parse_globs_json(&json, "extract_input_globs_from_build")
    }

    fn default_variants(
        &self,
        host_platform: rattler_conda_types::Platform,
    ) -> miette::Result<BTreeMap<NormalizedKey, Vec<Variable>>> {
        let host_platform_str = host_platform.to_string();
        let host_platform_c = string_to_char_box(&host_platform_str)?;

        let mut output = None;
        let mut generator = self
            .generator
            .lock()
            .map_err(|_| miette::miette!("generator callback panicked"))?;

        let error = generator.default_variants(host_platform_c.as_ref(), &mut output);

        handle_callback_error("default_variants", error)?;

        let Some(json) = output.map(|s| s.into_string()) else {
            return Ok(BTreeMap::new());
        };

        parse_variants_json(&json)
    }
}

// // ======main=======

// #[derive_ReprC(dyn)]
// pub trait CGenerator {
//     fn name(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn version(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn homepage(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn license(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn license_file(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn summary(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn description(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn documentation(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
//     fn repository(&mut self, output: &mut Option<char_p::Box>) -> ErrorMessage;
// }

// #[derive_ReprC]
// #[repr(transparent)]
// pub struct Generator {
//     backend_config: BackConfig,
//     generator: VirtualPtr<dyn CGenerator>,
// }

// impl GenerateRecipe for Generator {
//     type Config = BackConfig;

//     fn generate_recipe(
//         &self,
//         model: &pixi_build_types::ProjectModelV1,
//         config: &Self::Config,
//         manifest_path: std::path::PathBuf,
//         host_platform: CondaPlatform,
//         python_params: Option<pixi_build_backend::generated_recipe::PythonParams>,
//         variants: &std::collections::HashSet<pixi_build_backend::NormalizedKey>,
//     ) -> miette::Result<pixi_build_backend::generated_recipe::GeneratedRecipe> {
//         todo!()
//     }
// }

// #[ffi_export]
// fn ppb_main_sync(generator: Generator, args: safer_ffi::vec::Vec<char_p::Ref<'_>>) {
//     pixi_build_backend::cli::main(|log| {
//         IntermediateBackendInstantiator::new(log, Arc::new(Mutex::new(generator)))
//     })
// }

// #[ffi_export]
// fn ppb_run_backend(instance: Generator) {
//     // Just run backend main in here
// }

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
pub fn generate_headers() -> ::std::io::Result<()> {
    safer_ffi::headers::builder()
        .to_file("pixi-build-backend.h")?
        .generate()
}

// use std::collections::{BTreeMap, BTreeSet, HashSet};
// use std::ffi::{CStr, CString};
// use std::os::raw::{c_char, c_void};
// use std::path::{Path, PathBuf};
// use std::slice;
// use std::str::FromStr;
// use std::sync::Arc;

// use miette::{Context, IntoDiagnostic};
// use pixi_build_backend::cli_main;
// use pixi_build_backend::generated_recipe::{
//     BackendConfig, DefaultMetadataProvider, GenerateRecipe, GeneratedRecipe, MetadataProvider,
//     PythonParams,
// };
// use pixi_build_backend::intermediate_backend::IntermediateBackendInstantiator;
// use pixi_build_backend::{NormalizedKey, Variable};
// use pixi_build_types::ProjectModelV1;
// use rattler_conda_types::{Platform, Version};
// use recipe_stage0::recipe::IntermediateRecipe;
// use serde::{Deserialize, Serialize};
// use tokio::runtime::Runtime;
// use url::Url;

// #[repr(C)]
// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
// pub enum PixiStatus {
//     #[default]
//     OK = 0,
//     Error = 1,
// }

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct PixiOwnedString {
//     pub data: *mut c_char,
//     pub len: usize,
// }

// impl Default for PixiOwnedString {
//     fn default() -> Self {
//         Self {
//             data: std::ptr::null_mut(),
//             len: 0,
//         }
//     }
// }

// impl PixiOwnedString {
//     fn from_string(string: String) -> Self {
//         match CString::new(string) {
//             Ok(cstr) => {
//                 let len = cstr.as_bytes().len();
//                 Self {
//                     data: cstr.into_raw(),
//                     len,
//                 }
//             }
//             Err(_) => Self::default(),
//         }
//     }
// }

// unsafe fn owned_string_to_string(value: PixiOwnedString) -> miette::Result<String> {
//     if value.data.is_null() {
//         return Ok(String::new());
//     }
//     let cstring = unsafe { CString::from_raw(value.data) };
//     cstring
//         .into_string()
//         .into_diagnostic()
//         .wrap_err("string returned from C contained invalid UTF-8")
// }

// fn set_error(out_error: *mut PixiOwnedString, error: String) {
//     if out_error.is_null() {
//         return;
//     }
//     unsafe {
//         *out_error = PixiOwnedString::from_string(error);
//     }
// }

// fn clear_error(out_error: *mut PixiOwnedString) {
//     if out_error.is_null() {
//         return;
//     }
//     unsafe {
//         *out_error = PixiOwnedString::default();
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_owned_string_copy(
//     data: *const c_char,
//     len: usize,
// ) -> PixiOwnedString {
//     if data.is_null() {
//         return PixiOwnedString::default();
//     }
//     let slice = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
//     let string = String::from_utf8_lossy(slice).into_owned();
//     PixiOwnedString::from_string(string)
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_owned_string_free(string: PixiOwnedString) {
//     if string.data.is_null() {
//         return;
//     }
//     unsafe {
//         drop(CString::from_raw(string.data));
//     }
// }

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// pub struct PixiOptionalString {
//     pub is_some: bool,
//     pub value: *const c_char,
// }

// impl Default for PixiOptionalString {
//     fn default() -> Self {
//         Self {
//             is_some: false,
//             value: std::ptr::null(),
//         }
//     }
// }

// impl PixiOptionalString {
//     fn to_option(&self) -> Option<String> {
//         if !self.is_some || self.value.is_null() {
//             return None;
//         }
//         let cstr = unsafe { CStr::from_ptr(self.value) };
//         Some(cstr.to_string_lossy().into_owned())
//     }
// }

// #[repr(C)]
// pub struct PixiProjectModelV1Opaque {
//     _private: [u8; 0],
// }

// type PixiProjectModelV1Ptr = *mut PixiProjectModelV1Opaque;

// fn project_model_from_ptr(ptr: PixiProjectModelV1Ptr) -> *mut ProjectModelV1 {
//     ptr.cast()
// }

// fn allocate_project_model(model: ProjectModelV1) -> PixiProjectModelV1Ptr {
//     Box::into_raw(Box::new(model)) as PixiProjectModelV1Ptr
// }

// unsafe fn c_optional_string(value: *const c_char) -> Result<Option<String>, String> {
//     if value.is_null() {
//         return Ok(None);
//     }
//     unsafe { CStr::from_ptr(value) }
//         .to_str()
//         .map(|s| Some(s.to_owned()))
//         .map_err(|e| e.to_string())
// }

// unsafe fn c_string_array(authors: *const *const c_char, len: usize) -> Result<Vec<String>, String> {
//     if len == 0 {
//         return Ok(Vec::new());
//     }
//     if authors.is_null() {
//         return Err("authors pointer was null".to_string());
//     }
//     let slice = unsafe { slice::from_raw_parts(authors, len) };
//     let mut result = Vec::with_capacity(len);
//     for &ptr in slice {
//         if ptr.is_null() {
//             return Err("author string pointer was null".to_string());
//         }
//         let s = unsafe { CStr::from_ptr(ptr) }
//             .to_str()
//             .map_err(|e| e.to_string())?
//             .to_owned();
//         result.push(s);
//     }
//     Ok(result)
// }

// fn assign_optional_owned_string(value: Option<String>, out: *mut PixiOwnedString) {
//     if out.is_null() {
//         return;
//     }
//     unsafe {
//         *out = value.map(PixiOwnedString::from_string).unwrap_or_default();
//     }
// }

// fn assign_optional_path(value: Option<PathBuf>, out: *mut PixiOwnedString) {
//     assign_optional_owned_string(value.map(|p| p.display().to_string()), out);
// }

// fn assign_optional_url(value: Option<Url>, out: *mut PixiOwnedString) {
//     assign_optional_owned_string(value.map(|u| u.to_string()), out);
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_new(
//     out_model: *mut PixiProjectModelV1Ptr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if out_model.is_null() {
//         set_error(out_error, "out_model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     unsafe {
//         *out_model = allocate_project_model(ProjectModelV1::default());
//     }
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_clone(
//     model: PixiProjectModelV1Ptr,
//     out_clone: *mut PixiProjectModelV1Ptr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     if out_clone.is_null() {
//         set_error(out_error, "out_clone pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let clone = unsafe { (*project_model_from_ptr(model)).clone() };
//     unsafe {
//         *out_clone = allocate_project_model(clone);
//     }
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_free(model: PixiProjectModelV1Ptr) {
//     if model.is_null() {
//         return;
//     }
//     unsafe {
//         drop(Box::from_raw(project_model_from_ptr(model)));
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_name(
//     model: PixiProjectModelV1Ptr,
//     name: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(name) } {
//         Ok(value) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).name = value;
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_version(
//     model: PixiProjectModelV1Ptr,
//     version: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(version) } {
//         Ok(Some(value)) => match Version::from_str(&value) {
//             Ok(parsed) => {
//                 clear_error(out_error);
//                 unsafe {
//                     (*project_model_from_ptr(model)).version = Some(parsed);
//                 }
//                 PixiStatus::OK
//             }
//             Err(err) => {
//                 set_error(out_error, err.to_string());
//                 PixiStatus::Error
//             }
//         },
//         Ok(None) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).version = None;
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_description(
//     model: PixiProjectModelV1Ptr,
//     description: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(description) } {
//         Ok(value) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).description = value;
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_license(
//     model: PixiProjectModelV1Ptr,
//     license: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(license) } {
//         Ok(value) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).license = value;
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_license_file(
//     model: PixiProjectModelV1Ptr,
//     license_file: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(license_file) } {
//         Ok(value) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).license_file = value.map(PathBuf::from);
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_readme(
//     model: PixiProjectModelV1Ptr,
//     readme: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(readme) } {
//         Ok(value) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).readme = value.map(PathBuf::from);
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// fn set_optional_url_field(
//     model: PixiProjectModelV1Ptr,
//     setter: fn(&mut ProjectModelV1) -> &mut Option<Url>,
//     value: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_optional_string(value) } {
//         Ok(Some(url)) => match Url::parse(&url) {
//             Ok(parsed) => {
//                 clear_error(out_error);
//                 unsafe {
//                     *setter(&mut *project_model_from_ptr(model)) = Some(parsed);
//                 }
//                 PixiStatus::OK
//             }
//             Err(err) => {
//                 set_error(out_error, err.to_string());
//                 PixiStatus::Error
//             }
//         },
//         Ok(None) => {
//             clear_error(out_error);
//             unsafe {
//                 *setter(&mut *project_model_from_ptr(model)) = None;
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_homepage(
//     model: PixiProjectModelV1Ptr,
//     homepage: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     set_optional_url_field(model, |m| &mut m.homepage, homepage, out_error)
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_repository(
//     model: PixiProjectModelV1Ptr,
//     repository: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     set_optional_url_field(model, |m| &mut m.repository, repository, out_error)
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_documentation(
//     model: PixiProjectModelV1Ptr,
//     documentation: *const c_char,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     set_optional_url_field(model, |m| &mut m.documentation, documentation, out_error)
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_set_authors(
//     model: PixiProjectModelV1Ptr,
//     authors: *const *const c_char,
//     len: usize,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     if authors.is_null() && len > 0 {
//         set_error(out_error, "authors pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     match unsafe { c_string_array(authors, len) } {
//         Ok(values) => {
//             clear_error(out_error);
//             unsafe {
//                 (*project_model_from_ptr(model)).authors = if values.is_empty() {
//                     None
//                 } else {
//                     Some(values)
//                 };
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_name(
//     model: PixiProjectModelV1Ptr,
//     out_name: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_owned_string(model_ref.name.clone(), out_name);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_version(
//     model: PixiProjectModelV1Ptr,
//     out_version: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_owned_string(
//         model_ref.version.clone().map(|v| v.to_string()),
//         out_version,
//     );
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_description(
//     model: PixiProjectModelV1Ptr,
//     out_description: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_owned_string(model_ref.description.clone(), out_description);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_license(
//     model: PixiProjectModelV1Ptr,
//     out_license: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_owned_string(model_ref.license.clone(), out_license);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_license_file(
//     model: PixiProjectModelV1Ptr,
//     out_license_file: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_path(model_ref.license_file.clone(), out_license_file);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_readme(
//     model: PixiProjectModelV1Ptr,
//     out_readme: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_path(model_ref.readme.clone(), out_readme);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_homepage(
//     model: PixiProjectModelV1Ptr,
//     out_homepage: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_url(model_ref.homepage.clone(), out_homepage);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_repository(
//     model: PixiProjectModelV1Ptr,
//     out_repository: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_url(model_ref.repository.clone(), out_repository);
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_project_model_get_documentation(
//     model: PixiProjectModelV1Ptr,
//     out_documentation: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let model_ref = unsafe { &*project_model_from_ptr(model) };
//     assign_optional_url(model_ref.documentation.clone(), out_documentation);
//     PixiStatus::OK
// }

// pub type PixiMetadataQueryFn = unsafe extern "C" fn(
//     ctx: *mut c_void,
//     out_value: *mut PixiOptionalString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus;

// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct PixiOptionalMetadataQueryFn {
//     pub is_some: bool,
//     pub callback: PixiMetadataQueryFn,
// }

// impl PixiOptionalMetadataQueryFn {
//     fn into_option(self) -> Option<PixiMetadataQueryFn> {
//         if self.is_some {
//             Some(self.callback)
//         } else {
//             None
//         }
//     }
// }

// #[repr(C)]
// pub struct PixiMetadataProvider {
//     pub ctx: *mut c_void,
//     pub name: PixiOptionalMetadataQueryFn,
//     pub version: PixiOptionalMetadataQueryFn,
//     pub homepage: PixiOptionalMetadataQueryFn,
//     pub license: PixiOptionalMetadataQueryFn,
//     pub license_file: PixiOptionalMetadataQueryFn,
//     pub summary: PixiOptionalMetadataQueryFn,
//     pub description: PixiOptionalMetadataQueryFn,
//     pub documentation: PixiOptionalMetadataQueryFn,
//     pub repository: PixiOptionalMetadataQueryFn,
// }

// #[derive(Debug, thiserror::Error, miette::Diagnostic)]
// #[error("{message}")]
// struct CMetadataProviderError {
//     message: String,
// }

// struct CMetadataProvider<'a> {
//     provider: &'a PixiMetadataProvider,
// }

// impl<'a> CMetadataProvider<'a> {
//     fn new(provider: &'a PixiMetadataProvider) -> Self {
//         Self { provider }
//     }

//     fn call(
//         &mut self,
//         cb: PixiOptionalMetadataQueryFn,
//         field: &str,
//     ) -> Result<Option<String>, CMetadataProviderError> {
//         let Some(callback) = cb.into_option() else {
//             return Ok(None);
//         };

//         let mut opt = PixiOptionalString::default();
//         let mut err = PixiOwnedString::default();
//         let status = unsafe { callback(self.provider.ctx, &mut opt, &mut err) };
//         match status {
//             PixiStatus::OK => {
//                 if !err.data.is_null() {
//                     unsafe {
//                         let _ = owned_string_to_string(err);
//                     }
//                 }
//                 Ok(opt.to_option())
//             }
//             PixiStatus::Error => {
//                 let message = unsafe { owned_string_to_string(err) }
//                     .unwrap_or_else(|_| "metadata callback failed".to_string());
//                 Err(CMetadataProviderError {
//                     message: format!("{field}: {message}"),
//                 })
//             }
//         }
//     }
// }

// impl<'a> MetadataProvider for CMetadataProvider<'a> {
//     type Error = CMetadataProviderError;

//     fn name(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.name, "name")
//     }

//     fn version(&mut self) -> Result<Option<Version>, Self::Error> {
//         if let Some(value) = self.call(self.provider.version, "version")? {
//             let version = value
//                 .parse::<Version>()
//                 .map_err(|e| CMetadataProviderError {
//                     message: format!("version: {e}"),
//                 })?;
//             Ok(Some(version))
//         } else {
//             Ok(None)
//         }
//     }

//     fn homepage(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.homepage, "homepage")
//     }

//     fn license(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.license, "license")
//     }

//     fn license_file(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.license_file, "license_file")
//     }

//     fn summary(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.summary, "summary")
//     }

//     fn description(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.description, "description")
//     }

//     fn documentation(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.documentation, "documentation")
//     }

//     fn repository(&mut self) -> Result<Option<String>, Self::Error> {
//         self.call(self.provider.repository, "repository")
//     }
// }

// #[derive(Clone)]
// struct CBackendConfig {
//     debug_dir: Option<PathBuf>,
//     raw: serde_json::Value,
// }

// impl<'de> Deserialize<'de> for CBackendConfig {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let value = serde_json::Value::deserialize(deserializer)?;
//         let debug_dir = value
//             .get("debug_dir")
//             .and_then(|v| v.as_str())
//             .map(PathBuf::from);
//         Ok(Self {
//             debug_dir,
//             raw: value,
//         })
//     }
// }

// impl BackendConfig for CBackendConfig {
//     fn debug_dir(&self) -> Option<&Path> {
//         self.debug_dir.as_deref()
//     }

//     fn merge_with_target_config(&self, target_config: &Self) -> miette::Result<Self> {
//         if target_config.debug_dir.is_some() {
//             miette::bail!("`debug_dir` cannot have a target specific value");
//         }
//         Ok(self.clone())
//     }
// }

// pub type PixiGenerateRecipeFn = unsafe extern "C" fn(
//     ctx: *mut c_void,
//     project_model_json: *const c_char,
//     config_json: *const c_char,
//     manifest_path: *const c_char,
//     host_platform: *const c_char,
//     python_editable: bool,
//     variants_json: *const c_char,
//     out_generated_recipe: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus;

// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct PixiOptionalGenerateRecipeFn {
//     pub is_some: bool,
//     pub callback: PixiGenerateRecipeFn,
// }

// impl PixiOptionalGenerateRecipeFn {
//     fn into_option(self) -> Option<PixiGenerateRecipeFn> {
//         if self.is_some {
//             Some(self.callback)
//         } else {
//             None
//         }
//     }
// }

// pub type PixiExtractInputGlobsFn = unsafe extern "C" fn(
//     ctx: *mut c_void,
//     config_json: *const c_char,
//     workdir: *const c_char,
//     editable: bool,
//     out_globs: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus;

// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct PixiOptionalExtractInputGlobsFn {
//     pub is_some: bool,
//     pub callback: PixiExtractInputGlobsFn,
// }

// impl PixiOptionalExtractInputGlobsFn {
//     fn into_option(self) -> Option<PixiExtractInputGlobsFn> {
//         if self.is_some {
//             Some(self.callback)
//         } else {
//             None
//         }
//     }
// }

// pub type PixiDefaultVariantsFn = unsafe extern "C" fn(
//     ctx: *mut c_void,
//     host_platform: *const c_char,
//     out_variants: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus;

// #[repr(C)]
// #[derive(Clone, Copy)]
// pub struct PixiOptionalDefaultVariantsFn {
//     pub is_some: bool,
//     pub callback: PixiDefaultVariantsFn,
// }

// impl PixiOptionalDefaultVariantsFn {
//     fn into_option(self) -> Option<PixiDefaultVariantsFn> {
//         if self.is_some {
//             Some(self.callback)
//         } else {
//             None
//         }
//     }
// }

// #[repr(C)]
// pub struct PixiGenerateRecipe {
//     pub ctx: *mut c_void,
//     pub generate_recipe: PixiOptionalGenerateRecipeFn,
//     pub extract_input_globs_from_build: PixiOptionalExtractInputGlobsFn,
//     pub default_variants: PixiOptionalDefaultVariantsFn,
// }

// #[derive(Clone)]
// struct CGenerateRecipeAdapter {
//     ctx: *mut c_void,
//     generate_recipe: PixiGenerateRecipeFn,
//     extract_input_globs_from_build: Option<PixiExtractInputGlobsFn>,
//     default_variants: Option<PixiDefaultVariantsFn>,
// }

// unsafe impl Send for CGenerateRecipeAdapter {}
// unsafe impl Sync for CGenerateRecipeAdapter {}

// impl CGenerateRecipeAdapter {
//     unsafe fn try_from_raw(raw: *const PixiGenerateRecipe) -> miette::Result<Self> {
//         if raw.is_null() {
//             miette::bail!("generator pointer was null");
//         }
//         let generator = unsafe { &*raw };
//         let generate_recipe = generator
//             .generate_recipe
//             .into_option()
//             .ok_or_else(|| miette::miette!("generate_recipe callback missing"))?;
//         Ok(Self {
//             ctx: generator.ctx,
//             generate_recipe,
//             extract_input_globs_from_build: generator.extract_input_globs_from_build.into_option(),
//             default_variants: generator.default_variants.into_option(),
//         })
//     }
// }

// #[derive(Serialize, Deserialize)]
// struct GeneratedRecipeJson {
//     recipe: IntermediateRecipe,
//     #[serde(default)]
//     metadata_input_globs: BTreeSet<String>,
//     #[serde(default)]
//     build_input_globs: BTreeSet<String>,
// }

// impl From<GeneratedRecipe> for GeneratedRecipeJson {
//     fn from(value: GeneratedRecipe) -> Self {
//         Self {
//             recipe: value.recipe,
//             metadata_input_globs: value.metadata_input_globs,
//             build_input_globs: value.build_input_globs,
//         }
//     }
// }

// impl From<GeneratedRecipeJson> for GeneratedRecipe {
//     fn from(value: GeneratedRecipeJson) -> Self {
//         Self {
//             recipe: value.recipe,
//             metadata_input_globs: value.metadata_input_globs,
//             build_input_globs: value.build_input_globs,
//         }
//     }
// }

// impl GenerateRecipe for CGenerateRecipeAdapter {
//     type Config = CBackendConfig;

//     fn generate_recipe(
//         &self,
//         model: &ProjectModelV1,
//         config: &Self::Config,
//         manifest_path: PathBuf,
//         host_platform: Platform,
//         python_params: Option<PythonParams>,
//         variants: &HashSet<NormalizedKey>,
//     ) -> miette::Result<GeneratedRecipe> {
//         let project_model_json = serde_json::to_string(model).into_diagnostic()?;
//         let config_json = serde_json::to_string(&config.raw).into_diagnostic()?;
//         let manifest_path_str = manifest_path.display().to_string();
//         let host_platform_str = host_platform.to_string();
//         let editable = python_params.map(|p| p.editable).unwrap_or(false);
//         let variants_vec: Vec<String> = variants.iter().map(|v| v.0.clone()).collect();
//         let variants_json = serde_json::to_string(&variants_vec).into_diagnostic()?;

//         let project_model_c = CString::new(project_model_json).unwrap();
//         let config_c = CString::new(config_json).unwrap();
//         let manifest_c = CString::new(manifest_path_str).unwrap();
//         let host_platform_c = CString::new(host_platform_str).unwrap();
//         let variants_c = CString::new(variants_json).unwrap();

//         let mut out_generated_recipe = PixiOwnedString::default();
//         let mut out_error = PixiOwnedString::default();

//         let status = unsafe {
//             (self.generate_recipe)(
//                 self.ctx,
//                 project_model_c.as_ptr(),
//                 config_c.as_ptr(),
//                 manifest_c.as_ptr(),
//                 host_platform_c.as_ptr(),
//                 editable,
//                 variants_c.as_ptr(),
//                 &mut out_generated_recipe,
//                 &mut out_error,
//             )
//         };

//         match status {
//             PixiStatus::OK => {
//                 if !out_error.data.is_null() {
//                     unsafe {
//                         let _ = owned_string_to_string(out_error);
//                     }
//                 }
//                 let json = unsafe { owned_string_to_string(out_generated_recipe)? };
//                 let parsed: GeneratedRecipeJson = serde_json::from_str(&json)
//                     .into_diagnostic()
//                     .wrap_err("generate_recipe callback returned invalid JSON")?;
//                 Ok(parsed.into())
//             }
//             PixiStatus::Error => {
//                 let message = unsafe { owned_string_to_string(out_error) }
//                     .unwrap_or_else(|_| "generate_recipe callback failed".to_string());
//                 miette::bail!(message)
//             }
//         }
//     }

//     fn extract_input_globs_from_build(
//         &self,
//         config: &Self::Config,
//         workdir: impl AsRef<Path>,
//         editable: bool,
//     ) -> miette::Result<BTreeSet<String>> {
//         let Some(callback) = self.extract_input_globs_from_build else {
//             return Ok(BTreeSet::new());
//         };

//         let config_json = serde_json::to_string(&config.raw).into_diagnostic()?;
//         let config_c = CString::new(config_json).unwrap();
//         let workdir_c = CString::new(workdir.as_ref().display().to_string()).unwrap();

//         let mut out_globs = PixiOwnedString::default();
//         let mut out_error = PixiOwnedString::default();

//         let status = unsafe {
//             callback(
//                 self.ctx,
//                 config_c.as_ptr(),
//                 workdir_c.as_ptr(),
//                 editable,
//                 &mut out_globs,
//                 &mut out_error,
//             )
//         };

//         match status {
//             PixiStatus::OK => {
//                 if !out_error.data.is_null() {
//                     unsafe {
//                         let _ = owned_string_to_string(out_error);
//                     }
//                 }
//                 if out_globs.data.is_null() {
//                     return Ok(BTreeSet::new());
//                 }
//                 let data = unsafe { owned_string_to_string(out_globs)? };
//                 let list: Vec<String> = serde_json::from_str(&data)
//                     .into_diagnostic()
//                     .wrap_err("extract_input_globs_from_build returned invalid JSON")?;
//                 Ok(list.into_iter().collect())
//             }
//             PixiStatus::Error => {
//                 let message = unsafe { owned_string_to_string(out_error) }.unwrap_or_else(|_| {
//                     "extract_input_globs_from_build callback failed".to_string()
//                 });
//                 miette::bail!(message)
//             }
//         }
//     }

//     fn default_variants(
//         &self,
//         host_platform: Platform,
//     ) -> miette::Result<BTreeMap<NormalizedKey, Vec<Variable>>> {
//         let Some(callback) = self.default_variants else {
//             return Ok(BTreeMap::new());
//         };

//         let host_platform_c = CString::new(host_platform.to_string()).unwrap();
//         let mut out_variants = PixiOwnedString::default();
//         let mut out_error = PixiOwnedString::default();

//         let status = unsafe {
//             callback(
//                 self.ctx,
//                 host_platform_c.as_ptr(),
//                 &mut out_variants,
//                 &mut out_error,
//             )
//         };

//         match status {
//             PixiStatus::OK => {
//                 if !out_error.data.is_null() {
//                     unsafe {
//                         let _ = owned_string_to_string(out_error);
//                     }
//                 }
//                 if out_variants.data.is_null() {
//                     return Ok(BTreeMap::new());
//                 }
//                 let data = unsafe { owned_string_to_string(out_variants)? };
//                 let map: BTreeMap<String, Vec<String>> = serde_json::from_str(&data)
//                     .into_diagnostic()
//                     .wrap_err("default_variants callback returned invalid JSON")?;
//                 let mut result = BTreeMap::new();
//                 for (key, values) in map {
//                     result.insert(
//                         NormalizedKey::from(key),
//                         values.into_iter().map(Variable::from).collect(),
//                     );
//                 }
//                 Ok(result)
//             }
//             PixiStatus::Error => {
//                 let message = unsafe { owned_string_to_string(out_error) }
//                     .unwrap_or_else(|_| "default_variants callback failed".to_string());
//                 miette::bail!(message)
//             }
//         }
//     }
// }

// #[repr(C)]
// pub struct PixiGeneratedRecipeOpaque {
//     _private: [u8; 0],
// }

// type PixiGeneratedRecipePtr = *mut PixiGeneratedRecipeOpaque;

// unsafe fn generated_recipe_from_ptr(ptr: PixiGeneratedRecipePtr) -> *mut GeneratedRecipe {
//     ptr as *mut GeneratedRecipe
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_free(recipe: PixiGeneratedRecipePtr) {
//     if recipe.is_null() {
//         return;
//     }
//     unsafe {
//         drop(Box::from_raw(generated_recipe_from_ptr(recipe)));
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_to_json(
//     recipe: PixiGeneratedRecipePtr,
//     out_json: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if recipe.is_null() {
//         set_error(out_error, "recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);

//     let recipe = unsafe { &*generated_recipe_from_ptr(recipe) };
//     let json = serde_json::to_string(&GeneratedRecipeJson::from(recipe.clone()))
//         .into_diagnostic()
//         .map_err(|e| e.to_string());

//     match json {
//         Ok(json) => {
//             if !out_json.is_null() {
//                 unsafe {
//                     *out_json = PixiOwnedString::from_string(json);
//                 }
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err);
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_recipe_to_json(
//     recipe: PixiGeneratedRecipePtr,
//     out_json: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if recipe.is_null() {
//         set_error(out_error, "recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let recipe = unsafe { &*generated_recipe_from_ptr(recipe) };
//     match serde_json::to_string(&recipe.recipe) {
//         Ok(json) => {
//             if !out_json.is_null() {
//                 unsafe { *out_json = PixiOwnedString::from_string(json) };
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }

// fn serialize_set(
//     out_json: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
//     set: &BTreeSet<String>,
// ) -> PixiStatus {
//     let values: Vec<String> = set.iter().cloned().collect();
//     match serde_json::to_string(&values) {
//         Ok(json) => {
//             if !out_json.is_null() {
//                 unsafe { *out_json = PixiOwnedString::from_string(json) };
//             }
//             clear_error(out_error);
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_metadata_input_globs(
//     recipe: PixiGeneratedRecipePtr,
//     out_json: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if recipe.is_null() {
//         set_error(out_error, "recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     let recipe = unsafe { &*generated_recipe_from_ptr(recipe) };
//     serialize_set(out_json, out_error, &recipe.metadata_input_globs)
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_build_input_globs(
//     recipe: PixiGeneratedRecipePtr,
//     out_json: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if recipe.is_null() {
//         set_error(out_error, "recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     let recipe = unsafe { &*generated_recipe_from_ptr(recipe) };
//     serialize_set(out_json, out_error, &recipe.build_input_globs)
// }

// fn allocate_recipe(recipe: GeneratedRecipe) -> PixiGeneratedRecipePtr {
//     Box::into_raw(Box::new(recipe)) as PixiGeneratedRecipePtr
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_from_model(
//     model: PixiProjectModelV1Ptr,
//     metadata_provider: *const PixiMetadataProvider,
//     out_recipe: *mut PixiGeneratedRecipePtr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if out_recipe.is_null() {
//         set_error(out_error, "out_recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     if model.is_null() {
//         set_error(out_error, "model pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);

//     let model = unsafe { (*project_model_from_ptr(model)).clone() };

//     let result = if metadata_provider.is_null() {
//         let mut provider = DefaultMetadataProvider;
//         GeneratedRecipe::from_model(model, &mut provider)
//             .map_err(|e| miette::miette!(e.to_string()))
//     } else {
//         let mut provider = CMetadataProvider::new(unsafe { &*metadata_provider });
//         GeneratedRecipe::from_model(model, &mut provider)
//             .map_err(|e| miette::miette!(e.to_string()))
//     };

//     match result {
//         Ok(recipe) => {
//             unsafe { *out_recipe = allocate_recipe(recipe) };
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_from_model_with_provider(
//     model: PixiProjectModelV1Ptr,
//     metadata_provider: *const PixiMetadataProvider,
//     out_recipe: *mut PixiGeneratedRecipePtr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     unsafe { pixi_generated_recipe_from_model(model, metadata_provider, out_recipe, out_error) }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_platform_current(
//     out_platform: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     clear_error(out_error);
//     if out_platform.is_null() {
//         set_error(out_error, "out_platform pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     unsafe {
//         *out_platform = PixiOwnedString::from_string(Platform::current().to_string());
//     }
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_platform_parse(
//     value: *const c_char,
//     out_platform: *mut PixiOwnedString,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if value.is_null() {
//         set_error(out_error, "platform string pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     if out_platform.is_null() {
//         set_error(out_error, "out_platform pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let value = unsafe { CStr::from_ptr(value) }
//         .to_string_lossy()
//         .into_owned();
//     match value.parse::<Platform>() {
//         Ok(platform) => {
//             unsafe { *out_platform = PixiOwnedString::from_string(platform.to_string()) };
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_build_backend_main(
//     generator: *const PixiGenerateRecipe,
//     args: *const *const c_char,
//     args_len: usize,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     clear_error(out_error);
//     let adapter = match unsafe { CGenerateRecipeAdapter::try_from_raw(generator) } {
//         Ok(adapter) => adapter,
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             return PixiStatus::Error;
//         }
//     };

//     let args_vec = if args.is_null() {
//         Vec::new()
//     } else {
//         (0..args_len)
//             .map(|i| {
//                 let ptr = unsafe { *args.add(i) };
//                 if ptr.is_null() {
//                     String::new()
//                 } else {
//                     unsafe { CStr::from_ptr(ptr) }
//                         .to_string_lossy()
//                         .into_owned()
//                 }
//             })
//             .collect::<Vec<_>>()
//     };

//     let runtime = match Runtime::new() {
//         Ok(rt) => rt,
//         Err(err) => {
//             set_error(out_error, format!("failed to create tokio runtime: {err}"));
//             return PixiStatus::Error;
//         }
//     };

//     let generator = Arc::new(adapter);
//     let result = runtime.block_on(async move {
//         cli_main(
//             |log| IntermediateBackendInstantiator::new(log, generator.clone()),
//             args_vec,
//         )
//         .await
//     });

//     match result {
//         Ok(_) => PixiStatus::OK,
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_new_empty(
//     out_recipe: *mut PixiGeneratedRecipePtr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if out_recipe.is_null() {
//         set_error(out_error, "out_recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     unsafe {
//         *out_recipe = allocate_recipe(GeneratedRecipe::default());
//     }
//     PixiStatus::OK
// }

// #[unsafe(no_mangle)]
// pub unsafe extern "C" fn pixi_generated_recipe_from_json(
//     json: *const c_char,
//     out_recipe: *mut PixiGeneratedRecipePtr,
//     out_error: *mut PixiOwnedString,
// ) -> PixiStatus {
//     if json.is_null() {
//         set_error(out_error, "json pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     if out_recipe.is_null() {
//         set_error(out_error, "out_recipe pointer was null".to_string());
//         return PixiStatus::Error;
//     }
//     clear_error(out_error);
//     let json = unsafe { CStr::from_ptr(json) }
//         .to_string_lossy()
//         .into_owned();
//     match serde_json::from_str::<GeneratedRecipeJson>(&json) {
//         Ok(parsed) => {
//             unsafe {
//                 *out_recipe = allocate_recipe(parsed.into());
//             }
//             PixiStatus::OK
//         }
//         Err(err) => {
//             set_error(out_error, err.to_string());
//             PixiStatus::Error
//         }
//     }
// }
