use super::*;
use crate::project::ApplyProjectTask;
use std::sync::atomic::AtomicUsize;
use tinymist_std::fs::paths::write_atomic;

impl ExportTask {
    /// Exports a document.
    pub async fn do_export(
        task: ProjectTask,
        artifact: LspCompiledArtifact,
        lock_dir: Option<ImmutPath>,
    ) -> Result<Option<OnExportResponse>> {
        let CompiledArtifact { graph, .. } = &artifact;

        let Some(write_to) = Self::prepare_output_path(&task, graph)? else {
            return Ok(None);
        };

        static EXPORT_ID: AtomicUsize = AtomicUsize::new(0);
        let export_id = EXPORT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        log::debug!(
            "ExportTask({export_id},lock={lock_dir:?}): exporting {entry:?} to {write_to:?}",
            entry = graph.snap.world.entry_state()
        );
        if let Some(e) = write_to.parent() {
            if !e.exists() {
                std::fs::create_dir_all(e).context("failed to create directory")?;
            }
        }

        let _: Option<()> = lock_dir.and_then(|lock_dir| {
            let mut updater = crate::project::update_lock(lock_dir.clone());
            let root = graph.world().entry_state().root()?;

            let doc_id = updater.compiled(graph.world(), (&root, &lock_dir))?;

            updater.task(ApplyProjectTask {
                id: doc_id.clone(),
                document: doc_id.clone(),
                task: task.clone(),
            });
            updater.update_materials(doc_id.clone(), graph.world().depended_fs_paths());
            updater.route(doc_id, PROJECT_ROUTE_USER_ACTION_PRIORITY);
            updater.commit();

            Some(())
        });

        // Generate the data using common logic
        let artifact = Self::do_export_bytes(task.clone(), artifact, export_id).await?;

        let res = match artifact {
            ExportArtifact::Single(data) => {
                let res = OnExportResponse::Single {
                    path: Some(write_to.clone()),
                    data: None,
                };

                let to = write_to.clone();
                tokio::task::spawn_blocking(move || write_atomic(to, data))
                    .await
                    .context_ut("failed to export")??;

                res
            }
            ExportArtifact::Paged { total_pages, items } => {
                let can_handle_multiple =
                    output_template::has_indexable_template(write_to.to_str().unwrap_or_default());

                if !can_handle_multiple && items.len() > 1 {
                    bail!("cannot export multiple images without a page number template ({{p}}, {{0p}}) in the output path");
                }

                let mut res_items = Vec::new();
                let mut write_futures = Vec::new();
                for (page_idx, bytes) in items {
                    let to = if can_handle_multiple {
                        let storage = output_template::format(
                            write_to.to_str().unwrap_or_default(),
                            page_idx + 1,
                            total_pages,
                        );
                        PathBuf::from(storage)
                    } else {
                        write_to.clone()
                    };

                    res_items.push(PagedExportResponse {
                        page: page_idx,
                        path: Some(to.clone()),
                        data: None,
                    });

                    let fut = tokio::task::spawn_blocking(move || write_atomic(to, bytes));
                    write_futures.push(fut);
                }

                // Await all writes in parallel
                for result in futures::future::join_all(write_futures).await {
                    result.context_ut("failed to export")??;
                }

                OnExportResponse::Paged {
                    total_pages,
                    items: res_items,
                }
            }
            ExportArtifact::Bundle { items } => {
                let root = write_to.clone();
                let fut = tokio::task::spawn_blocking(move || write_bundle_files(&root, &items));
                fut.await.context_ut("failed to export")??;

                OnExportResponse::Single {
                    path: Some(write_to),
                    data: None,
                }
            }
        };

        log::debug!("ExportTask({export_id}): export complete");
        Ok(Some(res))
    }
}

fn write_bundle_files(root: &Path, items: &[(PathBuf, Bytes)]) -> Result<()> {
    std::fs::create_dir_all(root).context("failed to create output directory")?;
    for (path, data) in items {
        let realized = root.join(path);
        if let Some(parent) = realized.parent() {
            std::fs::create_dir_all(parent).context("failed to create directory")?;
        }
        write_atomic(realized, data.clone()).context("failed to write file")?;
    }
    Ok(())
}
