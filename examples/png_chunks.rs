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

use uom::{
    fmt::DisplayStyle::Abbreviation,
    si::{linear_number_density::per_inch, time::second},
};

use png_container::chunks::*;
use png_container::reader::*;
use png_container::types::*;

fn print_chunk(cd: &PngChunkData) {
    println!("data={:?}", &cd);
    match cd {
        PngChunkData::Chrm { .. } => {
            println!("white_coords={:?}", cd.chrm_white_coords());
            println!("  red_coords={:?}", cd.chrm_red_coords());
            println!("green_coords={:?}", cd.chrm_green_coords());
            println!(" blue_coords={:?}", cd.chrm_blue_coords());
        }

        PngChunkData::Gama { .. } => {
            if let Some(gamma) = cd.gama_gamma() {
                println!("gamma={}", gamma);
            }
        }

        PngChunkData::Iccp { .. } => {
            println!("profile={:?}", cd.iccp_profile());
        }

        PngChunkData::Ztxt { .. } => {
            if let Some(string) = cd.ztxt_string() {
                println!("string=\"{}\"", string);
            }
        }

        PngChunkData::Itxt { .. } => {
            if let Some(string) = cd.itxt_string() {
                println!("string=\"{}\"", string);
            }
        }

        PngChunkData::Phys { .. } => {
            if let Some((xres, yres)) = cd.phys_res() {
                println!(
                    "pixels per inch={} × {}",
                    xres.into_format_args(per_inch, Abbreviation),
                    yres.into_format_args(per_inch, Abbreviation)
                );
            }
        }

        PngChunkData::Time { .. } => {
            if let Some(time) = cd.time() {
                println!("time={}", time);
            }
        }

        PngChunkData::Fctl { .. } => {
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

    let mut reader = PngReader::from_stream(File::open(&args[1])?)?;
    let header_chunks = reader.scan_header_chunks()?;
    println!(
        "filetype={:?}, width={}, height={}, bit_depth={}, colour_type={:?}",
        reader.filetype, reader.width, reader.height, reader.bit_depth, reader.colour_type
    );

    println!("{} header chunks.", header_chunks.len());

    for c in &header_chunks {
        println!("type_str={}, ref={:?}", c.type_str(), c);

        if let Ok(cd) = reader.read_chunk(c) {
            print_chunk(&cd);
        }
        println!("");
    }

    if reader.filetype == PngFileType::Apng {
        reader.reset_next_chunk_position();
        for cr in reader.apng_scan_chunks()? {
            println!("frame");
            println!("\t{:?}", reader.read_chunk(&cr)?);
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
