use std::{io::Cursor, sync::Arc};

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
    pub fn play(&mut self, handle: ArenaId<AudioSource>) {
        let mut audio = self.audio_sources.get_mut(handle).unwrap();

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

    pub fn paused(&mut self, handle: ArenaId<AudioSource>) -> bool {
        let audio = self.audio_sources.get_mut(handle).unwrap();

        if let Some(sink) = &audio.sink {
            sink.is_paused()
        } else {
            false
        }
    }

    pub fn pause(&mut self, handle: ArenaId<AudioSource>) {
        let audio = self.audio_sources.get_mut(handle).unwrap();

        if let Some(sink) = &audio.sink {
            sink.pause();
        }
    }

    pub fn add(&mut self, bytes: Vec<u8>) -> ArenaId<AudioSource> {
        self.audio_sources.insert(AudioSource {
            bytes: bytes.into(),
            sink: None,
        })
    }
}

pub struct AudioSource {
    pub bytes: Arc<[u8]>,
    pub sink: Option<Sink>,
}
