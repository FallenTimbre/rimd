# rimd [![Build Status](https://travis-ci.org/nicklan/rimd.svg?branch=master)](https://travis-ci.org/nicklan/rimd)

rimd is a set of utilities to deal with MIDI messages and standard
MIDI files (SMF).  It handles both standard MIDI messages and the meta
messages that are found in SMFs.

rimd is fairly low level, and  messages are stored and accessed in
their underlying format (i.e. a vector of `u8`s).  There are some
utility methods for accessing the various pieces of a message, and
for constructing new messages.

For more information on the MIDI message format, see a [description of the
underlying format of MIDI messages](
http://www.midi.org/techspecs/midimessages.php) and [one for meta messages](
https://web.archive.org/web/20150217154504/http://cs.fit.edu/~ryan/cse4051/projects/midi/midi.html#meta_event).

## Docs

Most public functions have docs in the source.  To build the docs do

    cargo doc

and then point your browser at /path/to/rimd/target/doc/rimd/index.html

## Example Usage

### MIDI Files

Read a MIDI file:
```rust
{
    use rimd::*

    match SMF::from_file(&path) {
        Ok(smf) => {
            // Process the standard MIDI file contents here.
            // See generated docs on SMF for details.
            ()
        }
        Err(e) => {
            // An error occurred in reading the file
            match e {
                SMFError::InvalidSMFFile(s) => {println!("{}", s);}
                SMFError::Error(e) => {println!("io: {}", e);}
                SMFError::MidiError(_) => {println!("MIDI Error");}
                SMFError::MetaError(_) => {println!("Meta Error");}
            }
        }
    }
}
```

Write a MIDI file:
```rust
{
    use rimd::*;

    // Placeholder empty standard MIDI file.
    let smf = SMF {
        format: SMFFormat::Single,
        division: 96,
        tracks: vec![]
    };

    let writer = SMFWriter::from_smf(smf);
    writer.write_to_file(&path::Path::new(path)).unwrap();
}
```

### MIDI Data

Compose a MIDI Track:
```rust
{
    use rimd::*;

    // Basic track representing one percussion hit as it would be exported from
    // a DAW like Ableton Live.
    let track = Track {
        copyright: Some("Alex Kesling 2020\u{0}".to_string()),
        name: Some("Example Track\u{0}".to_string()),
        events: vec![
            TrackEvent {
                vtime: 0,
                event: Event::Meta(MetaEvent {
                    command: MetaCommand::SequenceOrTrackName,
                    length: 5,
                    data: vec![116, 101, 115, 116, 0],
                }),
            },
            TrackEvent {
                vtime: 0,
                event: Event::Meta(MetaEvent {
                    command: MetaCommand::TimeSignature,
                    length: 4,
                    data: vec![4, 2, 36, 8],
                }),
            },
            TrackEvent {
                vtime: 0,
                event: Event::Meta(MetaEvent {
                    command: MetaCommand::TimeSignature,
                    length: 4,
                    data: vec![4, 2, 36, 8],
                }),
            },
            TrackEvent {
                vtime: 0,
                event: Event::Midi(MidiMessage {
                    data: vec![144, 45, 100],
                }),
            },
            TrackEvent {
                vtime: 6,
                event: Event::Midi(MidiMessage {
                    data: vec![128, 45, 64],
                }),
            },
            TrackEvent {
                vtime: 0,
                event: Event::Meta(MetaEvent {
                    command: MetaCommand::EndOfTrack,
                    length: 0,
                    data: vec![],
                }),
            },
        ],
    };
}
```

## Installation

Use [Cargo](http://doc.crates.io/) and add the following to your Cargo.toml

```
[dependencies.rimd]
git = "https://github.com/RustAudio/rimd.git"
```

## Building

To build simply do

    cargo build

## License

MIT (see LICENSE file)
