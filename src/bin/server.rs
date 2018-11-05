// main.rs - Mix server main function.
// Copyright (C) 2018  David Anthony Stainton.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.


extern crate clap;
extern crate mix_server;

use clap::{Arg, App};
//use mix_server::server::Server;

fn main() {
    let matches = App::new("mixnet server")
        .version("0.0.0")
        .author("David Stainton <dawuud@riseup.net>")
        .about("Mix server is Sphinx based cryptographic router for composing traffic analysis resistant communication networks.")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config_file")
             .required(true)
             .value_name("FILE")
             .help("Specifies the configuration file.")
             .takes_value(true))
        .get_matches();
    let _config_file_path = matches.value_of("config").unwrap();
}
