#![feature(once_cell)]

use std::path::{PathBuf, Path};
use cargo_metadata::camino::Utf8PathBuf;
use walkdir::WalkDir;
use std::lazy::SyncOnceCell;
use regex::Regex;
use cargo_metadata::Package;
use std::fs::create_dir_all;
use std::io::{ErrorKind, BufReader, BufRead};

pub fn path(package: cargo_metadata::Package) -> Option<Utf8PathBuf> {
    package.manifest_path.parent().map(|path| path.to_path_buf())
}

static LICENSE_REGEX: SyncOnceCell<Regex> = SyncOnceCell::new();
static CHECK_CONTENT_REGEX: SyncOnceCell<Regex> = SyncOnceCell::new();


pub fn search_for_licenses(package: &Package) -> Option<(PathBuf, Box<dyn Iterator<Item=PathBuf>>)> {
    let license_regex = LICENSE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)LICENSE|COPYRIGHT|NOTICE|AUTHORS|COPYING").unwrap()
    });

    let check_content_regex = CHECK_CONTENT_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\.txt|\.md|README").unwrap()
    });

    let path = package.manifest_path.parent().map(|path| path.to_path_buf())?;

    Some((path.canonicalize().unwrap(),
        if let Some(license_file) = &package.license_file() {
            // If a license is explicitly given, only return it
            Box::new(vec!(license_file.canonicalize().unwrap()).into_iter())
        } else {
            Box::new(WalkDir::new(path)
                .into_iter()
                .filter_map(|entry| entry.ok())
                .filter(move |entry|
                    license_regex.is_match(&entry.path().to_string_lossy()) ||
                    (
                        check_content_regex.is_match(&entry.path().to_string_lossy()) &&
                        matches_any_line(&license_regex, entry.path()).unwrap_or_default()
                    )
                )
                .map(|entry| entry.path().to_path_buf()))
        }
    ))
}


fn matches_any_line(regex: &Regex, file: &Path) -> Option<bool> {
    let file = std::fs::File::open(file).ok()?;
    let reader = BufReader::new(file);
    Some(reader.lines()
        .filter_map(|line| line.ok())
        .any(|line| regex.is_match(&line)))
}

pub fn make_paths_relative<'a>(root_path: &'a Path, paths: Box<dyn Iterator<Item=PathBuf> + 'a>) -> Box<dyn Iterator<Item=PathBuf> + 'a> {
    // https://stackoverflow.com/a/39343127
    Box::new(paths.map(move |path| path.strip_prefix(root_path).unwrap().to_path_buf()))
}

pub fn with_relative_paths<'a>(root_path: PathBuf, paths: Box<dyn Iterator<Item=PathBuf> + 'a>) -> Box<dyn Iterator<Item=(PathBuf, PathBuf)> + 'a> {
    // https://stackoverflow.com/a/39343127
    Box::new(paths.map(move |path| {
        let stripped_path = (&path.canonicalize().unwrap()).strip_prefix(&root_path.canonicalize().unwrap()).map_err(|err| format!("Couldn't remove the prefix {:?} from {:?}: {}", &root_path, &path, err)).unwrap().to_path_buf();
        (path, stripped_path)
    }))
}

/// Includes (name, root path, license paths)
pub type Licenses = Box<dyn Iterator<Item=(String, PathBuf, Box<dyn Iterator<Item=PathBuf>>)>>;

/// Returns the root path for the crate and the list of (probable) paths for licenses
pub fn search_for_all_licenses(metadata: cargo_metadata::Metadata) -> Licenses {
    let packages = metadata.packages;
    let workspace_members = metadata.workspace_members;
    let external_packages: Vec<Package> = packages.into_iter()
        .filter(|package| !(&workspace_members).contains(&package.id))
        .collect();
    Box::new(external_packages.into_iter()
        .filter_map(|package| Some(package.name.clone()).zip(search_for_licenses(&package)))
        .map(|(name, (root_path, license_paths))| (name, root_path, license_paths))
        //.map(|(root_path, license_paths)| (root_path, make_paths_relative(&root_path, license_paths)))
        //.map(|(root_path, license_paths)| (root_path, license_paths.collect()))
        //.collect()
    )
}

pub fn copy_licenses_to(destination: &Path, licenses: Licenses) -> std::io::Result<Vec<std::io::Result<()>>> {
    create_dir_all(destination)?;

    Ok(
        licenses
            .map(move |(name, root_path, license_paths)| {
                // Remove the root directory from the license locations (while preserving a version with it as the source)
                (name, with_relative_paths(root_path, license_paths))
            })
            .map(|(name, license_pairs)|
                // And from that build the path where we want to copy the file to
                license_pairs.map(move |(src, stem)| (src, destination.join(name.clone()).join(stem)))
            )
            .flatten()
            // Create the parent directory if necessary and copy the file/directory
            .map(|(src, dst)| dst.parent().map(create_dir_all).unwrap_or(Ok(()))
                .and(flatten_copy_dir_result(copy_dir::copy_dir(src, dst))))
            .collect()
    )
}

#[inline]
fn flatten_copy_dir_result(result: Result<Vec<std::io::Error>, std::io::Error>) -> Result<(), std::io::Error> {
    match result {
        Ok(multiple) => if multiple.is_empty() {
            Ok(())
        } else {
            Err(std::io::Error::new(ErrorKind::Other, format!("{:?}", multiple)))
        },
        Err(err) => Err(err)
    }
}