use std::{io, process};
use std::str::FromStr;
use argparse::{ArgumentParser, List, Store, StoreTrue};

use pprint::{self, Config as PpConfig, Mode as PpMode};

#[derive(Copy, Clone, Debug)]
enum CommandType {
    Solve,
}

impl CommandType {
    fn setup_parser<'parser>(&'parser mut self,
                             ap: &mut ArgumentParser<'parser>,
                             args: &'parser mut Vec<String>) {
        let _ = ap.refer(self)
                  .required()
                  .add_argument("command", Store, "command to run (solve)");
        let _ = ap.refer(args)
                  .add_argument("arguments", List, "arguments for command");
        ap.stop_on_first_argument(true);
    }
}

impl Default for CommandType {
    fn default() -> CommandType {
        CommandType::Solve
    }
}

impl FromStr for CommandType {
    type Err = ();

    fn from_str(src: &str) -> Result<CommandType, ()> {
        match src {
            "solve" => Ok(CommandType::Solve),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
struct SolveArgs {
    derive_all: bool,
    output_mode: OutputModeArg,
    width: Size,
    height: Size,
    input_files: Vec<String>,
}

impl SolveArgs {
    fn setup_parser<'parser>(&'parser mut self, ap: &mut ArgumentParser<'parser>) {
        let _ = ap.refer(&mut self.derive_all)
                  .add_option(&["--all"], StoreTrue, "derive all solutions (if any).");
        let _ = ap.refer(&mut self.output_mode)
                  .add_option(&["--output-mode"],
                              Store,
                              "specify output mode (auto, pretty-color, pretty-ascii, raw, none) \
                               [default: auto]");
        let _ = ap.refer(&mut self.width)
                  .add_option(&["--width"], Store, "specify cell width [default: 2]");
        let _ = ap.refer(&mut self.height)
                  .add_option(&["--height"], Store, "specify cell width [default: 1]");
        let _ = ap.refer(&mut self.input_files)
                  .add_argument("input_files", List, "puzzle files to solve.");
    }

    fn output_mode(&self) -> OutputMode {
        let ppmode = match self.output_mode {
            OutputModeArg::Auto => {
                if pprint::is_pprintable() {
                    PpMode::Color
                } else {
                    PpMode::Ascii
                }
            }
            OutputModeArg::PrettyColor => PpMode::Color,
            OutputModeArg::PrettyAscii => PpMode::Ascii,
            OutputModeArg::Raw => return OutputMode::Raw,
            OutputModeArg::None => return OutputMode::None,
        };
        OutputMode::Pretty(PpConfig {
            mode: ppmode,
            cell_width: self.width.0,
            cell_height: self.height.0,
        })
    }
}

impl Default for SolveArgs {
    fn default() -> SolveArgs {
        SolveArgs {
            derive_all: false,
            output_mode: OutputModeArg::Auto,
            width: Size(2),
            height: Size(1),
            input_files: vec![],
        }
    }
}

impl Into<SolveConfig> for SolveArgs {
    fn into(self) -> SolveConfig {
        SolveConfig {
            derive_all: self.derive_all,
            output_mode: self.output_mode(),
            input_files: self.input_files,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Size(usize);
impl FromStr for Size {
    type Err = ();

    fn from_str(src: &str) -> Result<Size, ()> {
        let val = usize::from_str(src);
        if let Ok(v) = val {
            if v != 0 {
                return Ok(Size(v));
            }
        }
        Err(())

    }
}

#[derive(Copy, Clone, Debug)]
enum OutputModeArg {
    Auto,
    PrettyColor,
    PrettyAscii,
    Raw,
    None,
}

impl FromStr for OutputModeArg {
    type Err = ();

    fn from_str(src: &str) -> Result<OutputModeArg, ()> {
        match src {
            "auto" => Ok(OutputModeArg::Auto),
            "pretty-color" => Ok(OutputModeArg::PrettyColor),
            "pretty-ascii" => Ok(OutputModeArg::PrettyAscii),
            "raw" => Ok(OutputModeArg::Raw),
            "none" => Ok(OutputModeArg::None),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Config {
    Solve(SolveConfig),
}

#[derive(Clone, Debug)]
pub struct SolveConfig {
    pub derive_all: bool,
    pub output_mode: OutputMode,
    pub input_files: Vec<String>,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputMode {
    Pretty(PpConfig),
    Raw,
    None,
}

impl Config {
    pub fn parse() -> Config {
        let mut command = CommandType::default();
        let mut args = vec![];
        {
            let mut ap = ArgumentParser::new();
            command.setup_parser(&mut ap, &mut args);
            ap.parse_args_or_exit();
        }

        args.insert(0, format!("{:?}", command));

        match command {
            CommandType::Solve => {
                let mut sub_args = SolveArgs::default();
                {
                    let mut ap = ArgumentParser::new();
                    sub_args.setup_parser(&mut ap);
                    if let Err(x) = ap.parse(args, &mut io::stdout(), &mut io::stderr()) {
                        process::exit(x);
                    }
                }
                Config::Solve(sub_args.into())
            }
        }
    }
}
