use docopt::Docopt;
use rustc_serialize::{Decodable, Decoder};

use ::pprint::{self, Config as PpConfig, Mode as PpMode};

const USAGE: &'static str = "
Usage: slither [options]
       slither --help

Options:
  -h, --help       Show this message.
  --pretty MODE    Specify pretty-print mode.
                   Valid values: auto, color, ascii, none [default: auto]
  --width WIDTH    Specify cell width [default: 2].
  --height HEIGHT  Specify cell height [default: 1].
";

#[derive(Copy, Clone, Debug, RustcDecodable)]
struct Args {
    flag_pretty: Option<Pretty>,
    flag_width: Option<Width>,
    flag_height: Option<Height>
}

#[derive(Copy, Clone, Debug)]
struct Width(usize);
impl Decodable for Width {
    fn decode<D: Decoder>(d: &mut D) -> Result<Width, D::Error> {
        let w = try!(d.read_usize());
        if w == 0 {
            Err(d.error(&format!("Could not decode '{}' as width.", w)))
        } else {
            Ok(Width(w))
        }
    }
}
impl Default for Width {
    fn default() -> Width { Width(2) }
}

#[derive(Copy, Clone, Debug)]
struct Height(usize);
impl Decodable for Height {
    fn decode<D: Decoder>(d: &mut D) -> Result<Height, D::Error> {
        let h = try!(d.read_usize());
        if h == 0 {
            Err(d.error(&format!("Could not decode '{}' as height.", h)))
        } else {
            Ok(Height(h))
        }
    }
}
impl Default for Height {
    fn default() -> Height { Height(1) }
}

#[derive(Copy, Clone, Debug, RustcDecodable)]
enum Pretty {
    Auto, Color, Ascii, None
}
impl Default for Pretty {
    fn default() -> Pretty { Pretty::Auto }
}

#[derive(Copy, Clone, Debug)]
pub struct Config {
    pub output_type: OutputType
}

#[derive(Copy, Clone, Debug)]
pub enum OutputType {
    Pretty(PpConfig),
    Raw
}

impl Config {
    pub fn parse() -> Config {
        let args: Args = Docopt::new(USAGE)
            .and_then(|d| d.decode())
            .unwrap_or_else(|e| e.exit());

        let pretty = match args.flag_pretty.unwrap_or_default() {
            Pretty::Auto => {
                if pprint::is_pprintable() {
                    Some(PpMode::Color)
                } else {
                    Some(PpMode::Ascii)
                }
            }
            Pretty::Color => Some(PpMode::Color),
            Pretty::Ascii => Some(PpMode::Ascii),
            Pretty::None => None
        };

        let output_type = match pretty {
            Some(m) => OutputType::Pretty(PpConfig {
                mode: m,
                cell_width: args.flag_width.unwrap_or_default().0,
                cell_height: args.flag_height.unwrap_or_default().0
            }),
            None => OutputType::Raw
        };

        Config {
            output_type: output_type
        }
    }
}
