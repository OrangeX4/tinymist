//! Workspace inventory for completion, references, rename, and symbol search.

use super::{LocalContext, PathKind, TypstFileId, VirtualPath, WorkspaceResolver};
use crate::syntax::scan_workspace_files;
use tinymist_world::{EntryReader, ShadowApi};
use typst_shim::syntax::VirtualPathExt;

impl LocalContext {
    /// Enumerates both filesystem files and editor-owned memory files.
    pub(crate) fn completion_files(&self, pref: &PathKind) -> impl Iterator<Item = &TypstFileId> {
        let regexes = pref.ext_matcher();
        self.caches
            .completion_files
            .get_or_init(|| {
                let Some(root) = self.world().entry_state().workspace_root() else {
                    return vec![];
                };
                let resolve = |path: &std::path::Path| {
                    VirtualPath::virtualize(&root, path)
                        .ok()
                        .map(|path| WorkspaceResolver::workspace_file(Some(&root), path))
                };
                let mut files: Vec<_> =
                    scan_workspace_files(&root, PathKind::Special.ext_matcher(), |path| {
                        resolve(&root.join(path))
                    })
                    .into_iter()
                    .flatten()
                    .collect();
                // A browser workspace and unsaved native files have no on-disk entry.
                files.extend(
                    self.world()
                        .shadow_paths()
                        .iter()
                        .filter_map(|path| resolve(path)),
                );
                let mut seen = std::collections::HashSet::new();
                files.retain(|file| seen.insert(*file));
                files
            })
            .iter()
            .filter(move |fid| {
                fid.vpath()
                    .as_rooted_path_compat()
                    .extension()
                    .and_then(|path| path.to_str())
                    .is_some_and(|path| regexes.is_match(path))
            })
    }
}
