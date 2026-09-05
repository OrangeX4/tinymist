use crate::{Bytes, FileChangeSet, mock::MockWorkspace};

#[test]
fn adding_an_unread_memory_file_changes_the_snapshot_revision() {
    let workspace = MockWorkspace::new("/workspace");
    let mut vfs = workspace.vfs();
    let revision = vfs.revision();
    let path = workspace.path("scratch.typ");
    let content = Ok(Bytes::from_string("#tex".to_owned())).into();
    vfs.revise().map_shadow(&path, content).unwrap();
    assert!(
        vfs.revision() > revision,
        "language snapshots must see new files"
    );
    let revision = vfs.revision();
    let content = Ok(Bytes::from_string("#tex".to_owned())).into();
    vfs.revise().map_shadow(&path, content).unwrap();
    assert_eq!(
        vfs.revision(),
        revision,
        "identical memory bytes are unchanged"
    );
}

#[test]
fn an_unchanged_batch_tail_cannot_hide_an_earlier_change() {
    for changed_first in [true, false] {
        let workspace = MockWorkspace::builder("/workspace")
            .file("main.typ", "before")
            .file("stable.typ", "stable")
            .build();
        let mut vfs = workspace.vfs();
        let main = workspace.file_id("main.typ").unwrap();
        let stable = workspace.file_id("stable.typ").unwrap();
        assert_eq!(vfs.source(main).unwrap().text(), "before");
        assert_eq!(vfs.source(stable).unwrap().text(), "stable");
        let revision = vfs.revision();
        let mut inserts = vec![
            (
                workspace.immut_path("main.typ"),
                Ok(Bytes::from_string("after".to_owned())).into(),
            ),
            (
                workspace.immut_path("stable.typ"),
                Ok(Bytes::from_string("stable".to_owned())).into(),
            ),
        ];
        if !changed_first {
            inserts.reverse();
        }
        vfs.revise()
            .notify_fs_changes(FileChangeSet::new_inserts(inserts));
        assert!(
            vfs.revision() > revision,
            "batch order cannot erase invalidation"
        );
        assert_eq!(vfs.source(main).unwrap().text(), "after");
    }
}

#[test]
fn notified_unread_resources_change_the_snapshot_once() {
    let workspace = MockWorkspace::new("/workspace");
    let mut vfs = workspace.vfs();
    let changes = FileChangeSet::new_inserts(vec![(
        workspace.immut_path("data.json"),
        Ok(Bytes::from_string("{}".to_owned())).into(),
    )]);
    let revision = vfs.revision();
    vfs.revise().notify_fs_changes(changes.clone());
    assert!(vfs.revision() > revision);
    let revision = vfs.revision();
    vfs.revise().notify_fs_changes(changes);
    assert_eq!(vfs.revision(), revision);
}
