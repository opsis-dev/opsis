use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use opsis_store::db::Database;
use opsis_types::fonts::{FontCategory, FontFamilyDef, FontFormat, FontSource, FontStyle};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

use crate::inspect::{FontBytes, guess_category};
use crate::library::{FontError, FontLibrary, NewFace, NewFamily};

const API: &str = "https://api.fontsource.org/v1";
const CDN: &str = "https://cdn.jsdelivr.net";
const LATIN: &str = "latin";
const CHECK_CONNECTION: &str = "Check your internet connection.";
const LIST_FAILED: &str = "Couldn't load the font list.";
const FIND_FAILED: &str = "Couldn't find that font online.";
const FILE_FAILED: &str = "A font file didn't download.";
const AXES_FAILED: &str = "A font didn't download.";

#[derive(Debug, thiserror::Error)]
pub enum OnlineError {
    #[error("{0} {CHECK_CONNECTION}")]
    Offline(&'static str),
    #[error("{0}")]
    Failed(&'static str),
    #[error("Couldn't read that font's details.")]
    Unreadable,
    #[error("That font has no Latin files to download.")]
    NoLatinFiles,
    #[error(transparent)]
    Library(#[from] FontError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum CatalogCategory {
    SansSerif,
    Serif,
    Monospace,
    Display,
    Handwriting,
    Icons,
    #[default]
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FontCatalogEntry {
    pub id: String,
    pub family: String,
    #[serde(default)]
    pub category: CatalogCategory,
    pub variable: bool,
    pub styles: Vec<String>,
    pub weights: Vec<u32>,
    pub license: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize)]
struct CatalogRow {
    #[serde(flatten)]
    entry: FontCatalogEntry,
    subsets: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct FileUrls {
    woff2: Option<String>,
}

#[derive(Deserialize)]
struct Subset {
    url: FileUrls,
}

#[derive(Deserialize)]
struct Detail {
    family: String,
    #[serde(default)]
    category: Value,
    variable: bool,
    styles: Vec<String>,
    weights: Vec<u32>,
    #[serde(default)]
    license: Value,
    #[serde(default)]
    variants: HashMap<String, HashMap<String, HashMap<String, Subset>>>,
}

#[derive(Deserialize)]
struct Axis {
    min: Value,
    max: Value,
}

#[derive(Deserialize)]
struct Axes {
    #[serde(default)]
    axes: HashMap<String, Axis>,
}

fn number_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

pub struct FontsOnline {
    client: reqwest::Client,
    api: String,
    cdn: String,
    catalog: Mutex<Option<Arc<Vec<FontCatalogEntry>>>>,
}

impl Default for FontsOnline {
    fn default() -> Self {
        Self::with_hosts(API, CDN)
    }
}

impl FontsOnline {
    pub fn with_hosts(api: &str, cdn: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api: api.to_owned(),
            cdn: cdn.to_owned(),
            catalog: Mutex::new(None),
        }
    }

    async fn fetch(
        &self,
        url: &str,
        failure: &'static str,
    ) -> Result<reqwest::Response, OnlineError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| OnlineError::Offline(failure))?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(OnlineError::Failed(failure))
        }
    }

    async fn bytes(&self, url: &str) -> Result<Vec<u8>, OnlineError> {
        let response = self.fetch(url, FILE_FAILED).await?;
        Ok(response
            .bytes()
            .await
            .map_err(|_| OnlineError::Offline(FILE_FAILED))?
            .to_vec())
    }

    pub async fn catalog(&self) -> Result<Arc<Vec<FontCatalogEntry>>, OnlineError> {
        if let Some(catalog) = self.catalog.lock().expect("font catalog").clone() {
            return Ok(catalog);
        }
        let rows: Vec<Value> = self
            .fetch(&format!("{}/fonts", self.api), LIST_FAILED)
            .await?
            .json()
            .await
            .map_err(|_| OnlineError::Failed(LIST_FAILED))?;
        let catalog: Arc<Vec<FontCatalogEntry>> = Arc::new(
            rows.into_iter()
                .filter_map(|row| serde_json::from_value::<CatalogRow>(row).ok())
                .filter(|row| {
                    row.subsets
                        .as_ref()
                        .is_none_or(|s| s.iter().any(|s| s == LATIN))
                })
                .map(|row| row.entry)
                .collect(),
        );
        *self.catalog.lock().expect("font catalog") = Some(Arc::clone(&catalog));
        Ok(catalog)
    }

    async fn weight_axis(&self, id: &str) -> Result<Option<(String, String)>, OnlineError> {
        let text = self
            .fetch(
                &format!("{}/variable/{}", self.api, encode(id)),
                AXES_FAILED,
            )
            .await?
            .text()
            .await;
        let axes = text
            .ok()
            .and_then(|t| serde_json::from_str::<Axes>(&t).ok());
        Ok(axes
            .and_then(|mut a| a.axes.remove("wght"))
            .map(|axis| (number_text(&axis.min), number_text(&axis.max))))
    }

    async fn faces(&self, id: &str, detail: &Detail) -> Result<Vec<NewFace>, OnlineError> {
        let styles: Vec<FontStyle> = detail
            .styles
            .iter()
            .filter_map(|s| match s.as_str() {
                "normal" => Some(FontStyle::Normal),
                "italic" => Some(FontStyle::Italic),
                _ => None,
            })
            .collect();
        let style_name = |style: FontStyle| {
            if style == FontStyle::Italic {
                "italic"
            } else {
                "normal"
            }
        };
        let axis = if detail.variable {
            self.weight_axis(id).await?
        } else {
            None
        };
        let wanted: Vec<(String, FontStyle, String, String)> = match axis {
            Some((min, max)) => styles
                .iter()
                .map(|&style| {
                    let s = style_name(style);
                    let url = format!(
                        "{}/fontsource/fonts/{id}:vf@latest/latin-wght-{s}.woff2",
                        self.cdn
                    );
                    (
                        format!("{min} {max}"),
                        style,
                        format!("{id}-latin-wght-{s}.woff2"),
                        url,
                    )
                })
                .collect(),
            None => detail
                .weights
                .iter()
                .flat_map(|weight| styles.iter().map(move |&style| (weight, style)))
                .filter_map(|(weight, style)| {
                    let s = style_name(style);
                    let url = detail
                        .variants
                        .get(&weight.to_string())?
                        .get(s)?
                        .get(LATIN)?
                        .url
                        .woff2
                        .clone()?;
                    Some((
                        weight.to_string(),
                        style,
                        format!("{id}-latin-{weight}-{s}.woff2"),
                        url,
                    ))
                })
                .collect(),
        };
        let mut faces = Vec::new();
        for (weight, style, name, url) in wanted {
            let bytes = self.bytes(&url).await?;
            faces.push(NewFace {
                name,
                bytes: bytes.into(),
                weight,
                style,
                format: FontFormat::Woff2,
            });
        }
        Ok(faces)
    }

    async fn license(&self, id: &str) -> Option<FontBytes> {
        match self
            .bytes(&format!("{}/npm/@fontsource/{id}/LICENSE", self.cdn))
            .await
        {
            Ok(bytes) => Some(FontBytes {
                name: "LICENSE".into(),
                bytes,
            }),
            Err(error) => {
                log::warn!("{id} licence: {error}");
                None
            }
        }
    }

    pub async fn download(
        &self,
        library: &FontLibrary,
        db: &Database,
        id: &str,
    ) -> Result<FontFamilyDef, OnlineError> {
        let detail: Detail = self
            .fetch(&format!("{}/fonts/{}", self.api, encode(id)), FIND_FAILED)
            .await?
            .json()
            .await
            .map_err(|_| OnlineError::Unreadable)?;
        let files = self.faces(id, &detail).await?;
        if files.is_empty() {
            return Err(OnlineError::NoLatinFiles);
        }
        let category = serde_json::from_value::<FontCategory>(detail.category.clone())
            .unwrap_or_else(|_| guess_category(&detail.family));
        let family = NewFamily {
            family: detail.family,
            category,
            source: FontSource::Online,
            files,
            license: self.license(id).await,
            license_type: detail.license.as_str().map(str::to_owned),
        };
        let added = library.add_families(db, vec![family]).await?;
        added.into_iter().next().ok_or(OnlineError::Unreadable)
    }
}

fn encode(id: &str) -> String {
    percent_encoding::utf8_percent_encode(id, percent_encoding::NON_ALPHANUMERIC).to_string()
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    const CLOSED: &str = "http://127.0.0.1:9";

    async fn library() -> (tempfile::TempDir, FontLibrary, Database) {
        let dir = tempfile::tempdir().unwrap();
        let library = FontLibrary::new(dir.path());
        (dir, library, Database::in_memory().await.unwrap())
    }

    #[tokio::test]
    async fn asks_to_check_the_connection_when_the_font_list_or_a_font_cant_be_reached() {
        let online = FontsOnline::with_hosts(CLOSED, CLOSED);
        assert_eq!(
            online.catalog().await.unwrap_err().to_string(),
            "Couldn't load the font list. Check your internet connection."
        );
        let (_dir, library, db) = library().await;
        assert_eq!(
            online
                .download(&library, &db, "lato")
                .await
                .unwrap_err()
                .to_string(),
            "Couldn't find that font online. Check your internet connection."
        );
    }

    #[tokio::test]
    async fn says_the_font_wasnt_found_without_blaming_the_connection() {
        let server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let online = FontsOnline::with_hosts(&server.uri(), &server.uri());
        let (_dir, library, db) = library().await;
        assert_eq!(
            online
                .download(&library, &db, "nope")
                .await
                .unwrap_err()
                .to_string(),
            "Couldn't find that font online."
        );
    }

    #[tokio::test]
    async fn lists_fonts_with_latin_letters_once_and_reads_unknown_categories_as_other() {
        let server = MockServer::start().await;
        let entry = |id: &str, category: &str, subsets: Value| serde_json::json!({ "id": id, "family": id, "category": category, "variable": false, "styles": ["normal"], "weights": [400], "license": "OFL-1.1", "type": "google", "subsets": subsets });
        let rows = serde_json::json!([entry("a", "serif", serde_json::json!(["latin"])), entry("b", "brushes", Value::Null), entry("c", "serif", serde_json::json!(["cyrillic"])), { "id": "broken" }]);
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json(rows))
            .expect(1)
            .mount(&server)
            .await;
        let online = FontsOnline::with_hosts(&server.uri(), &server.uri());
        let listed: Vec<(String, CatalogCategory)> = online
            .catalog()
            .await
            .unwrap()
            .iter()
            .map(|e| (e.id.clone(), e.category))
            .collect();
        assert_eq!(
            listed,
            [
                ("a".to_owned(), CatalogCategory::Serif),
                ("b".to_owned(), CatalogCategory::Other)
            ]
        );
        online.catalog().await.unwrap();
    }
}
