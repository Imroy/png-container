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
use std::str;

use png_container::reader::*;
use png_container::chunks::*;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let f = File::open(&args[1])?;
    let bf = BufReader::new(f);
    let mut reader = PNGSeekableReader::from_stream(bf)?;
    println!(
        "filetype={:?}, width={}, height={}, bit_depth={}, colour_type={:?}",
        reader.filetype, reader.width, reader.height, reader.bit_depth, reader.colour_type
    );

    println!("{} chunks.", reader.all_chunks.len());
    if reader.plte.is_some() {
        println!("\tPLTE chunk");
    } else {
        println!("\tNo PLTE chunk");
    }
    println!("\t{} IDAT chunks", reader.idats.len());
    println!("\t{} frames", reader.frames.len());
    for chunktype in reader.optional_multi_chunks.keys() {
        println!("\t{} {} chunk(s).",
                 reader.optional_multi_chunks[chunktype].len(),
                 str::from_utf8(chunktype).unwrap_or("")
        );
    }
    println!("");

    for c in &reader.all_chunks {
        println!("type_str={}, chunk={:?}", c.type_str(), c);

        let ct = c.read_chunk(&mut reader.stream, Some(&reader.ihdr));
        if let Ok(ct) = ct {
            println!("data={:?}", &ct);
            match ct {
                PNGChunkData::CHRM { .. } => {
                    println!("white_coords={:?}", ct.chrm_white_coords());
                    println!("  red_coords={:?}", ct.chrm_red_coords());
                    println!("green_coords={:?}", ct.chrm_green_coords());
                    println!( "blue_coords={:?}", ct.chrm_blue_coords());
                },

                PNGChunkData::GAMA { .. } => {
                    if let Some(gamma) = ct.gama_gamma() {
                        println!("gamma={}", gamma);
                    }
                },

                PNGChunkData::ZTXT { .. } => {
                    if let Some(string) = ct.ztxt_string() {
                        println!("string=\"{}\"", string);
                    }
                },

                PNGChunkData::ITXT { .. } => {
                    if let Some(string) = ct.itxt_string() {
                        println!("string=\"{}\"", string);
                    }
                },

                PNGChunkData::FCTL { .. } => {
                    if let Some(delay) = ct.fctl_delay() {
                        println!("delay={}", delay.into_format_args(second, Abbreviation));
                    }
                },

                _ => ()
            }
        }
        println!("");
    }

    Ok(())
}
