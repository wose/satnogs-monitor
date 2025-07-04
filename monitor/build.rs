fn main() {
    // Use cmake to build libgpredict
    let dst = cmake::Config::new("libgpredict")
        .define("BUILD_SHARED_LIBS", "OFF") // Make sure we build static
        .define("CMAKE_POSITION_INDEPENDENT_CODE", "ON")
        .build();

    // Path to the built static library
    let lib_dir = dst.join("lib");

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=gpredict");

    // Ensure glib-2.0 is linked
    pkg_config::Config::new()
        .cargo_metadata(true)
        .probe("glib-2.0")
        .expect("Failed to find glib-2.0 via pkg-config");

    // Rebuild if anything in gpredict changes
    println!("cargo:rerun-if-changed=libgpredict");

    // Tell Cargo to find and link against glib
    pkg_config::Config::new()
        .atleast_version("2.0")
        .probe("glib-2.0")
        .expect("Failed to find glib-2.0 via pkg-config");
}
