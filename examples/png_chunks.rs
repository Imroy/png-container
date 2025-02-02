/*
  png-container
  Copyright (C) 2023 Ian Tester

  This program is free software: you can redistribute it and/or modify
  it under the terms of the GNU General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  This program is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
  GNU General Public License for more details.

  You should have received a copy of the GNU General Public License
  along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::env;
use std::fs::File;
use std::io::BufReader;

use uom::{
    fmt::DisplayStyle::Abbreviation,
    si::{
        linear_number_density::per_inch,
        time::second,
    },
};

use png_container::chunks::*;
use png_container::reader::*;
use png_container::types::*;

fn print_chunk(cd: &PNGChunkData) {
    println!("data={:?}", &cd);
    match cd {
        PNGChunkData::CHRM { .. } => {
            println!("white_coords={:?}", cd.chrm_white_coords());
            println!("  red_coords={:?}", cd.chrm_red_coords());
            println!("green_coords={:?}", cd.chrm_green_coords());
            println!(" blue_coords={:?}", cd.chrm_blue_coords());
        }

        PNGChunkData::GAMA { .. } => {
            if let Some(gamma) = cd.gama_gamma() {
                println!("gamma={}", gamma);
            }
        }

        PNGChunkData::ICCP { .. } => {
            println!("profile={:?}", cd.iccp_profile());
        }

        PNGChunkData::ZTXT { .. } => {
            if let Some(string) = cd.ztxt_string() {
                println!("string=\"{}\"", string);
            }
        }

        PNGChunkData::ITXT { .. } => {
            if let Some(string) = cd.itxt_string() {
                println!("string=\"{}\"", string);
            }
        }

        PNGChunkData::PHYS { .. } => {
            if let Some((xres, yres)) = cd.phys_res() {
                println!(
                    "pixels per inch={} × {}",
                    xres.into_format_args(per_inch, Abbreviation),
                    yres.into_format_args(per_inch, Abbreviation)
                );
            }
        }

        PNGChunkData::TIME { .. } => {
            if let Some(time) = cd.time() {
                println!("time={}", time);
            }
        }

        PNGChunkData::FCTL { .. } => {
            if let Some(delay) = cd.fctl_delay() {
                println!("delay={}", delay.into_format_args(second, Abbreviation));
            }
        }

        _ => (),
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Ok(());
    }

    let f = File::open(&args[1])?;
    let bf = BufReader::new(f);

    let mut reader = PNGSeekableReader::from_stream(bf)?;
    let header_chunks = reader.scan_header_chunks()?;
    println!(
        "filetype={:?}, width={}, height={}, bit_depth={}, colour_type={:?}",
        reader.filetype, reader.width, reader.height, reader.bit_depth, reader.colour_type
    );

    println!("{} header chunks.", header_chunks.len());
    if reader.plte.is_some() {
        println!("\tPLTE chunk");
    } else {
        println!("\tNo PLTE chunk");
    }

    for c in &header_chunks {
        println!("type_str={}, ref={:?}", c.type_str(), c);

        if let Ok(cd) = reader.read_chunk(c) {
            print_chunk(&cd);
        }
        println!("");
    }

    if reader.filetype == PNGFileType::APNG {
        reader.reset_next_chunk_position();
        for f in reader.apng_scan_frames()? {
            println!("frame");
            println!("\t{:?}", reader.read_chunk(&f.fctl)?);
            let fdats = f
                .fdats
                .iter()
                .map(|cr| reader.read_chunk(cr).unwrap())
                .collect::<Vec<_>>();
            println!("\tdata:");
            for fd in fdats {
                println!("\t\t{:?}", fd);
            }
        }
    } else {
        for c in reader.scan_all_chunks()? {
            println!("type_str={}, ref={:?}", c.type_str(), c);

            if let Ok(cd) = reader.read_chunk(&c) {
                print_chunk(&cd);
            }
            println!("");
        }
    }

    Ok(())
}
