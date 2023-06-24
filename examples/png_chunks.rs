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
    let mut reader = PNGFileReader::from_stream(bf)?;
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
        if ct.is_ok() {
            let ct2 = ct.unwrap();
            println!("data={:?}", &ct2);
            match ct2 {
                PNGChunkData::CHRM { .. } => {
                    println!("white_coords={:?}", ct2.chrm_white_coords());
                    println!("  red_coords={:?}", ct2.chrm_red_coords());
                    println!("green_coords={:?}", ct2.chrm_green_coords());
                    println!( "blue_coords={:?}", ct2.chrm_blue_coords());
                },

                PNGChunkData::GAMA { .. } => {
                    let gamma = ct2.gama_gamma();
                    if gamma.is_ok() {
                        println!("gamma={}", gamma.unwrap());
                    }
                },

                PNGChunkData::ZTXT { .. } => {
                    let string = ct2.ztxt_string();
                    if string.is_ok() {
                        println!("string=\"{}\"", string.unwrap());
                    }
                },

                PNGChunkData::ITXT { .. } => {
                    let string = ct2.itxt_string();
                    if string.is_ok() {
                        println!("string=\"{}\"", string.unwrap());
                    }
                },

                PNGChunkData::FCTL { .. } => {
                    let delay = ct2.fctl_delay();
                    if delay.is_ok() {
                        println!("delay={}", delay.unwrap());
                    }
                },

                _ => ()
            }
        }
        println!("");
    }

    Ok(())
}
