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

### Basic Example

Using the `SMFBuilder` helper, we can create a simple MIDI file that plays one
note:
```rust
{
    use rimd::*;

    let mut builder = SMFBuilder::new();
    {
        let note = 45;
        let velocity = 100;
        let channel = 0;

        builder.add_track();
        builder.add_event(0, TrackEvent{
            // `vtime` represents the number of "ticks" passed since the last event
            vtime: 0,
            event: Event::Midi(MidiMessage::note_on(note, velocity, channel))
        });
        builder.add_event(0, TrackEvent{
            // `vtime` represents the number of "ticks" passed since the last event
            vtime: 0,
            event: Event::Midi(MidiMessage::note_off(note, velocity, channel))
        });
    }

    let smf = builder.result();
    smf.division = 96;

    let writer = SMFWriter::from_smf(smf);
    writer.write_to_file(&std::path::Path::new("some_file.mid")).unwrap();
}
```


### MIDI Files

Read a MIDI file:
```rust
{
    use rimd::*

    match SMF::from_file(&std::path::Path::new("some_file.mid")) {
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

    // An empty SMF object
    let smf = SMF {
        format: SMFFormat::Single,
        division: 96,
        tracks: vec![]
    };

    let writer = SMFWriter::from_smf(smf);
    writer.write_to_file(&std::path::Path::new("some_file.mid")).unwrap();
}
```

### MIDI Data

Compose a MIDI Track with raw event literals:
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

There are also some helpers that make creating events a little cleaner:
```rust
{
    use rimd::*;

    let note = 45;
    let velocity = 100;
    let channel = 0;

    let note_on_event = TrackEvent {
        // `vtime` represents the number of "ticks" passed since the last event
        vtime: 0,
        event: Event::Midi(MidiMessage::note_on(note, velocity, channel)),
    };

    let note_off_event = TrackEvent {
        // `vtime` represents the number of "ticks" passed since the last event
        vtime: 6,
        event: Event::Midi(MidiMessage::note_off(note, velocity, channel)),
    },
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
