use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Only build bindings if the with-sdk feature is enabled
    #[cfg(feature = "with-sdk")]
    {
        let sdk_path = get_sdk_path();
        
        if sdk_path.exists() {
            link_sdk(&sdk_path);
            generate_bindings(&sdk_path);
        } else {
            panic!("BlackmagicRAW SDK not found at: {}", sdk_path.display());
        }
    }
    
    #[cfg(not(feature = "with-sdk"))]
    {
        println!("cargo:warning=BlackmagicRAW SDK support disabled (enable with --features with-sdk)");
    }
}

fn get_sdk_path() -> PathBuf {
    // Try environment variable first
    if let Ok(path) = env::var("BRAW_SDK_PATH") {
        return PathBuf::from(path);
    }
    
    // Default paths for different platforms
    let workspace_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = PathBuf::from(workspace_root).parent().unwrap().parent().unwrap().to_path_buf();
    
    // Try workspace-relative path first (.data/sdk/)
    let data_sdk_path = workspace_root.join(".data/sdk");
    if data_sdk_path.exists() {
        return data_sdk_path;
    }
    
    // Platform-specific defaults
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Program Files\Blackmagic Design\BlackmagicRAW\SDK")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Applications/Blackmagic RAW/Blackmagic RAW SDK")
    }
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/usr/local/include/BlackmagicRAW")
    }
}

#[cfg(feature = "with-sdk")]
fn link_sdk(sdk_path: &PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let framework_path = sdk_path.join("Mac/Libraries");
        println!("cargo:rustc-link-search=framework={}", framework_path.display());
        println!("cargo:rustc-link-lib=framework=BlackmagicRawAPI");
    }
    
    #[cfg(target_os = "windows")]
    {
        let lib_path = sdk_path.join("Win/Libraries");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=dylib=BlackmagicRawAPI");
    }
    
    #[cfg(target_os = "linux")]
    {
        let lib_path = sdk_path.join("Linux/Libraries");
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-lib=dylib=BlackmagicRawAPI");
    }
}

#[cfg(feature = "with-sdk")]
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
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));
    
    // Add platform-specific include paths
    #[cfg(target_os = "macos")]
    {
        let sdk_path = std::process::Command::new("xcrun")
            .args(&["--show-sdk-path"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk".to_string());
        
        builder = builder
            .clang_arg(format!("-isysroot{}", sdk_path))
            .clang_arg("-F/System/Library/Frameworks");
    }
    
    let bindings = match builder.generate() {
        Ok(bindings) => bindings,
        Err(e) => {
            eprintln!("Warning: Unable to generate BlackmagicRAW bindings: {}", e);
            eprintln!("Creating placeholder bindings file...");
            
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