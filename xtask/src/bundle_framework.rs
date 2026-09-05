use crate::bundle_common::{
    FrameworkInfoPlist, TARGET_ARM64, TARGET_X64, deploy_bundle_to_addon, get_target_dir,
    get_target_dir_for_target, run_cargo_for_macos_targets, run_lipo, sign_macos_code,
};
use crate::platform::{
    MACOS_CEF_APP_PATH, MACOS_EXTENSION_FRAMEWORK, MACOS_EXTENSION_INSTALL_NAME,
    MACOS_UNIVERSAL_TARGET,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn set_dylib_install_name(
    dylib: &Path,
    install_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("install_name_tool")
        .args(["-id", install_name])
        .arg(dylib)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        return Ok(());
    }

    Err(format!(
        "install_name_tool failed for {} with status: {}",
        dylib.display(),
        status
    )
    .into())
}

fn create_framework(
    fmwk_path: &Path,
    lib_name: &str,
    bin: &Path,
    cef_app: &Path,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let fmwk_path = fmwk_path.join(MACOS_EXTENSION_FRAMEWORK);
    if fmwk_path.exists() {
        fs::remove_dir_all(&fmwk_path)?;
    }

    let resources_path = fmwk_path.join("Resources");
    fs::create_dir_all(&resources_path)?;
    let info_plist = FrameworkInfoPlist::new(lib_name);
    plist::to_file_xml(resources_path.join("Info.plist"), &info_plist)?;
    let cef_app_path = fmwk_path.join(MACOS_CEF_APP_PATH);
    fs::create_dir_all(cef_app_path.parent().ok_or("Invalid CEF app path")?)?;
    fs::rename(cef_app, &cef_app_path)?;
    let library_path = fmwk_path.join(lib_name);
    fs::copy(bin, &library_path)?;
    set_dylib_install_name(&library_path, MACOS_EXTENSION_INSTALL_NAME)?;
    sign_macos_code(&fmwk_path)?;
    Ok(fmwk_path)
}

pub fn run(release: bool, target_dir: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    let cef_app = crate::bundle_app::build(release, target_dir)?;
    run_cargo_for_macos_targets(&["build", "--lib", "--package", "gdcef"], release)?;

    let target_dir_arm64 = get_target_dir_for_target(release, TARGET_ARM64, target_dir);
    let target_dir_x64 = get_target_dir_for_target(release, TARGET_X64, target_dir);
    let output_dir = get_target_dir(release, target_dir);

    let dylib_arm64 = target_dir_arm64.join("libgdcef.dylib");
    let dylib_x64 = target_dir_x64.join("libgdcef.dylib");
    let universal_dylib = output_dir.join("libgdcef_universal.dylib");

    run_lipo(&dylib_arm64, &dylib_x64, &universal_dylib)?;

    let fmwk_path = create_framework(&output_dir, "libgdcef.dylib", &universal_dylib, &cef_app)?;
    println!("Created: {}", fmwk_path.display());
    fs::remove_file(&universal_dylib)?;
    deploy_bundle_to_addon(&fmwk_path, MACOS_UNIVERSAL_TARGET)?;

    Ok(())
}
