//! Types:
//! - [ ] Config
//! - [ ] Generated recipe
//! - [ ] Metadata provider
//! - [x] Platform
//! - [ ] Project model
//! - [ ] Python params
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
use miette::Diagnostic;
use pixi_build_backend::generated_recipe::MetadataProvider as BackendMetadataProvider;
use rattler_conda_types::Version;
use safer_ffi::prelude::*;
use std::mem;
use thiserror::Error;

// ========Config========
// #[derive_ReprC]
// #[repr(opaque)]
// pub struct Config {

// }

// ===Generated Recipe===

// ===Metadata Provider==

#[derive_ReprC]
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum MetadataProviderStatus {
    Ok,
    Err,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct MetadataProviderResult {
    error_message: char_p::Box,
    status: MetadataProviderStatus,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct MetadataOptionalString {
    has_value: bool,
    value: char_p::Box,
}

impl Default for MetadataOptionalString {
    fn default() -> Self {
        Self {
            has_value: false,
            value: char_p::new(""),
        }
    }
}

impl MetadataOptionalString {
    fn to_option(self) -> Option<String> {
        if self.has_value {
            Some(self.value.into_string())
        } else {
            None
        }
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Clone)]
pub struct MetadataOptionalVersion {
    has_value: bool,
    value: char_p::Box,
}

impl Default for MetadataOptionalVersion {
    fn default() -> Self {
        Self {
            has_value: false,
            value: char_p::new(""),
        }
    }
}

impl MetadataOptionalVersion {
    fn to_option(self) -> Option<String> {
        if self.has_value {
            Some(self.value.into_string())
        } else {
            None
        }
    }
}

#[derive_ReprC(dyn)]
pub trait CMetadataProvider {
    fn name(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn version(&mut self, output: &mut MetadataOptionalVersion) -> MetadataProviderResult;
    fn homepage(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn license(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn license_file(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn summary(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn description(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn documentation(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
    fn repository(&mut self, output: &mut MetadataOptionalString) -> MetadataProviderResult;
}

#[derive_ReprC]
#[repr(transparent)]
pub struct MetadataProvider {
    actual_provider: VirtualPtr<dyn CMetadataProvider>,
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
    fn handle_result<O: MetadataValue>(
        field: &'static str,
        result: MetadataProviderResult,
        output: &mut O,
    ) -> Result<Option<String>, ForeignMetadataProviderError> {
        match result.status {
            MetadataProviderStatus::Ok => {
                let value = mem::take(output).into_option();
                Ok(value)
            }
            MetadataProviderStatus::Err => Err(ForeignMetadataProviderError::new(
                field,
                result.error_message.into_string(),
            )),
        }
    }
}

impl BackendMetadataProvider for MetadataProvider {
    type Error = ForeignMetadataProviderError;

    fn name(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.name(&mut output);
        Self::handle_result("name", result, &mut output)
    }

    fn version(&mut self) -> Result<Option<Version>, Self::Error> {
        let mut output = MetadataOptionalVersion::default();
        let result = self.actual_provider.version(&mut output);
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
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.homepage(&mut output);
        Self::handle_result("homepage", result, &mut output)
    }

    fn license(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.license(&mut output);
        Self::handle_result("license", result, &mut output)
    }

    fn license_file(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.license_file(&mut output);
        Self::handle_result("license_file", result, &mut output)
    }

    fn summary(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.summary(&mut output);
        Self::handle_result("summary", result, &mut output)
    }

    fn description(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.description(&mut output);
        Self::handle_result("description", result, &mut output)
    }

    fn documentation(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.documentation(&mut output);
        Self::handle_result("documentation", result, &mut output)
    }

    fn repository(&mut self) -> Result<Option<String>, Self::Error> {
        let mut output = MetadataOptionalString::default();
        let result = self.actual_provider.repository(&mut output);
        Self::handle_result("repository", result, &mut output)
    }
}

trait MetadataValue: Default {
    fn into_option(self) -> Option<String>;
}

impl MetadataValue for MetadataOptionalString {
    fn into_option(self) -> Option<String> {
        MetadataOptionalString::to_option(self)
    }
}

impl MetadataValue for MetadataOptionalVersion {
    fn into_option(self) -> Option<String> {
        MetadataOptionalVersion::to_option(self)
    }
}

#[ffi_export]
pub fn ppb_metadata_provider_new(
    actual_provider: VirtualPtr<dyn CMetadataProvider>,
) -> MetadataProvider {
    MetadataProvider { actual_provider }
}

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

// The following function is only necessary for the header generation.
#[cfg(feature = "headers")] // c.f. the `Cargo.toml` section
pub fn generate_headers() -> ::std::io::Result<()> {
    safer_ffi::headers::builder()
        .to_file("rust_points.h")?
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
