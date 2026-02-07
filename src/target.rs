use std::{collections::HashSet, env::var, error::Error, fmt::Display};

/// Contains the supported C3 operating system targets corresponding to Android.
pub const C3_ANDROID_TARGETS: &[&str] = &["android"];

/// Contains the supported C3 operating system targets corresponding to WebAssembly.
pub const C3_WASM_TARGETS: &[&str] = &["wasm32", "wasm64"];

/// Contains the supported C3 operating system targets corresponding to Windows.
pub const C3_WINDOWS_TARGETS: &[&str] = &["windows", "mingw"];

/// Contains the Rust ABIs corresponding to Android.
///
/// While 'androideabi' is provided by Rustup, C3 cannot compile for it, given its target architectures are not supported.
pub const RUST_ANDROID_ABIS: &[&str] = &["android"];

/// Contains the Rust ABIs corresponding to MinGW.
pub const RUST_MINGW_ABIS: &[&str] = &["gnu", "gnullvm"];

/// Contains the Rust architectures corresponding to WebAssembly.
///
/// While Rust does not support wasm64 yet, it is set here already for when it does.
pub const RUST_WASM_ARCHITECTURES: &[&str] = &["wasm32", "wasm64"];

/// Architectures whose support status is unknown, but are converted to a more generic name for compatiblity just in case.
///
/// If you find any of these to not be compatible with C3, you can either send us a PR or warn us.
pub const AMBIGUOUS_ARCHITECTURE_SUPPORT: &[(&str, &str)] = &[
    ("arm64ec", "aarch64"),
    ("riscv32i", "riscv32"),
    ("riscv32im", "riscv32"),
    ("riscv32imc", "riscv32"),
    ("riscv32imac", "riscv32"),
    ("riscv32imafc", "riscv32"),
];

/// Architectures known to be incompatible with C3 outright.
///
/// If in the future, any of these becomes compatible, they may be removed from this list.
pub const UNSUPPORTED_ARCHITECTURES: &[&str] = &[
    "arm",
    "armv5te",
    "armv7",
    "armv7a",
    "armv7r",
    "armebv7r",
    "thumbv6m",
    "thumbv7m",
    "thumbv7em",
    "thumbv7neon",
    "thumbv8m.base",
    "thumbv8m.main",
    "loongarch64",
    "nvptx64",
    "powerpc",
    "powerpc64",
    "powerpc64le",
    "riscv64gc",
    "riscv64imac",
    "s390x",
    "sparc64",
    "sparcv9",
];

pub struct C3Target {
    os: String,
    architecture: String,
}

impl C3Target {
    /// Converts a given Rust target into its C3 target equivalent.
    ///
    /// Example:
    /// ```rs
    /// assert!(c3ne::C3Target::convert("x86_64-unknown-linux-gnu").to_string(), String::from("linux-x64"));
    /// ```
    pub fn convert(rust_target: &str) -> Self {
        let target_split: Vec<&str> = rust_target.split("-").collect();

        let where_os = match (&target_split).len() {
            4 => 2,
            _ => {
                if RUST_WASM_ARCHITECTURES.contains(&target_split[0]) {
                    0
                } else {
                    1
                }
            }
        };

        let mut architecture = target_split[0];
        let mut os = target_split[where_os];
        let abi = target_split[where_os + 1];

        Self::panic_on_unsupported_architecture(architecture);
        architecture = Self::warn_on_ambiguous_architecture_support(architecture);

        if os.eq_ignore_ascii_case("windows") && RUST_MINGW_ABIS.contains(&abi) {
            os = "mingw";
        }
        if RUST_ANDROID_ABIS.contains(&abi) {
            os = "android";
        }

        // The Android AMD64 target is different from every other for some reason.
        if architecture.eq_ignore_ascii_case("x86_64") && !os.ends_with("android") {
            architecture = "x64";
        }
        if architecture.eq_ignore_ascii_case("arm64ec") {
            architecture = "aarch64";
        }

        Self {
            os: os.to_string(),
            architecture: architecture.to_string(),
        }
    }

    fn panic_on_unsupported_architecture(architecture: &str) {
        if UNSUPPORTED_ARCHITECTURES.contains(&architecture) {
            panic!("Architecture '{}' is unsupported by C3.", architecture);
        }
    }

    fn warn_on_ambiguous_architecture_support(architecture: &str) -> &str {
        let true_names: HashSet<&str> =
            AMBIGUOUS_ARCHITECTURE_SUPPORT.iter().map(|t| t.0).collect();

        if true_names.contains(&architecture) {
            eprintln!(
                "Architecture '{}' support status is unknown, it is enabled until proven to not be supported.",
                architecture
            );
            eprintln!(
                "Please report to the c3ne project if you find it unsupported. Thank you for your cooperation!"
            );

            return AMBIGUOUS_ARCHITECTURE_SUPPORT
                .iter()
                .filter(|t| t.1.eq_ignore_ascii_case(architecture))
                .map(|t| t.1)
                .collect::<Vec<&str>>()[0];
        }

        architecture
    }

    /// Converts the current compilation target into a C3 target.
    pub fn convert_current() -> Result<Self, Box<dyn Error>> {
        Ok(Self::convert(&var("TARGET")?))
    }

    /// Returns the C3-ified target operating system.
    pub fn os(&self) -> String {
        self.os.clone()
    }

    /// Returns the C3-ified target architecture.
    pub fn architecture(&self) -> String {
        self.architecture.clone()
    }

    /// Returns whether the target operating system is Android.
    pub fn is_android(&self) -> bool {
        C3_ANDROID_TARGETS.contains(&self.os.as_str())
    }

    /// Returns whether the target operating system is WebAssembly.
    pub fn is_wasm(&self) -> bool {
        C3_WASM_TARGETS.contains(&self.os.as_str())
    }

    /// Returns whether the target operating system is Windows.
    pub fn is_windows(&self) -> bool {
        C3_WINDOWS_TARGETS.contains(&self.os.as_str())
    }
}

impl Display for C3Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            &self.os,
            if !self.is_wasm() {
                format!("-{}", self.architecture)
            } else {
                String::new()
            }
        )
    }
}
