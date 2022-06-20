use gpx::{errors::GpxError, read, Gpx};
use log::{debug, LevelFilter};
use minigps::poi::{write_pois, POI};
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    #[structopt(
        parse(from_os_str),
        long,
        required_if("export_poi", "true"),
        help = "GPX filename"
    )]
    gpx: Option<PathBuf>,

    #[structopt(
        parse(from_os_str),
        long,
        required_if("export_poi", "true"),
        help = "DAT filename"
    )]
    dat: Option<PathBuf>,

    #[structopt(short, long)]
    export_poi: bool,

    #[structopt(long)]
    debug: bool,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
enum ConvError {
    #[error("IO failed")]
    IOError(#[from] std::io::Error),
    #[error("Argument parsing error")]
    ArgParseError(#[from] structopt::clap::Error),
    #[error("GPX conversion error")]
    GpxError(#[from] GpxError),
    #[error("Arguments error: {0}")]
    ArgsError(String),
}

fn gpx_to_poi(gpx: PathBuf, dat: PathBuf) -> Result<(), ConvError> {
    let gpxfile = File::open(gpx)?;
    let reader = BufReader::new(gpxfile);

    // read takes any io::Read and gives a Result<Gpx, Error>.
    let points: Gpx = read(reader)?;
    debug!("{:?}", points.waypoints);
    let pois: Vec<POI> = points
        .waypoints
        .into_iter()
        .map(|e| -> POI { e.into() })
        .collect();
    debug!("{:?}", pois);

    let datfile = File::create(dat)?;
    let mut writer = BufWriter::new(datfile);
    write_pois(pois, &mut writer)?;
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), ConvError> {
    let args = Cli::from_args();
    let mut logbuilder = env_logger::Builder::from_default_env();

    logbuilder
        .filter(None, {
            if args.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            }
        })
        .init();

    if args.export_poi {
        if args.gpx.is_none() || args.dat.is_none() {
            return Err(ConvError::ArgsError("No filenames provided".to_string()));
        }
        gpx_to_poi(args.gpx.unwrap(), args.dat.unwrap())?;
    }
    Ok(())
}
