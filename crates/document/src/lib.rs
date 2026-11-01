use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use opsis_compile::edits::timeline::{Film, TimelineEdits, timeline_reply};
use opsis_compile::edits::{EditContext, EditError, apply_edit, edited_nodes};
use opsis_compile::ids::{Ids, RandomIds, SaltedIds, random_id};
use opsis_compile::node::Platform;
use opsis_types::edits::{
    DocumentState, Edited, Removal, TimelineEdit, TimelineReply, WorkflowEdit,
};
use opsis_types::workflow::Workflow;

pub const MAX_UNDO_STEPS: usize = 200;

#[derive(Debug)]
struct Gesture {
    depth: u32,
    start: Workflow,
    salt: String,
}

#[derive(Debug)]
pub struct Document {
    session: String,
    dir: PathBuf,
    workflow: Workflow,
    past: VecDeque<Workflow>,
    future: Vec<Workflow>,
    gesture: Option<Gesture>,
    revision: u64,
    saved_revision: u64,
    file_version: String,
    platform: Platform,
    edited: Vec<String>,
}

impl Document {
    pub fn open(dir: &Path, workflow: Workflow, file_version: String, platform: Platform) -> Self {
        Self {
            session: random_id("doc"),
            dir: dir.to_path_buf(),
            workflow: workflow.normalized(),
            past: VecDeque::new(),
            future: Vec::new(),
            gesture: None,
            revision: 0,
            saved_revision: 0,
            file_version,
            platform,
            edited: Vec::new(),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn workflow(&self) -> &Workflow {
        &self.workflow
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn file_version(&self) -> &str {
        &self.file_version
    }

    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn state(&self) -> DocumentState {
        DocumentState {
            session: self.session.clone(),
            dir: self.dir.to_string_lossy().into_owned(),
            workflow: self.workflow.clone(),
            revision: self.revision,
            saved_revision: self.saved_revision,
            saving: false,
            can_undo: !self.past.is_empty(),
            can_redo: !self.future.is_empty(),
        }
    }

    pub fn apply(&mut self, edit: WorkflowEdit) -> Result<Edited, EditError> {
        self.apply_with(edit, &mut RandomIds)
    }

    pub fn apply_with(
        &mut self,
        edit: WorkflowEdit,
        ids: &mut dyn Ids,
    ) -> Result<Edited, EditError> {
        let mut next = self.workflow.clone();
        let mut ctx = EditContext {
            platform: self.platform,
            ids,
        };
        let created = apply_edit(&mut next, edit, &mut ctx)?;
        self.commit(next);
        Ok(Edited {
            revision: self.revision,
            ids: created,
        })
    }

    fn remember(&mut self, previous: Workflow) {
        self.past.push_back(previous);
        if self.past.len() > MAX_UNDO_STEPS {
            self.past.pop_front();
        }
        self.future.clear();
    }

    fn replace_workflow(&mut self, next: Workflow) -> Workflow {
        for id in edited_nodes(&self.workflow, &next) {
            if !self.edited.contains(&id) {
                self.edited.push(id);
            }
        }
        std::mem::replace(&mut self.workflow, next)
    }

    pub fn take_edited(&mut self) -> Vec<String> {
        std::mem::take(&mut self.edited)
    }

    fn commit(&mut self, next: Workflow) -> bool {
        if next == self.workflow {
            return false;
        }
        let previous = self.replace_workflow(next);
        if self.gesture.is_none() {
            self.remember(previous);
        }
        self.revision += 1;
        true
    }

    pub fn begin_gesture(&mut self) {
        match self.gesture.as_mut() {
            Some(gesture) => gesture.depth += 1,
            None => {
                self.gesture = Some(Gesture {
                    depth: 1,
                    start: self.workflow.clone(),
                    salt: random_id("g"),
                });
            }
        }
    }

    pub fn end_gesture(&mut self) {
        let Some(gesture) = self.gesture.as_mut() else {
            return;
        };
        gesture.depth -= 1;
        if gesture.depth > 0 {
            return;
        }
        if let Some(Gesture { start, .. }) = self.gesture.take()
            && start != self.workflow
        {
            self.remember(start);
        }
    }

    pub fn gesture_start(&self) -> Option<&Workflow> {
        self.gesture.as_ref().map(|g| &g.start)
    }

    pub fn edit_timeline(&mut self, edit: TimelineEdit, film: &Film) -> TimelineReply {
        let base = self.gesture_start().unwrap_or(&self.workflow).clone();
        let mut salted = self.gesture.as_ref().map(|g| SaltedIds::new(&g.salt));
        let mut random = RandomIds;
        let ids: &mut dyn Ids = match salted.as_mut() {
            Some(salted) => salted,
            None => &mut random,
        };
        let mut edits = TimelineEdits {
            workflow: base,
            fps: film.flat.fps,
            source: film.source.clone(),
            ids,
        };
        let outcome = edits.apply(edit, &film.flat);
        if outcome.is_ok() {
            let next = edits.workflow;
            self.commit(next);
        }
        timeline_reply(&outcome)
    }

    pub fn timeline_removal(&self, film: &Film, selected: &[String]) -> Option<Removal> {
        let edits = TimelineEdits {
            workflow: self.workflow.clone(),
            fps: film.flat.fps,
            source: film.source.clone(),
            ids: &mut RandomIds,
        };
        edits.selection_removal(&film.flat, selected)
    }

    pub fn undo(&mut self) -> bool {
        self.end_open_gesture();
        let Some(previous) = self.past.pop_back() else {
            return false;
        };
        let current = self.replace_workflow(previous);
        self.future.push(current);
        self.revision += 1;
        true
    }

    pub fn redo(&mut self) -> bool {
        self.end_open_gesture();
        let Some(next) = self.future.pop() else {
            return false;
        };
        let current = self.replace_workflow(next);
        self.past.push_back(current);
        self.revision += 1;
        true
    }

    fn end_open_gesture(&mut self) {
        if let Some(gesture) = self.gesture.as_mut() {
            gesture.depth = 1;
            self.end_gesture();
        }
    }

    pub fn mark_saved(&mut self, revision: u64, file_version: String) {
        self.saved_revision = revision;
        self.file_version = file_version;
    }

    pub fn keep_file_version(&mut self, file_version: String) {
        self.file_version = file_version;
    }

    pub fn load_from_disk(&mut self, workflow: Workflow, file_version: String) {
        self.end_open_gesture();
        self.commit(workflow.normalized());
        self.mark_saved(self.revision, file_version);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/todomvc/project.opsis.json"
    );

    fn project() -> Workflow {
        let text = std::fs::read_to_string(EXAMPLE).unwrap();
        let mut workflow: Workflow = serde_json::from_str(&text).unwrap();
        workflow.name = "Start".into();
        workflow
    }

    fn open() -> Document {
        Document::open(
            Path::new("/project"),
            project(),
            "v0".into(),
            Platform::Linux,
        )
    }

    fn rename(doc: &mut Document, name: &str) {
        doc.apply(WorkflowEdit::Rename { name: name.into() })
            .unwrap();
    }

    #[test]
    fn opens_clean_with_no_history() {
        let doc = open();
        assert!(!doc.is_dirty());
        assert!(!doc.state().can_undo);
        assert!(!doc.state().can_redo);
    }

    #[test]
    fn records_a_gesture_as_one_undo_step_and_counts_undo_and_redo_as_edits() {
        let mut doc = open();
        doc.begin_gesture();
        rename(&mut doc, "A");
        rename(&mut doc, "AB");
        doc.end_gesture();
        assert_eq!(doc.past.len(), 1);
        doc.mark_saved(doc.revision(), "v1".into());

        assert!(doc.undo());
        assert_eq!(doc.workflow().name, "Start");
        assert!(doc.is_dirty());

        doc.mark_saved(doc.revision(), "v2".into());
        assert!(doc.redo());
        assert_eq!(doc.workflow().name, "AB");
        assert!(doc.is_dirty());
    }

    #[test]
    fn treats_nested_gestures_as_one_and_records_nothing_for_a_gesture_without_changes() {
        let mut doc = open();
        doc.begin_gesture();
        doc.begin_gesture();
        rename(&mut doc, "A");
        doc.end_gesture();
        rename(&mut doc, "B");
        doc.end_gesture();
        assert_eq!(doc.past.len(), 1);

        doc.begin_gesture();
        doc.end_gesture();
        rename(&mut doc, "C");
        assert_eq!(doc.past.len(), 2);
    }

    #[test]
    fn does_not_change_the_revision_when_there_is_nothing_to_undo_or_no_change() {
        let mut doc = open();
        assert!(!doc.undo());
        rename(&mut doc, "Start");
        assert_eq!(doc.revision(), 0);
    }

    #[test]
    fn closes_a_gesture_left_open_when_undoing() {
        let mut doc = open();
        doc.begin_gesture();
        rename(&mut doc, "A");
        assert!(doc.undo());
        assert_eq!(doc.workflow().name, "Start");
        assert!(doc.gesture_start().is_none());
        rename(&mut doc, "B");
        assert_eq!(doc.past.len(), 1);
    }

    #[test]
    fn names_the_nodes_each_change_edited_until_they_are_taken() {
        let mut doc = open();
        let id = doc.workflow().nodes[0].id.clone();
        let patch = opsis_types::edits::NodePatch {
            disabled: Some(true),
            ..Default::default()
        };
        doc.apply(WorkflowEdit::UpdateNode {
            id: id.clone(),
            patch,
        })
        .unwrap();
        assert!(doc.take_edited().contains(&id));
        assert_eq!(doc.take_edited(), [] as [String; 0]);
        doc.undo();
        assert!(doc.take_edited().contains(&id));
    }

    #[test]
    fn keeps_at_most_the_undo_limit() {
        let mut doc = open();
        for step in 0..=MAX_UNDO_STEPS {
            rename(&mut doc, &step.to_string());
        }
        assert_eq!(doc.past.len(), MAX_UNDO_STEPS);
    }

    #[test]
    fn loads_the_file_from_disk_as_one_undo_step_and_marks_it_saved() {
        let mut doc = open();
        let mut changed = project();
        changed.name = "On disk".into();
        doc.load_from_disk(changed, "v9".into());
        assert_eq!(doc.workflow().name, "On disk");
        assert!(!doc.is_dirty());
        assert_eq!(doc.file_version(), "v9");
        assert!(doc.undo());
        assert_eq!(doc.workflow().name, "Start");
    }
}
