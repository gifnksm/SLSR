use std::str::FromStr;
use argparse::{ArgumentParser, Store, StoreTrue};

use pprint::{self, Config as PpConfig, Mode as PpMode};

#[derive(Copy, Clone, Debug)]
struct Args {
    all: bool,
    pretty: Pretty,
    width: Size,
    height: Size,
}

impl Args {
    fn setup_parser<'parser>(&'parser mut self, ap: &mut ArgumentParser<'parser>) {
        let _ = ap.refer(&mut self.all)
                  .add_option(&["--all"], StoreTrue, "show all solutions (if any).");
        let _ = ap.refer(&mut self.pretty).add_option(&["--pretty"],
                                                      Store,
                                                      "specify pretty-print mode (auto, color, \
                                                       ascii, none) [default: auto]");
        let _ = ap.refer(&mut self.width)
                  .add_option(&["--width"], Store, "specify cell width [default: 2]");
        let _ = ap.refer(&mut self.height)
                  .add_option(&["--height"], Store, "specify cell width [default: 1]");
    }
}

impl Default for Args {
    fn default() -> Args {
        Args {
            all: false,
            pretty: Pretty::Auto,
            width: Size(2),
            height: Size(1),
        }
    }
}

impl Into<Config> for Args {
    fn into(self) -> Config {
        let pretty = match self.pretty {
            Pretty::Auto => {
                if pprint::is_pprintable() {
                    Some(PpMode::Color)
                } else {
                    Some(PpMode::Ascii)
                }
            }
            Pretty::Color => Some(PpMode::Color),
            Pretty::Ascii => Some(PpMode::Ascii),
            Pretty::None => None,
        };

        let output_type = match pretty {
            Some(m) => OutputType::Pretty(PpConfig {
                mode: m,
                cell_width: self.width.0,
                cell_height: self.height.0,
            }),
            None => OutputType::Raw,
        };

        Config {
            show_all: self.all,
            output_type: output_type,
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
enum Pretty {
    Auto,
    Color,
    Ascii,
    None,
}
impl FromStr for Pretty {
    type Err = ();

    fn from_str(src: &str) -> Result<Pretty, ()> {
        match src {
            "auto" => Ok(Pretty::Auto),
            "color" => Ok(Pretty::Color),
            "ascii" => Ok(Pretty::Ascii),
            "none" => Ok(Pretty::None),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub show_all: bool,
    pub output_type: OutputType,
}

#[derive(Copy, Clone, Debug)]
pub enum OutputType {
    Pretty(PpConfig),
    Raw,
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
