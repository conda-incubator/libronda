use std::collections::HashMap;
use std::path::Path;

use serde::de;
use serde::Deserialize;

use crate::{Version, conda_parser};

#[derive(Deserialize, Debug)]
struct Record {
    build: String,
    build_number: u16,
    depends: Vec<String>,
    md5: String,
    name: String,
    sha256: String,
    size: u64,
    timestamp: u64,
    #[serde(deserialize_with="deserialize_json_str_to_version")]
    version: Version,
}

fn deserialize_json_str_to_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match Version::parse(s, &conda_parser) {
        Ok(v) => Ok(v),
        Err(e) => Err(de::Error::custom("Version parsing error"))
    }
}

#[derive(Deserialize, Debug)]
struct RepodataInfo {
    subdir: String
}

#[derive(Deserialize, Debug)]
pub struct Repodata {
    info: RepodataInfo,
    packages: HashMap<String, Record>,
    #[serde(rename = "packages.conda")]
    packages_conda: HashMap<String, Record>,
    repodata_version: u8,
    removed: Vec<String>,
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
