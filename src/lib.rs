use nom::{
	branch::alt,
	bytes::complete::tag,
	Err::Failure,
	error::{
		ErrorKind,
		ParseError
	},
	IResult,
	number::complete::{
		be_u32,
		u8
	}
};

const FILE_ID_SIZE: usize = 7;
const FILE_ID: [u8; FILE_ID_SIZE] = [b'O', b'P', b'B', b'i', b'n', b'1', 0 ];
const NUM_CHANNELS: usize = 18;
const NUM_TRACKS: usize = NUM_CHANNELS + 1;

#[derive(Debug, PartialEq)]
pub enum OpbError<'a> {
	Format(u8),
	Read(&'a [u8], ErrorKind),
	NotAnOpbFile([u8; FILE_ID_SIZE]),
	Version,
}

impl<'a> ParseError<&'a [u8]> for OpbError<'a> {
	fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
		OpbError::Read(input, kind)
	}

	fn append(_: &'a [u8], _: ErrorKind, other: Self) -> Self {
		other
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum OpbFormat {
	Standard,
	Raw,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OpbHeader {
	id: [u8; FILE_ID_SIZE],
	fmt: OpbFormat,
	size: u32,
	num_instruments: u32,
	num_chunks: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpbFile {
	header: OpbHeader,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OpbCommand {
	addr: u16,
	data: u8,
	time: f64,
	order_index: i32,
	data_index: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OpbInstOp {
	characteristic: i16,
	attack_decay: i16,
	sustain_release: i16,
	wave_select: i16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OpbInstrument {
	feed_conn: i16,
	modulator: OpbInstOp,
	carrier: OpbInstOp,
	index: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OpbData {
	count: u32,
	args: [u8; 16],
}

#[derive(Clone, Debug, PartialEq)]
struct OpbContext {
	cmd_stream: Vec<OpbCommand>,
	fmt: OpbFormat,
	data_map: Vec<OpbData>,
	instruments: Vec<OpbInstrument>,
	tracks: [Vec<OpbCommand>; NUM_TRACKS],
	time: f64,

}

fn read_u7<'a>(input: &'a [u8]) -> IResult<&'a [u8], u32, OpbError<'a>> {
	let (mut input, mut b0) = u8(input)?;
	let mut b1 = 0;
	let mut b2 = 0;
	let mut b3 = 0;

	if b0 >= 128 {
		b0 &= 0b01111111;
		let (inp2, b) = u8(input)?;
		input = inp2;
		b1 = b;

		if b1 >= 128 {
			b1 &= 0b01111111;
			let (inp2, b) = u8(input)?;
			input = inp2;
			b2 = b;

			if b2 >= 128 {
				b2 &= 0b01111111;
				let (inp2, b) = u8(input)?;
				input = inp2;
				b3 = b;
			}
		}
	}

	Ok((input, (b0 | (b1 << 7) | (b2 << 14) | (b3 << 21)) as u32))
}

const fn size_u7(val: u32) -> usize {
	match val {
		_i if val >= 2097152 => 4,
		_i if val >= 16384 => 3,
		_i if val >= 128 => 2,
		_ => 1,
	}
}

pub fn parse_opb<'a>(input: &'a [u8]) -> IResult<&'a [u8], OpbFile, OpbError<'a>> {
	let (input, id) = tag("OPBin1\x00")(input)?;
	let id: [u8; FILE_ID_SIZE] = [id[0], id[1], id[2], id[3], id[4], id[5], id[6]];
	if id != FILE_ID {
		return Err(Failure(OpbError::NotAnOpbFile(id)));
	}

	let (input, fmti) = u8(input)?;
	if fmti > 1 {
		return Err(Failure(OpbError::Format(fmti)));
	}
	let fmt = match fmti {
		0 => OpbFormat::Standard,
		1 => OpbFormat::Raw,
		_ => unreachable!(),
	};

	let (input, size) = be_u32(input)?;
	let (input, ninst) = be_u32(input)?;
	let (input, nchunks) = be_u32(input)?;

	Ok((input, OpbFile {
		header: OpbHeader {
			id: id,
			fmt: fmt,
			size: size,
			num_instruments: ninst,
			num_chunks: nchunks,
		},
	}))
}

#[cfg(test)]
mod tests {
	#[test]
	fn test_read_opb() {
		let input = include_bytes!("../test_data/test.opb");
		println!("{:#?}", super::parse_opb(input).unwrap().1);
	}
}
