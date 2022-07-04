/*
 * Copyright 2021 Constantin A.
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 *
 *
 *   OR
 *
 *
 *   Copyright 2021 Constantin A.
 *
 *   Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
 *
 *   The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
 *
 *   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

#![feature(once_cell)]

use std::path::{PathBuf, Path};
use cargo_metadata::camino::Utf8PathBuf;
use walkdir::WalkDir;
use std::sync::OnceLock;
use regex::Regex;
use cargo_metadata::Package;
use std::fs::create_dir_all;
use std::io::{ErrorKind, BufReader, BufRead};

/// Returns the source path for a package
pub fn path(package: cargo_metadata::Package) -> Option<Utf8PathBuf> {
    package.manifest_path.parent().map(|path| path.to_path_buf())
}

/// Used to identify files (most likely) containing a license
static LICENSE_REGEX: OnceLock<Regex> = OnceLock::new();
/// Used to select files to check the contents of for licenses
static CHECK_CONTENT_REGEX: OnceLock<Regex> = OnceLock::new();
/// Used to determine whether a file contains licensing information
static LICENSE_CONTENT_REGEX: OnceLock<Regex> = OnceLock::new();


/// Tries to find all license-related files for a given package
/// ## Returns
/// - The local root path of the crate (obtained by [`path`])
/// - An iterator over all potential license-related files
pub fn search_for_licenses(package: &Package) -> Option<(PathBuf, Box<dyn Iterator<Item=PathBuf>>)> {
    let license_regex = LICENSE_REGEX.get_or_init(|| {
        Regex::new(r"(?i)LICENSE|COPYRIGHT|NOTICE|AUTHORS|CONTRIBUTORS|COPYING|PATENT").unwrap()
    });

    let license_content_regex = LICENSE_CONTENT_REGEX.get_or_init(|| {
        // Some items are missing here because they tend to produce false-positives
        Regex::new(r"(?i)LICENSE|COPYRIGHT|COPYING|PATENT").unwrap()
    });

    let check_content_regex = CHECK_CONTENT_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\.html|\.txt|\.md|README").unwrap()
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
                .filter(move |entry| {
                    let ft = entry.file_type();
                    if !(ft.is_file() || ft.is_symlink()) {
                        return false;
                    };
                    license_regex.is_match(&entry.path().to_string_lossy()) ||
                    (
                        check_content_regex.is_match(&entry.path().to_string_lossy()) &&
                        matches_any_line(&license_content_regex, entry.path()).unwrap_or_default()
                    )
                })
                .map(|entry| entry.path().to_path_buf()))
        }
    ))
}


/// Checks whether a regular expression matches any (part of a) line in a given file
/// ## Returns
/// - [`core::Option::None`] if the file could not be opened
/// - [`core::Option::Some`] else
fn matches_any_line(regex: &Regex, file: &Path) -> Option<bool> {
    let file = std::fs::File::open(file).ok()?;
    let reader = BufReader::new(file);
    Some(reader.lines()
        .filter_map(|line| line.ok())
        .any(|line| regex.is_match(&line)))
}

/// Helper function to [std::path::Path::strip_prefix] for an iterator over [std::path::PathBuf]s
fn make_paths_relative<'a>(root_path: &'a Path, paths: Box<dyn Iterator<Item=PathBuf> + 'a>) -> Box<dyn Iterator<Item=PathBuf> + 'a> {
    // https://stackoverflow.com/a/39343127
    Box::new(paths.map(move |path| path.strip_prefix(root_path).unwrap().to_path_buf()))
}

/// Same as [make_paths_relative], only that the returned iterator still contains the full path
fn with_relative_paths<'a>(root_path: PathBuf, paths: Box<dyn Iterator<Item=PathBuf> + 'a>) -> Box<dyn Iterator<Item=(PathBuf, PathBuf)> + 'a> {
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

/// Copies all the licenses to a common directory, preserving their directory hierarchy
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


// It is a quick and dirty solution, but as it would only be printed anyway, it shouldn't be a major problem
/// Helper function to convert from the result of [copy_dir::copy_dir] into a more usable format with
/// only success or one error (which then contains the list of errors in a string).
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