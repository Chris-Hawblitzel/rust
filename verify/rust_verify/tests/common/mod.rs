extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_span;

mod pervasive;
pub use pervasive::pervasive;
pub use rust_verify::verifier::ErrorSpan;

use rust_verify::config::Args;
use rust_verify::verifier::Verifier;

use rustc_span::source_map::FileLoader;

#[derive(Default)]
struct TestFileLoader {
    files: std::collections::HashMap<std::path::PathBuf, String>,
}

impl FileLoader for TestFileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_file(&self, path: &std::path::Path) -> Result<String, std::io::Error> {
        match self.files.get(path) {
            Some(content) => Ok(content.clone()),
            None => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")),
        }
    }
}

pub fn rust_verify_files(
    files: impl IntoIterator<Item = (String, String)>,
    entry_file: String,
) -> Result<(), Vec<(Option<ErrorSpan>, Option<ErrorSpan>)>> {
    let rustc_args = vec![
        "../../install/bin/rust_verify".to_string(),
        "--edition".to_string(),
        "2018".to_string(),
        "--crate-type".to_string(),
        "lib".to_string(),
        "--sysroot".to_string(),
        "../../install".to_string(),
        entry_file,
        "-L".to_string(),
        "../../install/bin/".to_string(),
    ];
    let our_args: Args = Default::default();
    let captured_output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut verifier = Verifier::new(our_args);
    verifier.test_capture_output = Some(captured_output.clone());
    let mut compiler = rustc_driver::RunCompiler::new(&rustc_args, &mut verifier);
    let file_loader: TestFileLoader =
        TestFileLoader { files: files.into_iter().map(|(p, f)| (p.into(), f)).collect() };
    compiler.set_file_loader(Some(Box::new(file_loader)));
    let status = compiler.run();
    eprintln!(
        "{}",
        std::str::from_utf8(
            &captured_output.lock().expect("internal error: cannot lock captured output")
        )
        .expect("captured output is invalid utf8")
    );
    status.map_err(|_| verifier.errors)
}

const PERVASIVE_IMPORT_PRELUDE: &str = indoc::indoc!(
    r###"
    extern crate builtin;
    use builtin::*;
    mod pervasive;
    use pervasive::*;
"###
);

pub fn rust_verify_with_pervasive(
    code: String,
) -> Result<(), Vec<(Option<ErrorSpan>, Option<ErrorSpan>)>> {
    rust_verify_files(
        vec![
            pervasive(),
            ("test.rs".to_string(), format!("{}\n\n{}", PERVASIVE_IMPORT_PRELUDE, code.as_str())),
        ],
        "test.rs".to_string(),
    )
}
