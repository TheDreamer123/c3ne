pub use crate::target::*;
use std::{
    collections::HashMap,
    env::var,
    error::Error,
    fmt::Display,
    path::{Path, PathBuf},
    process::Command,
};

/// Builder for a C3 FFI. Compiles the given files into a static/dynamic library which can then be used from within Rust.
///
/// For alternative name, see: [Build].
pub struct C3FFI {
    compiler: String,
    linking_mode: LinkingMode,
    optimization_level: OptimizationLevel,
    debug_info: bool,
    #[cfg(target_os = "android")]
    android_ndk_path: PathBuf,
    #[cfg(target_os = "windows")]
    windows_sdk_path: PathBuf,
    files: Vec<PathBuf>,
    features: Vec<String>,
    args: Vec<String>,
    environment_variables: Vec<(String, String)>,
    linker_arguments: Vec<String>,
    compiled_lib_dirs: Vec<PathBuf>,
    compiled_libs: Vec<PathBuf>,
    c3_lib_dirs: Vec<PathBuf>,
    c3_libs: Vec<PathBuf>,
}

impl C3FFI {
    /// Initializes [C3FFI] with the default values.
    pub fn new() -> Self {
        let is_debug = cfg!(debug_assertions);

        Self {
            compiler: String::from("c3c"),
            linking_mode: LinkingMode::Static,
            optimization_level: if is_debug {
                OptimizationLevel::O0
            } else {
                OptimizationLevel::O1
            },
            debug_info: is_debug,
            #[cfg(target_os = "android")]
            android_ndk_path: var("NDK_HOME").unwrap_or_else(|_| String::new()).into(),
            #[cfg(target_os = "windows")]
            windows_sdk_path: PathBuf::new(),
            files: Vec::new(),
            features: Vec::new(),
            args: Vec::new(),
            environment_variables: Vec::new(),
            linker_arguments: Vec::new(),
            compiled_lib_dirs: Vec::new(),
            compiled_libs: Vec::new(),
            c3_lib_dirs: Vec::new(),
            c3_libs: Vec::new(),
        }
    }

    /// The path to the compiler, falling back to PATH if no explicit path is provided.
    ///
    /// Default: c3c.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .compiler(&format!("{}/.local/bin/c3c-0.7.6", env!("HOME")))
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compiler<P>(&mut self, compiler: P) -> &mut Self
    where
        P: Into<String>,
    {
        self.compiler = compiler.into();
        self
    }

    /// Whether the library is dynamically or statically linked.
    ///
    /// Default: [LinkingMode::Static].
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .linking_mode(LinkingMode::Dynamic)
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn linking_mode(&mut self, linking_mode: LinkingMode) -> &mut Self {
        self.linking_mode = linking_mode;
        self
    }

    /// The library's optimization level.
    ///
    /// Default: Dpends on compilation mode:
    /// - If debug it returns [OptimizationLevel::O0].
    /// - If release it returns [OptimizationLevel::O1].
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .optimization_level(OptimizationLevel::O0)
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn optimization_level(&mut self, optimization_level: OptimizationLevel) -> &mut Self {
        self.optimization_level = optimization_level;
        self
    }

    /// Whether debug information should be included or not.
    ///
    /// Default: Depends on compilation mode:
    /// - If debug it returns true.
    /// - If release it returns false.
    ///
    /// When `false`, this is equivalent to calling c3c with `-g0`.
    ///
    /// When `true`, this is equivalent to calling c3c with `-g`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .debug_info(true)
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn debug_info(&mut self, debug_info: bool) -> &mut Self {
        self.debug_info = debug_info;
        self
    }

    #[cfg(target_os = "android")]
    pub fn android_ndk_path<P>(&mut self, android_ndk_path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.android_ndk_path = android_ndk_path.into();
        self
    }

    #[cfg(target_os = "windows")]
    pub fn windows_sdk_path<P>(&mut self, windows_sdk_path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.windows_sdk_path = windows_sdk_path.into();
        self
    }

    /// Marks a file as target for compilation.
    ///
    /// Equivalent to calling c3c with the path to a source file.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn file<P>(&mut self, file: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        // At the time of writing, rust-analyzer failed to provide suggestions if not explicitly cast.
        let file = file.into() as PathBuf;
        if !self.files.contains(&file) {
            println!("cargo::rerun-if-changed={}", file.display());
            self.files.push(file);
        }

        self
    }

    /// Marks one or more files for compilation.
    ///
    /// Equivalent to calling c3c with the path to several source files.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .files(["extern/thingmabob.c3", "extern/thingmajane.c3"])
    ///     .compile("thing");
    /// ```
    pub fn files<P>(&mut self, files: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<PathBuf>,
    {
        for file in files {
            self.file(file.into());
        }

        self
    }

    /// Turns on a feature for the provided source files.
    ///
    /// Equivalent to calling c3c with `-D <feature>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .feature("foo")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn feature(&mut self, feature: &str) -> &mut Self {
        if !self.features.contains(&feature.to_string()) {
            self.features.push(feature.to_string());
        }
        self
    }

    /// Turns on one or more features for the provided source files.
    ///
    /// Equivalent to calling c3c with `-D <feature1> [-D <feature2> ... -D <featureN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .features(["foo", "bar"])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn features<P>(&mut self, features: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<String>,
    {
        for feature in features {
            let feature = feature.into() as String;
            self.feature(&feature);
        }

        self
    }

    /// Adds a custom argument to be passed to the compiler.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .arg("--no-headers")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn arg(&mut self, arg: &str) -> &mut Self {
        if !self.args.contains(&arg.to_string()) {
            self.args.push(arg.to_string());
        }
        self
    }

    /// Adds one or more custom arguments to be passed to the compiler.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .args(["--no-headers", "--use-old-enums"])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn args<P>(&mut self, args: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<String>,
    {
        for arg in args {
            let arg = arg.into() as String;
            self.arg(&arg);
        }

        self
    }

    /// Marks an argument to be passed to the linker.
    ///
    /// Equivalent to calling c3c with `-z <arg>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .linker_argument("--enable-linker-version")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn linker_argument(&mut self, linker_argument: &str) -> &mut Self {
        if !self.linker_arguments.contains(&linker_argument.to_string()) {
            self.linker_arguments.push(linker_argument.to_string());
        }
        self
    }

    /// Marks one or more arguments to be passed to the linker.
    ///
    /// Equivalent to calling c3c with `-z <arg1> [-z <arg2> ... -z <argN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .linker_arguments(["--enable-linker-version", "-EB"])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn linker_arguments<P>(&mut self, linker_arguments: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<String>,
    {
        for linker_argument in linker_arguments {
            let linker_argument = linker_argument.into() as String;
            self.linker_argument(&linker_argument);
        }

        self
    }

    /// Sets an environment variable.
    ///
    /// Equivalent to, on Unix systems, calling c3c with an environment variable beforehand `FOO=BAR c3c ...`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .environment_variable(("FOO", "BAR"))
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn environment_variable(&mut self, environment_variable: (&str, &str)) -> &mut Self {
        let environment_variable = (
            environment_variable.0.to_string(),
            environment_variable.1.to_string(),
        );

        if !self.environment_variables.contains(&environment_variable) {
            self.environment_variables.push(environment_variable);
        }
        self
    }

    /// Sets one or more environment variables.
    ///
    /// Equivalent to, on Unix systems, calling c3c with several environment variables beforehand `VAR_1=FOO [VAR_2=BAR ... VAR_N=FOOBAR] c3c ...`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .environment_variables([("VAR_1", "FOO"), ("VAR_2", "BAR")])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn environment_variables<V, P>(&mut self, environment_variables: P) -> &mut Self
    where
        V: Into<String>,
        P: IntoIterator,
        P::Item: Into<(V, V)>,
    {
        for environment_variable in environment_variables {
            let environment_variable: (V, V) = environment_variable.into();
            let environment_variable: (String, String) =
                (environment_variable.0.into(), environment_variable.1.into());

            self.environment_variable((&environment_variable.0, &environment_variable.1));
        }

        self
    }

    /// Marks a directory as containing compiled libraries.
    ///
    /// Equivalent to calling c3c with `-L <dir>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .compiled_lib_dir("libs")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compiled_lib_dir<P>(&mut self, compiled_lib_dir: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        // At the time of writing, rust-analyzer failed to provide suggestions if not explicitly cast.
        let compiled_lib_dir = compiled_lib_dir.into() as PathBuf;
        if !self.compiled_lib_dirs.contains(&compiled_lib_dir) {
            self.compiled_lib_dirs.push(compiled_lib_dir);
        }

        self
    }

    /// Marks one or more directories as containing compiled libraries.
    ///
    /// Equivalent to calling c3c with `-L <dir1> [-L <dir2> ... -L <dirN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .compiled_lib_dirs(["libs", &format("{}/.local.libs", env!("HOME"))])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compiled_lib_dirs<P>(&mut self, compiled_lib_dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<PathBuf>,
    {
        for compiled_lib_dir in compiled_lib_dirs {
            self.compiled_lib_dir(compiled_lib_dir.into());
        }

        self
    }

    /// Marks a file as a compiled library.
    ///
    /// Equivalent to calling c3c with `-l <lib>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .compiled_lib("somelib")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compiled_lib<P>(&mut self, compiled_lib: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        // At the time of writing, rust-analyzer failed to provide suggestions if not explicitly cast.
        let compiled_lib = compiled_lib.into() as PathBuf;
        if !self.compiled_libs.contains(&compiled_lib) {
            self.compiled_libs.push(compiled_lib);
        }

        self
    }

    /// Marks one or more files as a compiled library.
    ///
    /// Equivalent to calling c3c with `-l <lib1> [-l <lib2> ... -l <libN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .compiled_libs(["somelib", "otherlib"])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compiled_libs<P>(&mut self, compiled_libs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<PathBuf>,
    {
        for compiled_lib in compiled_libs {
            self.compiled_lib(compiled_lib.into());
        }

        self
    }

    /// Marks a directory as containing C3 libraries.
    ///
    /// Equivalent to calling c3c with `--libdir <dir>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .c3_lib_dir("libs")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn c3_lib_dir<P>(&mut self, c3_lib_dir: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        // At the time of writing, rust-analyzer failed to provide suggestions if not explicitly cast.
        let c3_lib_dir = c3_lib_dir.into() as PathBuf;
        if !self.c3_lib_dirs.contains(&c3_lib_dir) {
            self.c3_lib_dirs.push(c3_lib_dir);
        }

        self
    }

    /// Marks one or more directories as containing C3 libraries.
    ///
    /// Equivalent to calling c3c with `--libdir <dir1> [--libdir <dir2> ... --libdir <dirN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .c3_lib_dirs(["libs", &format("{}/.local.libs", env!("HOME"))])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn c3_lib_dirs<P>(&mut self, c3_lib_dirs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<PathBuf>,
    {
        for c3_lib_dir in c3_lib_dirs {
            self.c3_lib_dir(c3_lib_dir.into());
        }

        self
    }

    /// Marks a file as a C3 library.
    ///
    /// Equivalent to calling c3c with `--lib <lib>`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .c3_lib("somelib")
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn c3_lib<P>(&mut self, c3_lib: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        // At the time of writing, rust-analyzer failed to provide suggestions if not explicitly cast.
        let c3_lib = c3_lib.into() as PathBuf;
        if !self.c3_libs.contains(&c3_lib) {
            self.c3_libs.push(c3_lib);
        }

        self
    }

    /// Marks one or more files as a C3 library.
    ///
    /// Equivalent to calling c3c with `--lib <lib1> [--lib <lib2> ... --lib <libN>]`.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .c3_libs(["somelib", "otherlib"])
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn c3_libs<P>(&mut self, c3_libs: P) -> &mut Self
    where
        P: IntoIterator,
        P::Item: Into<PathBuf>,
    {
        for c3_lib in c3_libs {
            self.c3_lib(c3_lib.into());
        }

        self
    }

    /// Attempts to compile the provided C3 source files, panicking if it fails to do so.
    ///
    ///
    /// Example:
    /// ```rs
    /// c3ne::C3FFI::new()
    ///     .file("extern/thing.c3")
    ///     .compile("thing");
    /// ```
    pub fn compile(&mut self, name: &str) {
        if let Err(err) = self.attempt_compilation(name) {
            panic!("{}", err);
        }
    }

    /// Attempts to compile the provided C3 source files, returning an error if it fails to do so.
    ///
    ///
    /// Example:
    /// ```rs
    /// if let Err(err) = c3ne::C3FFI::new()
    ///     .files(["extern/thingmabob.c3", "extern/thingmajane.c3"])
    ///     .attempt_compilation("thing")
    /// {
    ///     panic!("{}", err);
    /// }
    /// ```
    pub fn attempt_compilation(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        let out_dir = &var("OUT_DIR")?;

        let rust_features = var("CARGO_CFG_FEATURE").unwrap_or_else(|_| String::new());
        let rust_features: Vec<String> = rust_features
            .split(',')
            .map(|s| s.to_uppercase().replace("-", "_"))
            .collect();

        let target = C3Target::convert_current()?;
        let target_string = target.to_string();

        let command_corresponding_linking = if target.is_wasm() {
            "compile"
        } else {
            &self.linking_mode.to_string()
        };

        let debug_flag = format!("-g{}", if self.debug_info { "" } else { "0" });
        let optimization_level_flag = format!("-{}", self.optimization_level);

        let out_name: String = format!(
            "{}{}",
            if target.is_wasm() || target.is_windows() {
                ""
            } else {
                "lib"
            },
            name
        );

        let args = {
            let mut args: Vec<&str> = Vec::new();

            args.push(command_corresponding_linking);
            args.push(&debug_flag);
            args.push(&optimization_level_flag);
            args.push("--output-dir");
            args.push(out_dir);
            args.push("-o");
            args.push(&out_name);
            args.push("--target");
            args.push(&target_string);
            for feature in &self.features {
                args.push("-D");
                args.push(&feature);
            }
            for feature in &rust_features {
                if !feature.is_empty() {
                    args.push("-D");
                    args.push(&feature);
                }
            }
            for linker_argument in &self.linker_arguments {
                args.push("-z");
                args.push(&linker_argument);
            }
            for c3_lib_dir in &self.c3_lib_dirs {
                args.push("--libdir");
                args.push(c3_lib_dir.as_os_str().to_str().unwrap());
            }
            for c3_lib in &self.c3_libs {
                args.push("--lib");
                args.push(c3_lib.as_os_str().to_str().unwrap());
            }

            if !target.is_wasm() {
                for compiled_lib_dir in &self.compiled_lib_dirs {
                    if !compiled_lib_dir.exists() || !compiled_lib_dir.is_dir() {
                        panic!(
                            "Provided path '{}' for compiled library directory is not valid.",
                            compiled_lib_dir.display()
                        );
                    }

                    args.push("-L");
                    args.push(compiled_lib_dir.as_os_str().to_str().unwrap());
                }

                for compiled_lib in &self.compiled_libs {
                    if compiled_lib.exists() && compiled_lib.is_file() {
                        panic!(
                            "Provided path '{}' for compiled library is not valid.",
                            compiled_lib.display()
                        );
                    }

                    args.push("-l");
                    args.push(compiled_lib.as_os_str().to_str().unwrap());
                }

                #[cfg(target_os = "android")]
                {
                    if !self.android_ndk_path.exists() || !self.android_ndk_path.is_dir() {
                        panic!(
                            "Provided path '{}' is not a valid Android NDK path.",
                            self.android_ndk_path.display()
                        );
                    }

                    args.push("--android-ndk");
                    args.push(self.android_ndk_path.as_os_str().to_str().unwrap());
                }
                #[cfg(target_os = "windows")]
                {
                    if !C3Target::convert(&var("HOST_TARGET")?).is_windows() {
                        if !self.windows_sdk_path.exists() || !self.windows_sdk_path.is_dir() {
                            panic!(
                                "Provided path '{}' is not a valid Windows SDK path.",
                                self.windows_sdk_path.display()
                            );
                        }

                        args.push("--winsdk");
                        args.push(self.windows_sdk_path.as_os_str().to_str().unwrap());
                    }
                }
            } else {
                args.append(&mut vec![
                    "--link-libc=no",
                    "--use-stdlib=yes",
                    "--no-entry",
                    "-z",
                    "--relocatable",
                    "-z",
                    "--allow-undefined",
                ]);
            }

            for file in &self.files {
                args.push(file.as_os_str().to_str().unwrap());
            }
            for arg in &self.args {
                args.push(&arg);
            }

            args
        };

        let mut environment_variables: HashMap<String, String> = HashMap::new();
        for (key, value) in &self.environment_variables {
            environment_variables.insert(key.clone(), value.clone());
        }

        match Command::new(&self.compiler)
            .args(args.clone())
            .envs(environment_variables)
            .output()
        {
            Ok(out) => {
                if !out.status.success() {
                    let error_message = String::from_utf8(out.stderr)?;
                    panic!("{}", error_message)
                }
            }
            Err(err) => panic!("{}", err),
        }

        if !target.is_wasm() {
            println!("cargo:rustc-link-search=native={}", out_dir);
            println!("cargo:rustc-link-lib=static={}", name);
        } else {
            println!(
                "cargo:rustc-link-arg={}",
                Path::new(out_dir)
                    .join(out_name.clone() + ".wasm")
                    .as_os_str()
                    .to_str()
                    .unwrap()
            );
        }

        Ok(())
    }
}

/// Enum containing the available linking modes.
pub enum LinkingMode {
    /// Equivalent to calling c3c with the `static-lib` command.
    Static,
    /// Equivalent to calling c3c with the `dynamic-lib` command.
    Dynamic,
}

impl Display for LinkingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Static => "static-lib",
                Self::Dynamic => "dynamic-lib",
            }
        )
    }
}

/// Enum containing the available optimization levels.
pub enum OptimizationLevel {
    /// Safe, no optimizations, emit debug info.
    ///
    /// Equivalent to calling c3c with `-O0`.
    O0,
    /// Safe, high optimization, emit debug info.
    ///
    /// Equivalent to calling c3c with `-O1`.
    O1,
    /// Unsafe, high optimization, emit debug info.
    ///
    /// Equivalent to calling c3c with `-O2`.
    O2,
    /// Unsafe, high optimization, single module, emit debug info.
    ///
    /// Equivalent to calling c3c with `-O3`.
    O3,
    /// Unsafe, highest optimization, relaxed maths, single module, emit debug info, no panic messages.
    ///
    /// Equivalent to calling c3c with `-O4`.
    O4,
    /// Unsafe, highest optimization, fast maths, single module, emit debug info, no panic messages, no backtrace.
    ///
    /// Equivalent to calling c3c with `-O5`.
    O5,
    /// Unsafe, high optimization, small code, single module, no debug info, no panic messages.
    ///
    /// Equivalent to calling c3c with `-Os`.
    Os,
    /// Unsafe, high optimization, tiny code, single module, no debug info, no panic messages, no backtrace.
    ///
    /// Equivalent to calling c3c with `-Oz`.
    Oz,
}

impl Display for OptimizationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OptimizationLevel::O0 => "O0",
                OptimizationLevel::O1 => "O1",
                OptimizationLevel::O2 => "O2",
                OptimizationLevel::O3 => "O3",
                OptimizationLevel::O4 => "O4",
                OptimizationLevel::O5 => "O5",
                OptimizationLevel::Os => "Os",
                OptimizationLevel::Oz => "Oz",
            }
        )
    }
}
