use std::collections::HashMap;
use std::path::Path;

use serde::de;
use serde::Deserialize;

use crate::{Version, conda_parser};

#[derive(Deserialize, Debug)]
pub struct Record {
    pub build: String,
    pub build_number: u16,
    pub depends: Vec<String>,
    pub md5: String,
    pub name: String,
    pub sha256: String,
    pub size: u64,
    pub timestamp: u64,
    #[serde(deserialize_with="deserialize_json_str_to_version")]
    pub version: Version,
}

fn deserialize_json_str_to_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match Version::parse(s, &conda_parser) {
        Ok(v) => Ok(v),
        Err(_e) => Err(de::Error::custom("Version parsing error"))
    }
}

#[derive(Deserialize, Debug)]
pub struct RepodataInfo {
    pub subdir: String
}

#[derive(Deserialize, Debug)]
pub struct Repodata {
    pub info: RepodataInfo,
    pub packages: HashMap<String, Record>,
    #[serde(rename = "packages.conda")]
    pub packages_conda: HashMap<String, Record>,
    pub repodata_version: u8,
    pub removed: Vec<String>,
}

pub fn read_repodata<'a, P: AsRef<Path>>(path: P) -> Result<Repodata, serde_json::error::Error> {
    let file = std::fs::read_to_string(path).unwrap();
    // Read the JSON contents of the file as an instance of `Repodata`.
    let r = serde_json::from_str(&file)?;

    // Return the `Repodata`.
    Ok(r)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;

    #[test]
    fn test_load_repodata() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("tests/data/current_repodata.json");
        println!("{}", d.display());
        let _u: Repodata = read_repodata(d).unwrap();
        assert_eq!(_u.info.subdir, "win-64");

    }
}
