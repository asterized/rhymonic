use iced::futures::{StreamExt, channel::mpsc, select};
use iced::task::{Never, Sipper, sipper};
use rodio::{Player, Source};
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

use crate::{MediaEvent, MediaSignal, Song};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::{fs::File, sync::Arc};

pub enum PlayError {
    FileError,
    DecodeError,
}

pub enum TrackedEvent {
    ControlEvent(MediaControlEvent),
    NewSong,
}

pub struct Handler {
    _stream_handle: rodio::MixerDeviceSink,
    sink: Player,
    sender: sipper::Sender<MediaEvent>,
    reciever: mpsc::Receiver<MediaSignal>,
    controls: MediaControls,
    event_listener: Arc<mpsc::UnboundedReceiver<TrackedEvent>>,
    event_sender: Arc<mpsc::UnboundedSender<TrackedEvent>>,
    currently_playing: Option<Arc<Song>>,
    duration: Arc<AtomicU64>,
}

impl Handler {
    pub async fn new(mut sender: sipper::Sender<MediaEvent>) -> Self {
        let mut stream_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("Could not open stream");
        let sink = Player::connect_new(&stream_handle.mixer());

        let duration = Arc::new(AtomicU64::new(0));

        let (main_sender, recv) = mpsc::channel(100);
        sender
            .send(MediaEvent::Connect((main_sender, duration.clone())))
            .await;

        let config = PlatformConfig {
            dbus_name: "my_player",
            display_name: "My Player",
            hwnd: None,
        };

        let mut controls = MediaControls::new(config).unwrap();

        let (e_sender, e_listener) = mpsc::unbounded();
        let event_sender = Arc::new(e_sender);
        let event_listener = Arc::new(e_listener);

        let control_sender = event_sender.clone();
        controls
            .attach(move |event| {
                let _ = control_sender.unbounded_send(TrackedEvent::ControlEvent(event));
            })
            .unwrap();

        stream_handle.log_on_drop(false);

        Self {
            _stream_handle: stream_handle,
            sink: sink,
            sender: sender,
            reciever: recv,
            controls: controls,
            event_listener: event_listener,
            event_sender: event_sender,
            currently_playing: None,
            duration: duration,
        }
    }

    fn _add_song(&self, song: Arc<Song>) -> Result<(), PlayError> {
        let opened = File::open(song.path.clone()).map_err(|_| PlayError::FileError)?;

        let duration = self.duration.clone();

        let decoder = rodio::Decoder::try_from(opened)
            .map_err(|_| PlayError::DecodeError)?
            .track_position()
            .periodic_access(Duration::from_millis(50), move |s| {
                duration.store(s.get_pos().as_millis() as u64, Ordering::Relaxed);
            });

        self.sink.append(decoder);

        let sender_clone = self.event_sender.clone();
        self.sink
            .append(rodio::source::EmptyCallback::new(Box::new(move || {
                let _ = sender_clone.unbounded_send(TrackedEvent::NewSong);
            })));

        Ok(())
    }

    async fn queue_song(&mut self, song: Arc<Song>) {
        if self._add_song(song).is_err() {
            self.sender.send(MediaEvent::FailedQueue).await;
        }
    }

    async fn resume(&mut self) {
        self.sink.play();
        self.sender.send(MediaEvent::Play).await;
    }

    async fn pause(&mut self) {
        self.sink.pause();
        self.sender.send(MediaEvent::Pause).await;
    }

    async fn handle_signals(&mut self, signal: MediaSignal) {
        match signal {
            MediaSignal::PlaySong(song) => {
                self.sink.stop();

                self.queue_song(song).await;
                self.resume().await;
            }

            MediaSignal::Pause => {
                self.pause().await;
            }

            MediaSignal::Play => {
                self.resume().await;
            }

            MediaSignal::NewPosition(position) => {
                if self
                    .sink
                    .try_seek(Duration::from_millis(position))
                    .is_err()
                {
                    self.sender.send(MediaEvent::EndedSong).await;
                }
            }
        };
    }

    async fn handle_media_event(&mut self, signal: MediaControlEvent) {
        match signal {
            MediaControlEvent::Pause => self.pause().await,
            MediaControlEvent::Play => self.resume().await,
            MediaControlEvent::Toggle => {
                if self.sink.is_paused() {
                    self.resume().await;
                } else {
                    self.pause().await;
                }
            }
            _ => {}
        };
    }

    fn update_controls(&mut self) {
        if self.sink.empty() {
            let _ = self.controls.set_playback(MediaPlayback::Stopped);
            return;
        }

        let Some(playing) = self.currently_playing.as_ref() else {
            return;
        };

        let _ = self.controls.set_metadata(MediaMetadata {
            title: Some(&playing.title),
            album: Some(&playing.album),
            artist: Some(
                &playing
                    .artists
                    .iter()
                    .map(|x| &*x.name)
                    .collect::<Vec<&str>>()
                    .join(", "),
            ),
            cover_url: None,
            duration: Some(playing.length),
        });

        let progress = Some(MediaPosition(self.sink.get_pos()));
        if self.sink.is_paused() {
            let _ = self
                .controls
                .set_playback(MediaPlayback::Paused { progress: progress });
        } else {
            let _ = self
                .controls
                .set_playback(MediaPlayback::Playing { progress: progress });
        }
    }

    pub async fn handle_event(&mut self) {
        select! {
            signal = self.reciever.select_next_some() => {
                self.handle_signals(signal).await;
            },

            // safe as this is the only function where event_listener is used
            signal = Arc::get_mut(&mut self.event_listener).unwrap().select_next_some() => {
                match signal {
                    TrackedEvent::ControlEvent(event) => {
                        self.handle_media_event(event).await;
                    }

                    TrackedEvent::NewSong => {
                        self.sender.send(MediaEvent::EndedSong).await;
                    }
                }
            },
        };

        self.update_controls();
    }
}

pub fn listen() -> impl Sipper<Never, MediaEvent> {
    sipper(async move |output| {
        let mut state = Handler::new(output).await;

        loop {
            state.handle_event().await;
        }
    })
}
