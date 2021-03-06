// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Command line processing for Chamber

use {Config, enchamber};
use getopts::OptGroup;

static DEFAULT_CHAMBER: &'static str = "rcr_baseline";

pub fn main() {
    use std::os;

    let args = os::args();
    let exit_code = match parse_config(args) {
        Run(config) => {
            let config = merge_config_defaults(config);
            match enchamber(config) {
                Ok(_) => 0,
                Err(_) => 1
            }
        }
        Help => {
            usage();
            0
        }
        ParseError(s) => {
            println!("error: {}", s);
            1
        }
    };

    os::set_exit_status(exit_code);
}

enum ExeMode {
    Run(Config),
    Help,
    ParseError(String)
}

fn parse_config(mut args: Vec<String>) -> ExeMode {

    use rustc::driver::config::parse_crate_types_from_list;
    use getopts::{getopts};

    let _ = args.remove(0).unwrap();

    if args.is_empty() { return Help }

    let matches = match getopts(args.as_slice(), optgroups().as_slice()) {
        Ok(m) => m,
        Err(f) => return ParseError(f.to_string())
    };

    if matches.opt_present("h") || matches.opt_present("help") { return Help }

    let crate_types = match parse_crate_types_from_list(matches.opt_strs("crate-type")) {
        Ok(c) => c,
        Err(s) => return ParseError(s)
    };
    let search_paths = matches.opt_strs("L").iter().map(|s| Path::new(s.as_slice())).collect();

    let out_dir = matches.opt_str("out-dir").map(|o| Path::new(o));
    let out_file = matches.opt_str("o").map(|o| Path::new(o));

    let sysroot = matches.opt_str("sysroot").map(|o| Path::new(o));

    let chamber_name = matches.opt_str("chamber").unwrap_or(DEFAULT_CHAMBER.to_string());

    let input_file = match matches.free.len() {
        0 => return Help,
        1 => Path::new(matches.free[0].as_slice()),
        _ => return Help,
    };

    Run(Config {
        input_file: input_file,
        chamber_name: chamber_name,
        crate_types: crate_types,
        search_paths: search_paths,
        out_dir: out_dir,
        out_file: out_file,
        sysroot: sysroot
    })
}

fn merge_config_defaults(mut config: Config) -> Config {

    // Add some conveniences so cargo'd chambers can be found.
    config.search_paths.push_all([Path::new("."),
                                  Path::new("./target"),
                                  Path::new("./target/deps")]);

    // Hope that rustc is installed to the usual place
    if config.sysroot.is_none() {
        config.sysroot = Some(Path::new("/usr/local"));
    }

    config
}

fn optgroups() -> Vec<OptGroup> {

    use getopts::{optflag, optopt, optmulti};

    vec![optflag("h", "help", "Display this message"),
         optopt("", "chamber",
                "The name of the crate link to as `std`",
                "CHAMBER"),
         optmulti("L", "", "Add a directory to the library search path", "PATH"),
         optmulti("", "crate-type", "Comma separated list of types of crates
                                    for the compiler to emit",
                               "[bin|lib|rlib|dylib|staticlib]"),
         optopt("o", "", "Write output to <filename>", "FILENAME"),
         optopt( "",  "out-dir", "Write output to compiler-chosen filename in <dir>", "DIR"),
         optopt("", "sysroot", "Override the system root", "PATH"),
         ]
}

fn usage() {
    use getopts;

    println!("{}", getopts::usage("Usage: chamber [OPTIONS] INPUT",
                                  optgroups().as_slice()));
}
