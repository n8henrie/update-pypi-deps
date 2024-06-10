use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PypiResp {
    pub info: Info,
    // pub last_serial: i64,
    // pub releases: HashMap<String, Vec<Release>>,
    // pub urls: Vec<Release>,
    // pub vulnerabilities: Vec<Value>,
}

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// pub struct Release {
//     pub comment_text: String,
//     pub digests: Digests,
//     pub downloads: i64,
//     pub filename: String,
//     pub has_sig: bool,
//     pub md5_digest: String,
//     pub packagetype: String,
//     pub python_version: String,
//     pub requires_python: Value,
//     pub size: i64,
//     pub upload_time: String,
//     pub upload_time_iso_8601: String,
//     pub url: Url,
//     pub yanked: bool,
//     pub yanked_reason: Value,
// }

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// pub struct Digests {
//     pub blake2b_256: String,
//     pub md5: String,
//     pub sha256: String,
// }

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Info {
    // pub author: String,
    // pub author_email: String,
    // pub bugtrack_url: Value,
    // pub classifiers: Vec<String>,
    // pub description: String,
    // pub description_content_type: String,
    // pub docs_url: Value,
    // pub download_url: String,
    // pub downloads: Downloads,
    // pub dynamic: Value,
    // pub home_page: String,
    // pub keywords: String,
    // pub license: String,
    // pub maintainer: String,
    // pub maintainer_email: String,
    // pub name: String,
    // pub package_url: String,
    // pub platform: Value,
    // pub project_url: String,
    // pub project_urls: ProjectUrls,
    // pub provides_extra: Value,
    // pub release_url: String,
    // pub requires_dist: Vec<String>,
    // pub requires_python: String,
    // pub summary: String,
    pub version: String,
    // pub yanked: bool,
    // pub yanked_reason: Value,
}

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// pub struct Downloads {
//     pub last_day: i64,
//     pub last_month: i64,
//     pub last_week: i64,
// }

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "lowercase")]
// pub struct ProjectUrls {
//     pub homepage: String,
// }
