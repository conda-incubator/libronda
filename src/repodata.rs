use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::fmt;

use serde::de;
use serde::Deserialize;

use crate::Version;

#[derive(Deserialize, Debug)]
struct Record<'a> {
    build: &'a str,
    build_number: u16,
    depends: Vec<&'a str>,
    md5: &'a str,
    name: &'a str,
    sha256: &'a str,
    size: u64,
    timestamp: u64,
    #[serde(deserialize_with="deserialize_json_str_to_version")]
    version: Version<'a>,
}

fn deserialize_json_str_to_version<'de: 'a, 'a, D>(deserializer: D) -> Result<Version<'a>, D::Error>
    where
        D: de::Deserializer<'de>,
{
    // define a visitor that deserializes
    // `ActualData` encoded as json within a string
    struct JsonStringVisitor;

    impl<'de> de::Visitor<'de> for JsonStringVisitor {
        type Value = Version<'de>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing json data")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
        {
            // unfortunately we lose some typed information
            // from errors deserializing the json string
            Version::from(v).map_err(E::custom)
        }
    }

    // use our visitor to deserialize an `Version`
    deserializer.deserialize_any(JsonStringVisitor)
}

#[derive(Deserialize, Debug)]
struct RepodataInfo<'a> {
    subdir: &'a str
}

#[derive(Deserialize, Debug)]
pub struct Repodata<'a> {
    info: RepodataInfo<'a>,
    packages: HashMap<&'a str, Record<'a>>,
    #[serde(rename = "packages.conda")]
    packages_conda: HashMap<&'a str, Record<'a>>,
    repodata_version: u8,
    removed: Vec<&'a str>,
}

pub fn read_repodata<'a, P: AsRef<Path>>(path: P) -> Result<Repodata<'a>, serde_json::error::Error> {
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
