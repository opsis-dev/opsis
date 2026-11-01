use std::collections::HashSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use opsis_compile::asset_ref::{
    base_name, extension_of, library_ref, library_rel_path, project_ref,
};
use opsis_compile::fonts::{font_stack, same_family};
use opsis_compile::text::slug_name;
use opsis_engine::files::none_if_missing;
use opsis_store::db::Database;
use opsis_types::fonts::{
    FontCategory, FontFaceDef, FontFamilyDef, FontFormat, FontSource, FontStyle,
};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::inspect::{
    FontBytes, FontCandidate, PlannedFamily, format_of, guess_category, installed_file_name,
    is_license_name, plan_font_files, read_candidate,
};

const LICENSE_FILE: &str = "LICENSE.txt";
const FONT_FALLBACK: &str = "font";
const MAC_RESOURCES: &str = "__MACOSX";

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Those fonts are no longer ready to add. Choose them again.")]
    NotReady,
    #[error("That font file is damaged.")]
    Damaged(#[from] zip::result::ZipError),
    #[error(transparent)]
    Store(#[from] opsis_store::db::StoreError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct InspectedFonts {
    pub token: String,
    pub families: Vec<PlannedFamily>,
    pub license: Option<String>,
    pub skipped: u32,
    pub unreadable: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct InstalledFamily {
    pub family: String,
    pub faces: u32,
    pub stack: String,
}

#[derive(Debug)]
pub struct NewFace {
    pub name: String,
    pub bytes: Arc<[u8]>,
    pub weight: String,
    pub style: FontStyle,
    pub format: FontFormat,
}

#[derive(Debug)]
pub struct NewFamily {
    pub family: String,
    pub category: FontCategory,
    pub source: FontSource,
    pub files: Vec<NewFace>,
    pub license: Option<FontBytes>,
    pub license_type: Option<String>,
}

#[derive(Debug, Clone)]
struct FontFile {
    candidate: FontCandidate,
    bytes: Arc<[u8]>,
}

impl AsRef<FontCandidate> for FontFile {
    fn as_ref(&self) -> &FontCandidate {
        &self.candidate
    }
}

#[derive(Debug)]
struct Pending {
    token: String,
    families: Vec<PlannedFamily<FontFile>>,
    license: Option<FontBytes>,
    source: FontSource,
}

struct Library {
    families: Vec<FontFamilyDef>,
    ids: Vec<String>,
}

async fn read_library(db: &Database) -> Result<Library, FontError> {
    let mut library = Library {
        families: Vec::new(),
        ids: Vec::new(),
    };
    for row in db.font_rows().await? {
        match serde_json::from_str(&row.def) {
            Ok(family) => library.families.push(family),
            Err(_) => log::warn!("{} couldn't be read and was kept as it is", row.font_id),
        }
        library.ids.push(row.font_id);
    }
    Ok(library)
}

fn unique_id(base: &str, taken: &[String]) -> String {
    std::iter::once(base.to_owned())
        .chain((2..).map(|n| format!("{base}-{n}")))
        .take(taken.len() + 1)
        .find(|id| !taken.contains(id))
        .expect("a free id")
}

pub fn font_slug(family: &str) -> String {
    slug_name(family, FONT_FALLBACK)
}

fn file_name_for(name: &str) -> String {
    match sanitize_filename::sanitize(base_name(name)) {
        clean if clean.trim_matches(['.', ' ']).is_empty() => FONT_FALLBACK.to_owned(),
        clean => clean,
    }
}

fn zip_entries(bytes: &[u8]) -> Result<Vec<FontBytes>, FontError> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name()?.into_owned();
        if file.is_dir() || name.contains(MAC_RESOURCES) {
            continue;
        }
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        entries.push(FontBytes { name, bytes: data });
    }
    Ok(entries)
}

#[derive(Debug)]
pub struct FontLibrary {
    dir: PathBuf,
    pending: Mutex<Option<Pending>>,
}

impl FontLibrary {
    pub fn new(user_data: &Path) -> Self {
        Self {
            dir: user_data.join("fonts"),
            pending: Mutex::new(None),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub async fn list(&self, db: &Database) -> Result<Vec<FontFamilyDef>, FontError> {
        Ok(read_library(db).await?.families)
    }

    fn hold_for_review(&self, entries: Vec<FontBytes>, source: FontSource) -> InspectedFonts {
        let mut candidates = Vec::new();
        let mut license = None;
        let mut unreadable = 0;
        for entry in entries {
            if format_of(&entry.name).is_some() {
                match read_candidate(&entry.name, &entry.bytes) {
                    Some(candidate) => candidates.push(FontFile {
                        candidate,
                        bytes: entry.bytes.into(),
                    }),
                    None => unreadable += 1,
                }
            } else if license.is_none() && is_license_name(&entry.name) {
                license = Some(entry);
            }
        }
        let plan = plan_font_files(&candidates);
        let kept: usize = plan.iter().map(|f| f.faces.len()).sum();
        let token = uuid::Uuid::new_v4().to_string();
        let inspected = InspectedFonts {
            token: token.clone(),
            families: plan
                .iter()
                .map(|f| PlannedFamily {
                    family: f.family.clone(),
                    faces: f.faces.iter().map(|c| c.candidate.clone()).collect(),
                })
                .collect(),
            license: license.as_ref().map(|l| base_name(&l.name)),
            skipped: u32::try_from(candidates.len() - kept).unwrap_or(u32::MAX),
            unreadable,
        };
        *self.pending.lock().expect("pending fonts") = (!plan.is_empty()).then_some(Pending {
            token,
            families: plan,
            license,
            source,
        });
        inspected
    }

    pub fn inspect_files(&self, paths: &[PathBuf]) -> Result<InspectedFonts, FontError> {
        let mut entries = Vec::new();
        for file in paths {
            let bytes = std::fs::read(file)?;
            let name = file
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if extension_of(&name) == "zip" {
                entries.extend(zip_entries(&bytes)?);
            } else {
                entries.push(FontBytes { name, bytes });
            }
        }
        Ok(self.hold_for_review(entries, FontSource::File))
    }

    pub fn inspect_installed(
        &self,
        system: &fontdb::Database,
        families: &[String],
    ) -> InspectedFonts {
        let entries = system
            .faces()
            .filter(|face| {
                face.families
                    .iter()
                    .any(|(name, _)| families.iter().any(|f| same_family(f, name)))
            })
            .filter_map(|face| match &face.source {
                fontdb::Source::File(path) if face.index == 0 => {
                    let bytes = std::fs::read(path).ok()?;
                    Some(FontBytes {
                        name: installed_file_name(&face.post_script_name, &bytes),
                        bytes,
                    })
                }
                _ => None,
            })
            .collect();
        self.hold_for_review(entries, FontSource::Installed)
    }

    pub async fn add_inspected(
        &self,
        db: &Database,
        token: &str,
        families: &[String],
    ) -> Result<Vec<FontFamilyDef>, FontError> {
        let pending = {
            let mut held = self.pending.lock().expect("pending fonts");
            if held.as_ref().is_none_or(|p| p.token != token) {
                return Err(FontError::NotReady);
            }
            held.take().ok_or(FontError::NotReady)?
        };
        let license = pending.license.map(Arc::new);
        let chosen = pending
            .families
            .into_iter()
            .filter(|f| families.contains(&f.family))
            .map(|fam| NewFamily {
                category: guess_category(&fam.family),
                family: fam.family,
                source: pending.source,
                files: fam
                    .faces
                    .into_iter()
                    .map(|f| NewFace {
                        name: f.candidate.name,
                        bytes: f.bytes,
                        weight: f.candidate.weight,
                        style: f.candidate.style,
                        format: f.candidate.format,
                    })
                    .collect(),
                license: license.as_deref().map(|l| FontBytes {
                    name: l.name.clone(),
                    bytes: l.bytes.clone(),
                }),
                license_type: None,
            })
            .collect();
        self.add_families(db, chosen).await
    }

    pub fn release_inspected(&self, token: &str) {
        let mut held = self.pending.lock().expect("pending fonts");
        if held.as_ref().is_some_and(|p| p.token == token) {
            *held = None;
        }
    }

    fn write_family(&self, library: &Library, fam: NewFamily) -> Result<FontFamilyDef, FontError> {
        let id = match library
            .families
            .iter()
            .find(|f| same_family(&f.family, &fam.family))
        {
            Some(existing) => existing.id.clone(),
            None => unique_id(&font_slug(&fam.family), &library.ids),
        };
        let dir = self.dir.join(&id);
        none_if_missing(std::fs::remove_dir_all(&dir))?;
        std::fs::create_dir_all(&dir)?;
        let mut faces = Vec::new();
        for file in fam.files {
            let name = file_name_for(&file.name);
            std::fs::write(dir.join(&name), &file.bytes)?;
            faces.push(FontFaceDef {
                file: library_ref(&format!("{id}/{name}")),
                weight: file.weight,
                style: file.style,
                format: file.format,
            });
        }
        if let Some(license) = &fam.license {
            std::fs::write(dir.join(LICENSE_FILE), &license.bytes)?;
        }
        Ok(FontFamilyDef {
            license_file: fam
                .license
                .map(|_| library_ref(&format!("{id}/{LICENSE_FILE}"))),
            id,
            family: fam.family,
            category: fam.category,
            source: fam.source,
            license: fam.license_type,
            faces,
        })
    }

    pub async fn add_families(
        &self,
        db: &Database,
        families: Vec<NewFamily>,
    ) -> Result<Vec<FontFamilyDef>, FontError> {
        let mut added = Vec::new();
        for fam in families {
            let def = self.write_family(&read_library(db).await?, fam)?;
            let row = serde_json::to_string(&def).map_err(std::io::Error::other)?;
            db.save_font_row(&def.id, &row).await?;
            added.push(def);
        }
        Ok(added)
    }

    pub async fn remove(&self, db: &Database, id: &str) -> Result<(), FontError> {
        none_if_missing(std::fs::remove_dir_all(self.dir.join(id)))?;
        db.remove_font_row(id).await?;
        Ok(())
    }

    pub fn copy_to_project(
        &self,
        project_dir: &Path,
        def: &FontFamilyDef,
    ) -> Result<FontFamilyDef, FontError> {
        let dest = project_dir.join("assets").join("fonts").join(&def.id);
        std::fs::create_dir_all(&dest)?;
        let to_project = |r: &str| -> std::io::Result<String> {
            let rel = library_rel_path(r).unwrap_or(r);
            let base = base_name(rel);
            std::fs::copy(self.dir.join(rel), dest.join(&base))?;
            Ok(project_ref(&format!("assets/fonts/{}/{base}", def.id)))
        };
        let mut faces = Vec::new();
        for face in &def.faces {
            faces.push(FontFaceDef {
                file: to_project(&face.file)?,
                ..face.clone()
            });
        }
        Ok(FontFamilyDef {
            faces,
            license_file: def.license_file.as_deref().map(to_project).transpose()?,
            ..def.clone()
        })
    }
}

pub fn installed_families(system: &fontdb::Database) -> Vec<InstalledFamily> {
    let mut counts: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    let mut seen = HashSet::new();
    for face in system.faces() {
        if let Some((family, _)) = face.families.first()
            && seen.insert((family.clone(), face.post_script_name.clone()))
        {
            *counts.entry(family.clone()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .map(|(family, faces)| InstalledFamily {
            stack: font_stack(&family, &[]),
            family,
            faces,
        })
        .collect()
}

pub fn system_fonts() -> fontdb::Database {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    db
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundled(file: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../public/fonts/bundled")
            .join(file)
    }

    fn old_font() -> NewFamily {
        NewFamily {
            family: "Old".into(),
            category: FontCategory::SansSerif,
            source: FontSource::File,
            files: vec![NewFace {
                name: "old.woff2".into(),
                bytes: Arc::from(vec![1u8]),
                weight: "400".into(),
                style: FontStyle::Normal,
                format: FontFormat::Woff2,
            }],
            license: None,
            license_type: None,
        }
    }

    async fn setup() -> (tempfile::TempDir, FontLibrary, Database) {
        let user_data = tempfile::tempdir().unwrap();
        let library = FontLibrary::new(user_data.path());
        (user_data, library, Database::in_memory().await.unwrap())
    }

    #[tokio::test]
    async fn reads_the_family_weight_range_and_style_of_each_face_and_finds_the_licence() {
        let (_dir, library, _db) = setup().await;
        let inspected = library
            .inspect_files(&[
                bundled("dm-sans/dm-sans-latin-wght-normal.woff2"),
                bundled("dm-sans/dm-sans-latin-wght-italic.woff2"),
                bundled("dm-sans/LICENSE"),
            ])
            .unwrap();
        let faces: Vec<(&str, FontStyle, &str)> = inspected.families[0]
            .faces
            .iter()
            .map(|f| (f.name.as_str(), f.style, f.weight.as_str()))
            .collect();
        assert_eq!(inspected.families.len(), 1);
        assert_eq!(inspected.families[0].family, "DM Sans 9pt");
        assert_eq!(
            faces,
            [
                (
                    "dm-sans-latin-wght-normal.woff2",
                    FontStyle::Normal,
                    "100 1000"
                ),
                (
                    "dm-sans-latin-wght-italic.woff2",
                    FontStyle::Italic,
                    "100 1000"
                )
            ]
        );
        assert_eq!(
            (
                inspected.license.as_deref(),
                inspected.skipped,
                inspected.unreadable
            ),
            (Some("LICENSE"), 0, 0)
        );
    }

    #[tokio::test]
    async fn reads_fonts_inside_a_zip_and_counts_files_it_cant_read() {
        let (dir, library, _db) = setup().await;
        let zip_file = dir.path().join("fonts.zip");
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&zip_file).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        for (name, bytes) in [
            (
                "DM/dm-sans-latin-wght-normal.woff2",
                std::fs::read(bundled("dm-sans/dm-sans-latin-wght-normal.woff2")).unwrap(),
            ),
            ("__MACOSX/DM/._dm-sans.woff2", vec![0]),
            ("DM/broken.ttf", vec![1, 2, 3, 4]),
            ("DM/OFL.txt", b"licence".to_vec()),
        ] {
            zip.start_file(name, options).unwrap();
            std::io::Write::write_all(&mut zip, &bytes).unwrap();
        }
        zip.finish().unwrap();
        let inspected = library.inspect_files(&[zip_file]).unwrap();
        assert_eq!(
            inspected.families[0].faces[0].name,
            "DM/dm-sans-latin-wght-normal.woff2"
        );
        assert_eq!(
            (inspected.license.as_deref(), inspected.unreadable),
            (Some("OFL.txt"), 1)
        );
    }

    #[tokio::test]
    async fn adds_the_chosen_families_with_their_licence_and_copies_them_into_a_project() {
        let (_dir, library, db) = setup().await;
        let inspected = library
            .inspect_files(&[
                bundled("dm-sans/dm-sans-latin-wght-normal.woff2"),
                bundled("dm-sans/LICENSE"),
            ])
            .unwrap();
        let [added] = library
            .add_inspected(&db, &inspected.token, &["DM Sans 9pt".into()])
            .await
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(added.id, "dm-sans-9pt");
        assert_eq!(
            added.faces[0].file,
            "library:dm-sans-9pt/dm-sans-latin-wght-normal.woff2"
        );
        assert_eq!(
            added.license_file.as_deref(),
            Some("library:dm-sans-9pt/LICENSE.txt")
        );
        assert_eq!(added.category, FontCategory::SansSerif);
        let project = tempfile::tempdir().unwrap();
        let copied = library.copy_to_project(project.path(), &added).unwrap();
        assert_eq!(
            copied.faces[0].file,
            "project:assets/fonts/dm-sans-9pt/dm-sans-latin-wght-normal.woff2"
        );
        assert!(
            project
                .path()
                .join("assets/fonts/dm-sans-9pt/LICENSE.txt")
                .exists()
        );
    }

    #[tokio::test]
    async fn lets_go_of_fonts_the_review_closed_without_adding_and_keeps_only_the_latest() {
        let (_dir, library, db) = setup().await;
        let files = [bundled("dm-sans/dm-sans-latin-wght-normal.woff2")];
        let family = ["DM Sans 9pt".to_owned()];
        let released = library.inspect_files(&files).unwrap();
        library.release_inspected(&released.token);
        assert!(matches!(
            library.add_inspected(&db, &released.token, &family).await,
            Err(FontError::NotReady)
        ));
        let first = library.inspect_files(&files).unwrap();
        library.inspect_files(&files).unwrap();
        assert!(matches!(
            library.add_inspected(&db, &first.token, &family).await,
            Err(FontError::NotReady)
        ));
    }

    #[tokio::test]
    async fn keeps_entries_it_cant_read_when_the_library_changes() {
        let (_dir, library, db) = setup().await;
        let unreadable = r#"{"id":"old","family":"Old"}"#;
        db.save_font_row("old", unreadable).await.unwrap();
        library.add_families(&db, vec![old_font()]).await.unwrap();
        let rows = db.font_rows().await.unwrap();
        assert_eq!(
            (rows[0].font_id.as_str(), rows[0].def.as_str()),
            ("old", unreadable)
        );
        assert_eq!(
            library
                .list(&db)
                .await
                .unwrap()
                .iter()
                .map(|f| f.id.as_str())
                .collect::<Vec<_>>(),
            ["old-2"]
        );
    }

    #[tokio::test]
    async fn replaces_a_family_added_again_and_removes_it_by_id() {
        let (_dir, library, db) = setup().await;
        let [first] = library
            .add_families(&db, vec![old_font()])
            .await
            .unwrap()
            .try_into()
            .unwrap();
        library.add_families(&db, vec![old_font()]).await.unwrap();
        assert_eq!(
            library
                .list(&db)
                .await
                .unwrap()
                .iter()
                .map(|f| f.id.as_str())
                .collect::<Vec<_>>(),
            [first.id.as_str()]
        );
        library.remove(&db, &first.id).await.unwrap();
        assert_eq!(library.list(&db).await.unwrap(), []);
        assert!(!library.dir().join(&first.id).exists());
    }
}
