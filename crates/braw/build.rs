use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Only build bindings if the native-ffi feature is enabled
    #[cfg(feature = "native-ffi")]
    {
        let sdk_path = get_sdk_path();

        if sdk_path.exists() {
            link_sdk(&sdk_path);
            generate_bindings(&sdk_path);
        } else {
            panic!("BlackmagicRAW SDK not found at: {}", sdk_path.display());
        }
    }
    // No warning for with-sdk only (safe stubs mode)
}

fn get_sdk_path() -> PathBuf {
    // Try environment variable first
    if let Ok(path) = env::var("BRAW_SDK_PATH") {
        return PathBuf::from(path);
    }

    // Platform-specific defaults (prefer system installation)
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Program Files\Blackmagic Design\BlackmagicRAW\SDK")
    }
    #[cfg(target_os = "macos")]
    {
        let system_path = PathBuf::from("/Applications/Blackmagic RAW/Blackmagic RAW SDK");
        if system_path.exists() {
            return system_path;
        }

        // Fallback to workspace copy if system installation not found
        let workspace_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let workspace_root = PathBuf::from(workspace_root).parent().unwrap().parent().unwrap().to_path_buf();
        workspace_root.join(".data/sdk")
    }
    #[cfg(target_os = "linux")]
    {
        let system_path = PathBuf::from("/usr/local/include/BlackmagicRAW");
        if system_path.exists() {
            return system_path;
        }

        // Fallback to workspace copy if system installation not found
        let workspace_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let workspace_root = PathBuf::from(workspace_root).parent().unwrap().parent().unwrap().to_path_buf();
        workspace_root.join(".data/sdk")
    }
}

#[cfg(feature = "native-ffi")]
fn link_sdk(sdk_path: &PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let framework_path = sdk_path.join("Mac/Libraries");
        println!("cargo:rustc-link-search=framework={}", framework_path.display());

        // Link to the BlackmagicRawAPI framework
        println!("cargo:rustc-link-lib=framework=BlackmagicRawAPI");

        // Set runtime library path so the framework can be found at runtime
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", framework_path.display());

        // Also add the standard framework search paths
        println!("cargo:rustc-link-arg=-Wl,-rpath,/System/Library/Frameworks");
        println!("cargo:rustc-link-arg=-Wl,-rpath,/Library/Frameworks");

        // Tell the linker where to find the framework at runtime
        println!("cargo:rustc-env=DYLD_FRAMEWORK_PATH={}", framework_path.display());

        println!("cargo:warning=Linking BlackmagicRawAPI framework from: {}", framework_path.display());
    }

    #[cfg(target_os = "windows")]
    {
        let lib_path = sdk_path.join("Win/Libraries");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=dylib=BlackmagicRawAPI");

        // Set runtime library path for Windows
        println!("cargo:rustc-link-arg=/LIBPATH:{}", lib_path.display());
    }

    #[cfg(target_os = "linux")]
    {
        let lib_path = sdk_path.join("Linux/Libraries");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=dylib=BlackmagicRawAPI");

        // Set runtime library path for Linux
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_path.display());
    }
}

#[cfg(feature = "native-ffi")]
fn generate_bindings(sdk_path: &PathBuf) {
    let header_path = {
        #[cfg(target_os = "macos")]
        {
            sdk_path.join("Mac/Include/BlackmagicRawAPI.h")
        }
        #[cfg(target_os = "windows")]
        {
            sdk_path.join("Win/Include/BlackmagicRawAPI.h")
        }
        #[cfg(target_os = "linux")]
        {
            sdk_path.join("Linux/Include/BlackmagicRawAPI.h")
        }
    };

    if !header_path.exists() {
        panic!("BlackmagicRAW API header not found at: {}", header_path.display());
    }

    let mut builder = bindgen::Builder::default()
        .header(header_path.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg("-x")
        .clang_arg("c++")
        // Filter out problematic C++ templates and std library types
        .blocklist_type("std.*")
        .blocklist_type("_Tp")
        .blocklist_function("std.*")
        // Only include BlackmagicRAW types and functions
        .allowlist_type("IBlackmagic.*")
        .allowlist_type("Variant")
        .allowlist_type("blackmagic.*")
        .allowlist_function("CreateBlackmagic.*")
        .allowlist_function("Variant.*")
        .allowlist_var("blackmagic.*");

    // Add platform-specific include paths
    #[cfg(target_os = "macos")]
    {
        let sdk_include_path = sdk_path.join("Mac/Include");

        let macos_sdk_path = std::process::Command::new("xcrun")
            .args(&["--show-sdk-path"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk".to_string());

        println!("cargo:warning=Using macOS SDK path: {}", macos_sdk_path);
        println!("cargo:warning=Using BlackmagicRAW SDK include path: {}", sdk_include_path.display());

        // Use the exact same arguments that worked with clang
        builder = builder
            .clang_arg(format!("-isysroot{}", macos_sdk_path))
            .clang_arg(format!("-F{}/System/Library/Frameworks", macos_sdk_path))
            .clang_arg(format!("-I{}/System/Library/Frameworks/CoreFoundation.framework/Headers", macos_sdk_path))
            .clang_arg("-x").clang_arg("c++");
    }

    let bindings = match builder.generate() {
        Ok(bindings) => bindings,
        Err(e) => {
            println!("cargo:warning=Unable to generate BlackmagicRAW bindings: {}", e);
            println!("cargo:warning=Creating placeholder bindings file...");

            // Create a minimal placeholder for when SDK isn't available
            let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
            let bindings_path = out_path.join("bindings.rs");
            std::fs::write(&bindings_path,
                "// Placeholder bindings - SDK not available\n\
                 pub type IBlackmagicRawFactory = *mut std::ffi::c_void;\n\
                 pub type IBlackmagicRaw = *mut std::ffi::c_void;\n\
                 pub type IBlackmagicRawClip = *mut std::ffi::c_void;\n\
                 pub type IBlackmagicRawMetadataIterator = *mut std::ffi::c_void;\n\
                 pub type IBlackmagicRawJob = *mut std::ffi::c_void;\n\
                 pub struct Variant { pub vt: u32, pub iVal: i16 }\n\
                 pub const blackmagicRawVariantTypeEmpty: u32 = 0;\n\
                 pub const blackmagicRawVariantTypeString: u32 = 7;\n\
                 pub const blackmagicRawVariantTypeU32: u32 = 5;\n\
                 pub const blackmagicRawVariantTypeU16: u32 = 3;\n\
                 pub const blackmagicRawVariantTypeS32: u32 = 4;\n\
                 pub const blackmagicRawVariantTypeS16: u32 = 2;\n\
                 pub const blackmagicRawVariantTypeFloat32: u32 = 6;\n\
                 pub const blackmagicRawVariantTypeFloat64: u32 = 9;\n\
                 pub unsafe fn CreateBlackmagicRawFactoryInstanceFromPath(_: *const std::ffi::c_void) -> *mut IBlackmagicRawFactory { std::ptr::null_mut() }\n\
                 pub unsafe fn VariantClear(_: *mut Variant) {}\n")
                .expect("Failed to write placeholder bindings");
            return;
        }
    };

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}