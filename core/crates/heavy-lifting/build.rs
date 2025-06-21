fn main() {
    #[cfg(target_os = "macos")]
    {
        let sdk_path = "/Applications/Blackmagic RAW/Blackmagic RAW SDK/Mac/Libraries";
        
        // Compile C++ wrapper
        cc::Build::new()
            .cpp(true)
            .file("wrapper.cpp")
            .include(".")
            .include(sdk_path)
            .flag("-std=c++17")
            .flag("-ObjC++")
            .compile("braw_wrapper");
        
        println!("cargo:rustc-link-search=framework={}", sdk_path);
        println!("cargo:rustc-link-lib=framework=BlackmagicRawAPI");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=MetalKit");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
        println!("cargo:rustc-link-lib=c++");
        
        println!("cargo:rerun-if-changed=wrapper.cpp");
        println!("cargo:rerun-if-changed=wrapper.h");
        println!("cargo:rerun-if-changed=src/media_processor/helpers/braw_decoder.rs");
        println!("cargo:rerun-if-changed=src/media_processor/helpers/braw_thumbnailer.rs");
        println!("cargo:rerun-if-changed=src/media_processor/helpers/braw_media_data.rs");
    }
}