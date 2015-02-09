use std::old_io::{File,Writer,IoError, Truncate, Write};

use SMF;
use ::{Event,AbsoluteEvent,MetaCommand,SMFFormat};

/// An SMFWriter is used to write an SMF to a file.  It can be either
/// constructed empty and have tracks added, or created from an
/// existing rimd::SMF.
///
/// # Writing an existing SMF to a file
/// ```
/// use rimd::{SMF,SMFWriter};
/// // Create smf
/// // let writer = SMFWriter::from_smf(smf);
/// // let result = writer.write_to_file(Path::new("/path/to/file.smf"));
/// // handle result
pub struct SMFWriter {
    format: u16,
    ticks: i16,
    tracks: Vec<Vec<u8>>,
}

impl SMFWriter {

    /// Create a new SMFWriter with the given number of units per
    /// beat.  The SMFWriter will initially have no tracks.
    pub fn new_with_division(ticks: i16) -> SMFWriter {
        SMFWriter {
            format: 1,
            ticks: ticks,
            tracks: Vec::new(),
        }
    }

    /// Create a new SMFWriter with the given format and number of
    /// units per beat.  The SMFWriter will initially have no tracks.
    pub fn new_with_division_and_format(format: SMFFormat, ticks: i16) -> SMFWriter {
        SMFWriter {
            format: format as u16,
            ticks: ticks,
            tracks: Vec::new(),
        }
    }

    /// Create a writer that has all the tracks from the given SMF already added
    pub fn from_smf(smf: SMF) -> SMFWriter {
        let mut writer = SMFWriter::new_with_division_and_format
            (smf.format, smf.division);

        let mut length = 0;
        let mut saw_eot = false;

        for track in smf.tracks.iter() {
            let mut vec = Vec::new();
            writer.start_track_header(&mut vec);

            for event in track.events.iter() {
                length += SMFWriter::write_vtime(event.vtime as u64,&mut vec).unwrap(); // TODO: Handle error
                writer.write_event(&mut vec, &(event.event) , &mut length, &mut saw_eot);
            }

            writer.finish_track_write(&mut vec, &mut length, saw_eot);
            writer.tracks.push(vec);
        }

        writer
    }

    // Write a variable length value.  Return number of bytes written.
    fn write_vtime(val: u64, writer: &mut Writer) -> Result<u32,IoError> {
        let mut storage = Vec::new();
        let mut cur = val;
        let mut written = 0;
        let cont_mask = 0x80 as u8;
        let val_mask = 0x7F as u64;
        loop {
            let mut to_write = (cur & val_mask) as u8;
            cur = cur >> 7;
            if written != 0 {
                // we're writing a continuation byte, so set the bit
                to_write |= cont_mask;
            }
            storage.push(to_write);
            written += 1;
            if cur == 0 { break; }
        }
        for b in storage.iter().rev() {
            try!(writer.write_u8(*b));
        }
        Ok(written)
    }

    fn start_track_header(&self, vec: &mut Vec<u8>) {
        vec.push(0x4D);
        vec.push(0x54);
        vec.push(0x72);
        vec.push(0x6B);
        // reserve space for track len
        vec.push(0);
        vec.push(0);
        vec.push(0);
        vec.push(0);
    }

    fn write_event(&self, vec: &mut Vec<u8>, event: &Event, length: &mut u32, saw_eot: &mut bool) {
        match event {
            &Event::Midi(ref midi) => {
                vec.push_all(midi.data.as_slice());
                *length += midi.data.len() as u32;
            }
            &Event::Meta(ref meta) => {
                vec.push(0xff); // indicate we're writing a meta event
                vec.push(meta.command as u8);
                // +2 on next line for the 0xff and the command byte we just wrote
                *length += SMFWriter::write_vtime(meta.length,vec).unwrap() + 2;
                vec.push_all(meta.data.as_slice());
                *length += meta.data.len() as u32;
                if meta.command == MetaCommand::EndOfTrack {
                    *saw_eot = true;
                }
            }
        }
    }

    fn finish_track_write(&self, vec: &mut Vec<u8>, length: &mut u32, saw_eot: bool) {
        if !saw_eot {
            // no end of track marker in passed data, add one
            *length += SMFWriter::write_vtime(0,vec).unwrap() + 1;
            vec.push(MetaCommand::EndOfTrack as u8);
            *length += SMFWriter::write_vtime(0,vec).unwrap() + 1; // write length of meta command: 0
        }

        // write in the length in the space we reserved
        for i in (0..4) {
            let lbyte = (*length & 0xFF) as u8;
            // 7-i because smf is big endian and we want to put this in bytes 4-7
            vec[7-i] = lbyte;
            *length = (*length)>>8;
        }
    }

    /// Add any sequence of AbsoluteEvents as a track to this writer
    pub fn add_track<'a,I>(&mut self, track: I) where I: Iterator<Item=&'a AbsoluteEvent> {
        let mut vec = Vec::new();

        self.start_track_header(&mut vec);

        let mut length = 0;
        let mut cur_time: u64 = 0;
        let mut saw_eot = false;

        for ev in track {
            let vtime = ev.get_time() - cur_time;
            cur_time = vtime;
            length += SMFWriter::write_vtime(vtime as u64,&mut vec).unwrap(); // TODO: Handle error
            self.write_event(&mut vec, ev.get_event(), &mut length, &mut saw_eot);
        }

        self.finish_track_write(&mut vec, &mut length, saw_eot);

        self.tracks.push(vec);
    }

    // actual writing stuff below

    fn write_header(&self, writer: &mut Writer) -> Result<(),IoError> {
        try!(writer.write_all(&[0x4D,0x54,0x68,0x64]));
        try!(writer.write_be_u32(6));
        try!(writer.write_be_u16(self.format));
        try!(writer.write_be_u16(self.tracks.len() as u16));
        try!(writer.write_be_i16(self.ticks));
        Ok(())
    }

    /// Write out all the tracks that have been added to this
    /// SMFWriter to the passed writer
    pub fn write_all(self, writer: &mut Writer) -> Result<(),IoError> {
        try!(self.write_header(writer));
        for track in self.tracks.into_iter() {
            try!(writer.write_all(track.as_slice()));
        }
        Ok(())
    }

    /// Write out the result of the tracks that have been added to a
    /// file.
    /// Warning: This will overwrite an existing file
    pub fn write_to_file(self, path: &Path) -> Result<(),IoError> {
        let mut file = try!(File::open_mode(path,Truncate,Write));
        self.write_all(&mut file)
    }

}

#[test]
fn vwrite() {
    let mut vec1 = Vec::new();
    SMFWriter::write_vtime(127,&mut vec1).unwrap();
    assert!(vec1[0] == 0x7f);

    vec1.clear();
    SMFWriter::write_vtime(255,&mut vec1).unwrap();
    assert!(vec1[0] == 0x81);
    assert!(vec1[1] == 0x7f);

    vec1.clear();
    SMFWriter::write_vtime(32768,&mut vec1).unwrap();
    assert!(vec1[0] == 0x82);
    assert!(vec1[1] == 0x80);
    assert!(vec1[2] == 0x00);
}
