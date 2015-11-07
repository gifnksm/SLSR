use std::str::FromStr;
use argparse::{ArgumentParser, Store, StoreTrue};

use pprint::{self, Config as PpConfig, Mode as PpMode};

#[derive(Copy, Clone, Debug)]
struct Args {
    derive_all: bool,
    output_mode: OutputModeArg,
    width: Size,
    height: Size,
}

impl Args {
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
    }
}

impl Default for Args {
    fn default() -> Args {
        Args {
            derive_all: false,
            output_mode: OutputModeArg::Auto,
            width: Size(2),
            height: Size(1),
        }
    }
}

impl Into<Config> for Args {
    fn into(self) -> Config {
        Config {
            derive_all: self.derive_all,
            output_mode: self.output_mode(),
        }
    }
}

impl Args {
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

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub derive_all: bool,
    pub output_mode: OutputMode,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputMode {
    Pretty(PpConfig),
    Raw,
    None,
}

impl Config {
    pub fn parse() -> Config {
        let mut args = Args::default();
        {
            let mut ap = ArgumentParser::new();
            args.setup_parser(&mut ap);
            ap.parse_args_or_exit();
        }
        args.into()
    }
}
