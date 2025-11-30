fn main() {
    // On Windows, we need to explicitly link against system libraries
    // that OpenSSL depends on for cryptographic operations
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=ws2_32");
    }
}
