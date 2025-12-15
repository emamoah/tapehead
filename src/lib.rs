pub mod repl;
pub mod strings;

use std::{env, path::Path, sync::LazyLock};

pub static PROGNAME: LazyLock<String> = LazyLock::new(|| {
    let argv0 = env::args().next();
    let basename = argv0.and_then(|path| {
        Path::new(&path)
            .file_name()
            .map(|s| s.to_str().unwrap_or_default().to_string())
    });
    basename.unwrap_or_default()
});
