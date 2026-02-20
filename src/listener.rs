use iced::futures::{StreamExt, channel::mpsc, select};
use iced::task::{Never, Sipper, sipper};
use rodio::Sink;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

use crate::{MediaEvent, MediaSignal, Song};
use std::{
    fs::File,
    sync::Arc,
};

pub enum PlayError {
    FileError,
    DecodeError,
}

pub enum TrackedEvent {
    ControlEvent(MediaControlEvent),
    NewSong,
}

pub struct Handler {
    _stream_handle: rodio::OutputStream,
    sink: Sink,
    sender: sipper::Sender<MediaEvent>,
    reciever: mpsc::Receiver<MediaSignal>,
    controls: MediaControls,
    event_listener: Arc<mpsc::UnboundedReceiver<TrackedEvent>>,
    event_sender: Arc<mpsc::UnboundedSender<TrackedEvent>>,
    currently_playing: Option<Arc<Song>>,
}

impl Handler {
    pub async fn new(mut sender: sipper::Sender<MediaEvent>) -> Self {
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("Could not open stream");
        let sink = Sink::connect_new(stream_handle.mixer());

        let (main_sender, recv) = mpsc::channel(100);
        sender.send(MediaEvent::Connect(main_sender)).await;

        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::windows::WindowsHandle;

            let handle: WindowsHandle = unimplemented!();
            Some(handle.hwnd)
        };

        let config = PlatformConfig {
            dbus_name: "my_player",
            display_name: "My Player",
            hwnd,
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

        Self {
            _stream_handle: stream_handle,
            sink: sink,
            sender: sender,
            reciever: recv,
            controls: controls,
            event_listener: event_listener,
            event_sender: event_sender,
            currently_playing: None,
        }
    }

    fn _add_song(&self, song: Arc<Song>) -> Result<(), PlayError> {
        let opened = File::open(song.path.clone()).map_err(|_| PlayError::FileError)?;

        let decoder = rodio::Decoder::try_from(opened).map_err(|_| PlayError::DecodeError)?;

        self.sink.append(decoder);

        let sender_clone = self.event_sender.clone();
        self.sink
            .append(rodio::source::EmptyCallback::new(Box::new(move || {
                let _ = sender_clone.unbounded_send(TrackedEvent::NewSong);
            })));

        Ok(())
    }

    async fn queue_song(&mut self, song: Arc<Song>) {
        if self._add_song(song).is_ok() {
            self.sender.send(MediaEvent::Queued).await;
        } else {
            self.sender.send(MediaEvent::FailedQueue).await;
        }
    }

    async fn handle_signals(&mut self, signal: MediaSignal) {
        match signal {
            MediaSignal::AddSong(song) => {
                self.queue_song(song).await;
                self.sink.play();
            }

            MediaSignal::PlaySong(song) => {
                self.sink.stop();

                self.queue_song(song).await;
                self.sink.play();
            }

            MediaSignal::PlayPause => {
                if self.sink.is_paused() {
                    self.sink.play();
                } else {
                    self.sink.pause()
                }
            }

            _ => todo!(),
        };
    }

    fn handle_media_event(&self, signal: MediaControlEvent) {
        match signal {
            MediaControlEvent::Pause => self.sink.pause(),
            MediaControlEvent::Play => self.sink.play(),
            MediaControlEvent::Toggle => {
                if self.sink.is_paused() {
                    self.sink.play();
                } else {
                    self.sink.pause();
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
            // detect when UI signals new song should be added to the queue
            signal = self.reciever.select_next_some() => {
                self.handle_signals(signal).await;
            },

            // safe as this is the only function where event_listener is used
            signal = Arc::get_mut(&mut self.event_listener).unwrap().select_next_some() => {
                match signal {
                    TrackedEvent::ControlEvent(event) => {
                        self.handle_media_event(event);
                    }

                    TrackedEvent::NewSong => {
                        self.sender.send(MediaEvent::EndedSong).await;
                    }
                }
            },
        }

        self.update_controls();
    }
}

pub fn listen() -> impl Sipper<Never, MediaEvent> {
    sipper(async move |mut output| {
        output.send(MediaEvent::Play).await;

        let mut state = Handler::new(output.clone()).await;

        loop {
            state.handle_event().await;
        }
    })
}
