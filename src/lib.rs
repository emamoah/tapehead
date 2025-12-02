pub mod repl;

use std::{env, path::Path, sync::LazyLock};

pub const VERSION: &str = "0.1.0";

pub const PROGNAME: LazyLock<String> = LazyLock::new(|| {
    let argv0 = env::args().next();
    let basename = argv0.and_then(|path| {
        Path::new(&path)
            .file_name()
            .map(|s| s.to_str().unwrap_or_default().to_string())
    });
    basename.unwrap_or_default()
});
