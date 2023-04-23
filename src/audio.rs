use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    sync::Arc,
};

use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::arena::{Arena, ArenaId};

pub struct Audio {
    audio_sources: Arena<AudioSource>,
    stream_handle: OutputStreamHandle,
    output_stream: OutputStream,
}

impl Default for Audio {
    fn default() -> Self {
        let (output_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

        Audio {
            audio_sources: Arena::default(),
            stream_handle,
            output_stream,
        }
    }
}

impl Audio {
    pub fn play(&mut self, audio_id: ArenaId) {
        let mut audio = self.audio_sources.get_mut(audio_id).unwrap();

        let sink = if let Some(sink) = audio.sink.take() {
            sink
        } else {
            let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
            let buffer = Cursor::new(audio.bytes.clone());
            sink.append(rodio::Decoder::new(buffer).unwrap());
            sink
        };

        sink.play();
        audio.sink = Some(sink);
    }

    pub fn paused(&mut self, audio_id: ArenaId) -> bool {
        let audio = self.audio_sources.get_mut(audio_id).unwrap();

        if let Some(sink) = &audio.sink {
            sink.is_paused()
        } else {
            false
        }
    }

    pub fn pause(&mut self, audio_id: ArenaId) {
        let audio = self.audio_sources.get_mut(audio_id).unwrap();

        if let Some(sink) = &audio.sink {
            sink.pause();
        }
    }

    pub fn add_file(&mut self, file: File) -> ArenaId {
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        self.audio_sources.insert(AudioSource {
            bytes: buffer.into(),
            sink: None,
        })
    }
}

pub struct AudioSource {
    pub bytes: Arc<[u8]>,
    pub sink: Option<Sink>,
}