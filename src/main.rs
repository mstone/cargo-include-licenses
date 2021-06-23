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

use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use std::path::PathBuf;
use cargo_include_licenses::{copy_licenses_to, search_for_all_licenses};

fn main() {
    let matches = App::new("Cargo include-licenses")
        .version("0.1.0")
        .about("Finds licenses for the dependencies of your program and copies the files to a directory")
        .subcommand(SubCommand::with_name("include-licenses")
            .arg(Arg::with_name("out_dir")
                .help("The directory where to put the licenses into")
                .required(true)
                .value_name("DIR")
                .default_value("licenses")
                .validator(|out_dir| (PathBuf::from(&out_dir).is_dir() || !PathBuf::from(out_dir).exists()).then(|| ()).ok_or_else(|| String::from("Not a directory")))
            )
        )
        .get_matches();
    let out_dir = PathBuf::from(matches.subcommand().1.unwrap().value_of("out_dir").unwrap());
    let command = MetadataCommand::new();
    let metadata = command.exec().unwrap();

    copy_licenses_to(&out_dir, search_for_all_licenses(metadata)).unwrap();
}
