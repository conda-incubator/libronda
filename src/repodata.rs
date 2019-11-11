use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_derive::Deserialize;
use serde_json::Result;

use crate::Version;

#[derive(Deserialize, Debug)]
struct Record<'a> {
    build: String,
    build_number: u16,
    depends: Vec<String>,
    md5: String,
    name: String,
    sha256: String,
    size: u64,
    timestamp: u64,
    #[serde(deserialize_with="Version::from")]
    version: Version<'a>,
}

#[derive(Deserialize, Debug)]
struct RepodataInfo {
    subdir: String
}

#[derive(Deserialize, Debug)]
pub struct Repodata<'a> {
    info: RepodataInfo,
    packages: HashMap<String, Record<'a>>,
    #[serde(rename = "packages.conda")]
    packages_conda: HashMap<String, Record<'a>>,
    repodata_version: u8,
    removed: Vec<String>,
}

pub fn read_repodata<'a, P: AsRef<Path>>(path: P) -> Result<Repodata<'a>> {
    // Open the file in read-only mode with buffer.
    let f = File::open(path);
    let f = match f {
        Ok(file) => file,
        Err(error) => {
            panic!("Problem opening the file: {:?}", error)
        },
    };
    let reader = BufReader::new(f);

    // Read the JSON contents of the file as an instance of `Repodata`.
    let r = serde_json::from_reader(reader)?;

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
