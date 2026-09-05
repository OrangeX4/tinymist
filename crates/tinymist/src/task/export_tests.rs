use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::Parser;

use super::*;
use crate::export::ProjectCompilation;
use crate::project::{CompileOnceArgs, CompileSignal, WorldProvider};
use crate::world::base::{CompileSnapshot, WorldComputeGraph};

#[test]
fn test_default_never() {
    let conf = ExportUserConfig::default();
    assert!(!conf.count_words);
    assert_eq!(conf.task.when(), Some(&TaskWhen::Never));
}

#[test]
fn compilation_default_never() {
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let verse = args
        .resolve_system()
        .expect("failed to resolve system universe");

    let snap = CompileSnapshot::from_world(verse.snapshot());

    let graph = WorldComputeGraph::new(snap);

    let needs_run = ProjectCompilation::run(&graph).expect("failed to compile diagnostics");

    assert!(!needs_run);
}

// todo: on demand compilation
#[test]
fn compilation_run_paged_diagnostics() {
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let verse = args
        .resolve_system()
        .expect("failed to resolve system universe");

    let mut snap = CompileSnapshot::from_world(verse.snapshot());

    snap.signal = CompileSignal {
        by_entry_update: true,
        by_fs_events: false,
        by_mem_events: false,
    };

    let graph = WorldComputeGraph::new(snap);

    let needs_run = ProjectCompilation::run(&graph).expect("failed to compile diagnostics");

    assert!(needs_run);
}

use chrono::{DateTime, Utc};
use tinymist_std::time::*;

/// Parses a UNIX timestamp according to <https://reproducible-builds.org/specs/source-date-epoch/>
pub fn convert_source_date_epoch(seconds: i64) -> Result<DateTime<Utc>, String> {
    DateTime::from_timestamp(seconds, 0).ok_or_else(|| "timestamp out of range".to_string())
}

/// Parses a UNIX timestamp according to <https://reproducible-builds.org/specs/source-date-epoch/>
pub fn convert_system_time(seconds: i64) -> Result<Time, String> {
    if seconds < 0 {
        return Err("negative timestamp since unix epoch".to_string());
    }

    Time::UNIX_EPOCH
        .checked_add(Duration::new(seconds as u64, 0))
        .ok_or_else(|| "timestamp out of range".to_string())
}

#[test]
fn test_timestamp_chrono() {
    let timestamp = 1_000_000_000;
    let date_time = convert_source_date_epoch(timestamp).unwrap();
    assert_eq!(date_time.timestamp(), timestamp);
}

#[test]
fn test_timestamp_system() {
    let timestamp = 1_000_000_000;
    let date_time = convert_system_time(timestamp).unwrap();
    assert_eq!(
        date_time
            .duration_since(Time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        timestamp as u64
    );
}

use typst::foundations::Datetime as TypstDatetime;

fn convert_datetime_chrono(date_time: DateTime<Utc>) -> Option<TypstDatetime> {
    use chrono::{Datelike, Timelike};
    TypstDatetime::from_ymd_hms(
        date_time.year(),
        date_time.month().try_into().ok()?,
        date_time.day().try_into().ok()?,
        date_time.hour().try_into().ok()?,
        date_time.minute().try_into().ok()?,
        date_time.second().try_into().ok()?,
    )
}

#[test]
fn test_timestamp_pdf() {
    let timestamp = 1_000_000_000;
    let date_time = convert_source_date_epoch(timestamp).unwrap();
    assert_eq!(date_time.timestamp(), timestamp);
    let chrono_pdf_ts = convert_datetime_chrono(date_time).unwrap();

    let timestamp = 1_000_000_000;
    let date_time = convert_system_time(timestamp).unwrap();
    let system_pdf_ts = tinymist_std::time::to_typst_time(date_time.into());
    assert_eq!(chrono_pdf_ts, system_pdf_ts);
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(files: &[(&str, &str)]) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        let root = std::env::temp_dir().join(format!(
            "tinymist-export-path-test-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).expect("failed to create test workspace");

        for (path, contents) in files {
            let path = root.join(path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("failed to create parent directory");
            }
            fs::write(path, contents).expect("failed to write test source");
        }

        Self { root }
    }

    fn graph(&self, main: &str) -> LspComputeGraph {
        let input = self.root.join(main);
        let args = CompileOnceArgs::parse_from([
            "tinymist".to_owned(),
            input.to_string_lossy().into_owned(),
            "--root".to_owned(),
            self.root.to_string_lossy().into_owned(),
        ]);
        let verse = args.resolve().expect("failed to resolve lsp universe");
        let snap = CompileSnapshot::from_world(verse.snapshot());

        WorldComputeGraph::new(snap)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn pdf_task(output: Option<&str>) -> ProjectTask {
    ProjectTask::ExportPdf(ExportPdfTask {
        export: ProjectExportTask {
            when: TaskWhen::Never,
            output: output.map(PathPattern::new),
            transform: vec![],
        },
        ..Default::default()
    })
}

#[test]
fn test_prepare_output_path_preserves_multi_dot_pdf_names() {
    let workspace = TestWorkspace::new(&[
        ("Chapter 1.1.typ", ""),
        ("Chapter 1.1.1.typ", ""),
        ("test....typ", ""),
        ("README", ""),
    ]);
    let task = pdf_task(Some("$root/$dir/$name"));

    for (main, expected) in [
        ("Chapter 1.1.typ", "Chapter 1.1.pdf"),
        ("Chapter 1.1.1.typ", "Chapter 1.1.1.pdf"),
        ("test....typ", "test....pdf"),
        ("README", "README.pdf"),
    ] {
        let graph = workspace.graph(main);
        assert_eq!(
            ExportTask::prepare_output_path(&task, &graph).unwrap(),
            Some(workspace.root.join(expected))
        );
    }
}

#[test]
fn test_prepare_output_path_explicit_dir_name_matches_default() {
    let workspace = TestWorkspace::new(&[
        ("Chapter 1.1.typ", ""),
        ("chapters/Chapter 1.1.typ", ""),
        ("README", ""),
        ("docs/README", ""),
    ]);

    for (main, expected) in [
        ("Chapter 1.1.typ", "Chapter 1.1.pdf"),
        ("chapters/Chapter 1.1.typ", "chapters/Chapter 1.1.pdf"),
        ("README", "README.pdf"),
        ("docs/README", "docs/README.pdf"),
    ] {
        let graph = workspace.graph(main);
        let expected = Some(workspace.root.join(expected));

        assert_eq!(
            ExportTask::prepare_output_path(&pdf_task(None), &graph).unwrap(),
            expected
        );
        assert_eq!(
            ExportTask::prepare_output_path(&pdf_task(Some("$dir/$name")), &graph).unwrap(),
            expected
        );
    }
}
