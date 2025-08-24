fn main() {
    // Download newlib
    if !std::fs::exists("./x86_64-elf.tar.gz").unwrap() {
        std::process::Command::new("wget")
        .args([
            "https://github.com/uchan-nos/mikanos-build/releases/download/v2.0/x86_64-elf.tar.gz",
        ])
        .status()
        .unwrap();
        std::process::Command::new("tar")
            .args(["zxvf", "x86_64-elf.tar.gz"])
            .status()
            .unwrap();
    }

    let newlib_support_object = cc::Build::new()
        .flag("-Wno-unused-parameter")
        .flag("-ffreestanding")
        .flag("-mno-red-zone")
        .flag("-fshort-wchar")
        .define("__ELF__", None)
        .define("_LDBL_EQ_DBL", None)
        .define("_GNU_SOURCE", None)
        .define("_POSIX_TIMERS", None)
        .include("./x86_64-elf/include")
        .file("../mikanos/kernel/newlib_support.c")
        .compile_intermediates()
        .into_iter()
        .next()
        .unwrap();

    let usb_cxx_srcs = glob::glob("../mikanos/kernel/usb/**/*.cpp")
        .unwrap()
        .map(|res| res.unwrap())
        .collect::<Vec<_>>();

    // Build C++ library
    cc::Build::new()
        .flag("-std=c++17")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-sign-compare")
        .flag("-ffreestanding")
        .flag("-mno-red-zone")
        .flag("-nostdlibinc")
        .flag("-fno-exceptions")
        .flag("-fno-rtti")
        .flag("-fshort-wchar")
        .define("__ELF__", None)
        .define("_LDBL_EQ_DBL", None)
        .define("_GNU_SOURCE", None)
        .define("_POSIX_TIMERS", None)
        .include("./x86_64-elf/include")
        .include("./x86_64-elf/include/c++/v1")
        .include("../mikanos/kernel")
        .object(newlib_support_object)
        .file("./cpp/ffi.cpp")
        .file("./cpp/logger.cpp")
        .file("../mikanos/kernel/libcxx_support.cpp")
        .files(usb_cxx_srcs)
        .compile("usb");

    let current_dir = std::env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    println!("cargo::rerun-if-changed={current_dir}/cpp");
    println!("cargo::rerun-if-changed={current_dir}/../mikanos/kernel");
    println!("cargo::rustc-link-search=native={current_dir}/x86_64-elf/lib");
    println!("cargo::rustc-link-lib=static=c++");
    println!("cargo::rustc-link-lib=static=c++abi");
    println!("cargo::rustc-link-lib=static=c");
}
