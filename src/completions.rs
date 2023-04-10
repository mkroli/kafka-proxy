/*
 * Copyright 2023 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::io::stdout;
use clap::{CommandFactory, Parser, ColorChoice};
use clap_complete::Shell;

#[allow(unused)]
mod cli;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Generate shell completions",
    long_about = None,
    propagate_version = true,
    color = ColorChoice::Auto
)]
struct Cli {
    #[arg()]
    pub shell: Shell,
}

fn main() {
    let cli = Cli::parse();
    let mut command = cli::Cli::command();
    let cmd = &mut command;
    clap_complete::generate(cli.shell, cmd, cmd.get_name().to_string(), &mut stdout());
}
