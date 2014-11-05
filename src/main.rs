extern crate serialize;

use std::os;
use std::io::{File};
use std::io::process::{Command,ProcessOutput};
use std::io::fs;
use serialize::{json, Encodable, Decodable};

static TIMEHOLDER_PATH: &'static str = "timeholder";
static PROCESSES:  [&'static str, ..4] = [ "vlc", "smplayer", "gnome-mplayer", "totem" ];
//static PROCESSES:  [&'static str, ..1] = [ "sleep" ];
static UPDATE_TIME: i64 = 5;

#[deriving(Encodable, Decodable)]
#[deriving(Show)]
pub struct Times {
	run: i64,
	idle: i64
}

fn load_times() -> Times
{
	let mut file = match File::open( &Path::new( TIMEHOLDER_PATH ) ) {
		Ok(f) => f,
		Err(_) => return Times{ run: 0, idle: 0 },
	};

	let content = file.read_to_string();
	let json_object = json::from_str( content.unwrap().as_slice() ).unwrap();

	let mut decoder = json::Decoder::new( json_object );
    let times: Times = Decodable::decode( &mut decoder ).unwrap();
	return times;
}

fn save_times( times: &Times )
{
	let string = json::encode( times );
	let mut file = File::create( &Path::new( TIMEHOLDER_PATH ) ).unwrap();

	file.write_str( string.as_slice() ).unwrap();
}

fn is_exist( name: &str ) -> bool
{
    let mut cmd = Command::new( "pgrep" );
    cmd.arg(name);

    let ProcessOutput { error: _, output: _, status: exit } = cmd.output().unwrap();
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
		std::io::timer::sleep( std::time::duration::Duration::seconds( UPDATE_TIME ) );

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
			fs::unlink( &Path::new( TIMEHOLDER_PATH ) ).unwrap();
		}
	}
}

fn main()
{
	let args = os::args();

	if args.len() < 3
	{	println!( "Usage: video_blocker <allow time> <deny time>, all times in seconds" );
		return;
	}

	let allow_time = from_str::< i64 >( args[ 1 ].as_slice() ).unwrap();
	let deny_time = from_str::< i64 >( args[ 2 ].as_slice() ).unwrap();

	println!( "Allow time is {} s, Deny time is {} s", allow_time, deny_time );

	worker( allow_time, deny_time );
}
