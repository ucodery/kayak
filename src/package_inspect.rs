use anyhow::{anyhow, Result};
use csv;
use ini;
use mail_parser;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use ureq;
use url::Url;
use zip::read::read_zipfile_from_stream;

fn dist_filename(entry: &str) -> Option<&str> {
    if let Some((dir, name)) = entry.split_once('/') {
        if dir.ends_with(".dist-info") {
            Some(name)
        } else {
            None
        }
    } else {
        None
    }
}

fn is_dist_dir(entry: &str) -> bool {
    if let Some((dir, _)) = entry.split_once('/') {
        dir.ends_with(".dist-info")
    } else {
        false
    }
}

fn data_filename(entry: &str) -> Option<&str> {
    if let Some((dir, name)) = entry.split_once('/') {
        if dir.ends_with(".data") {
            Some(name)
        } else {
            None
        }
    } else {
        None
    }
}

fn is_data_dir(entry: &str) -> bool {
    if let Some((dir, _)) = entry.split_once('/') {
        dir.ends_with(".data")
    } else {
        false
    }
}

pub fn fetch(wheel_url: &str) -> Result<Package> {
    Url::parse(wheel_url)?;
    let mut record: Result<Record> = Err(anyhow!("no RECORD file found in distribution"));
    let mut metadata: Result<Metadata> = Err(anyhow!("no METADATA file found in distribution"));
    let mut entry_points: Option<EntryPoints> = None;
    let mut wheel = ureq::get(wheel_url).call()?.into_reader();
    while let Some(zipfile) = read_zipfile_from_stream(&mut wheel)? {
        if let Some(name) = dist_filename(zipfile.name()) {
            if name == "RECORD" {
                record = Record::from_file(zipfile);
            } else if name == "METADATA" {
                metadata = Metadata::from_file(zipfile);
            } else if name == "entry_points.txt" {
                entry_points = Some(EntryPoints::from_file(zipfile)?);
            };
        };
    }
    Ok(Package {
        record: record?,
        metadata: metadata?,
        entry_points,
    })
}

#[derive(Debug)]
struct RecordEntry {
    entry: String,
    algo: String,
    hash: String,
    size: usize,
}

#[derive(Debug)]
struct Record {
    entries: Vec<RecordEntry>,
}

impl Record {
    fn from_file<R: Read>(file: R) -> Result<Self> {
        // build specified by PEP-376
        let mut read = csv::ReaderBuilder::new()
            .delimiter(b',')
            .quote(b'"')
            .has_headers(false)
            .from_reader(file);
        let rec = read
            .records()
            .filter_map(|rec| {
                let r = rec.ok()?;
                if r.len() != 3 {
                    return None;
                };
                let size = r[2].parse().ok()?;
                let (algo, hash) = r[1].split_once('=')?;
                Some(RecordEntry {
                    entry: r[0].to_string(),
                    algo: algo.to_string(),
                    hash: hash.to_string(),
                    size,
                })
            })
            .collect();
        Ok(Self { entries: rec })
    }
}

// https://packaging.python.org/en/latest/specifications/core-metadata/
// TODO: incomplete; only what is required/needed is here
#[derive(Debug)]
struct Metadata {
    metadata_version: String,
    name: String,
    version: String,
}

impl Metadata {
    fn from_file<R: Read>(mut file: R) -> Result<Self> {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let mp = mail_parser::MessageParser::default()
            .parse_headers(buf.as_slice())
            .unwrap();
        let metadata_version = match mp.header("Metadata-Version").ok_or(anyhow!(
            "METADATA file missing required Metadata-Version key"
        ))? {
            mail_parser::HeaderValue::Text(mv) => mv.to_string(),
            _ => {
                return Err(anyhow!(
                    "METADATA file missing required Metadata-Version value"
                ))
            }
        };
        let name = match mp
            .header("Name")
            .ok_or(anyhow!("METADATA file missing required Name key"))?
        {
            mail_parser::HeaderValue::Text(n) => n.to_string(),
            _ => return Err(anyhow!("METADATA file missing required Name value")),
        };
        let version = match mp
            .header("Version")
            .ok_or(anyhow!("METADATA file missing required Version key"))?
        {
            mail_parser::HeaderValue::Text(v) => v.to_string(),
            _ => return Err(anyhow!("METADATA file missing required Version value")),
        };
        Ok(Metadata {
            metadata_version,
            name,
            version,
        })
    }
}

#[derive(Debug)]
struct ObjectReference {
    module: String,
    object: Option<String>,
    extras: Option<String>,
}

impl ObjectReference {
    fn from_str(raw: &str) -> Self {
        let module: String;
        let object: Option<String>;
        let extras: Option<String>;

        let modobj = if let Some((objr, ex)) = raw.rsplit_once('[') {
            extras = Some(ex.trim_end_matches(']').to_string());
            objr.trim_end().to_string()
        } else {
            extras = None;
            raw.to_string()
        };

        if let Some((objr, obj)) = modobj.split_once(':') {
            module = objr.to_string();
            object = Some(obj.to_string());
        } else {
            module = modobj;
            object = None;
        };

        Self {
            module,
            object,
            extras,
        }
    }
}

// https://packaging.python.org/en/latest/specifications/entry-points/#file-format
#[derive(Debug)]
struct EntryPoints {
    group: HashMap<String, HashMap<String, ObjectReference>>,
}

impl EntryPoints {
    fn from_file<R: Read>(mut file: R) -> Result<Self> {
        let entry_points = ini::Ini::read_from(&mut file)?;
        Ok(EntryPoints {
            group: entry_points
                .iter()
                .filter_map(|(g, names)| {
                    Some((
                        g?.to_string(),
                        names
                            .iter()
                            .map(|(n, o)| (n.to_string(), ObjectReference::from_str(o)))
                            .collect::<HashMap<String, ObjectReference>>(),
                    ))
                })
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct Package {
    metadata: Metadata,
    record: Record,
    entry_points: Option<EntryPoints>,
}

impl Package {
    /// Returns all top-level import names that this package provides
    ///
    /// this could be package roots, top-level modules, or namespace packages
    pub fn provides_packages(&self) -> HashSet<String> {
        self.record
            .entries
            .iter()
            .map(|r| r.entry.clone())
            .filter(|r| !(is_dist_dir(r) || is_data_dir(r)))
            .map(|r| {
                let top = if let Some((a, _)) = r.split_once('/') {
                    a
                } else {
                    &r
                };
                if let Some(package) = top.strip_suffix(".py") {
                    package.to_string()
                } else {
                    top.to_string()
                }
            })
            .collect()
    }

    /// Returns all scripts, entry-points, binaries this package provides
    pub fn provides_executables(&self) -> HashSet<String> {
        self.record
            .entries
            .iter()
            .filter_map(|r| {
                let (d, f) = data_filename(&r.entry)?.split_once('/')?;
                if d == "scripts" {
                    Some(f.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Return the names from the special entry_points group console_scripts
    pub fn console_scripts(&self) -> Vec<String> {
        if let Some(entry_points) = &self.entry_points {
            if let Some(scripts) = entry_points.group.get("console_scripts") {
                return scripts.keys().map(|k| k.to_string()).collect();
            };
        };
        Vec::new()
    }
}
