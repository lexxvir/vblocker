
extern crate rustc_serialize;

use std::fs::File;
use std::env;
use std::process::{Command, Output};
use rustc_serialize::{json, Encodable};
use std::io::prelude::*;

static TIMEHOLDER_PATH: &'static str = "timeholder";
static PROCESSES: &'static [&'static str] = &[ "vlc", "smplayer", "gnome-mplayer", "totem" ] /*[ "sleep" ]*/;
static UPDATE_TIME: i64 = 5;

#[derive(RustcEncodable, RustcDecodable)]
#[derive(Debug)]
pub struct Times {
	run: i64,
	idle: i64
}

fn load_times() -> Times
{
	let mut file = match File::open( TIMEHOLDER_PATH ) {
		Ok(f) => f,
		Err(_) => return Times{ run: 0, idle: 0 },
	};

	let mut content = String::new();
	file.read_to_string( &mut content ).unwrap();
	return json::decode( &content ).unwrap();
}

fn save_times( times: &Times )
{
	let string = json::encode( times ).unwrap();
	let mut file = File::create( TIMEHOLDER_PATH ).unwrap();

	file.write_all( string.as_bytes() ).unwrap();
}

fn is_exist( name: &str ) -> bool
{
    let mut cmd = Command::new( "pgrep" );
    cmd.arg(name);

    let Output { stderr: _, stdout: _, status: exit } = cmd.output().unwrap();
	return exit.success();
}

fn check_processes() -> bool
{
	for p in PROCESSES.iter() {
		if is_exist( *p ) {
			return true;
		}
	}
	return false;
}

fn kill_processes()
{
	println!("kill!");

	for p in PROCESSES.iter() {
		let mut cmd = Command::new( "pkill" );
		cmd.arg( *p );
		let _ = cmd.output();
	}
}

fn worker( allow_time: i64, deny_time: i64 )
{
	loop {
		std::thread::sleep_ms( UPDATE_TIME as u32 * 1000 );

		let mut times = load_times();
		if check_processes() {
			times.run += UPDATE_TIME;
		}
		else {
			times.idle += UPDATE_TIME;
		}

		save_times( &times );

		if times.run > allow_time && times.idle < deny_time
		{
			kill_processes();
		}
		else if times.idle >= deny_time
		{
			std::fs::remove_file( TIMEHOLDER_PATH ).unwrap();
		}
	}
}

fn main()
{
	let args = env::args();

	if args.len() < 3
	{	println!( "Usage: video_blocker <allow time> <deny time>, all times in seconds" );
		return;
	}

	let mut allow_time: i64 = 0;
	let mut deny_time: i64 = 0;
	let mut i = 0;

	for arg in args {
		match i {
			1 => allow_time = arg.parse().unwrap(),
			2 => deny_time = arg.parse().unwrap(),
			_ => (),
		};

		i += 1;
	}

	println!( "Allow time is {} s, Deny time is {} s", allow_time, deny_time );

	worker( allow_time, deny_time );
}
