use rodio::Sink;
use souvlaki::{MediaControlEvent, MediaControls, PlatformConfig};
use iced::futures;
use futures::{StreamExt, FutureExt, never::Never, select, channel::mpsc};
use iced::task::{sipper, Sipper};

use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock};

use crate::{MediaEvent, MediaSignal};

pub fn listen() -> impl Sipper<Never, MediaEvent> {
    sipper(async |mut output| {
        let stream_handle = rodio::OutputStreamBuilder::open_default_stream().expect("Could not open stream");
        let sink = Sink::connect_new(stream_handle.mixer());

        let (sender, mut recv) = mpsc::channel(100);
        output.send(MediaEvent::Play).await;
        output.send(MediaEvent::Connect(sender)).await;

        let mut prev = 0;

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
        let media_event: Arc<RwLock<Option<MediaControlEvent>>> = Arc::new(RwLock::new(None));

        let media_event_clone = media_event.clone();

        controls.attach(move |event| {
            *media_event_clone.write().unwrap() = Some(event);
        }).unwrap();

        let playing = Arc::new(AtomicUsize::new(0));

        loop {
            select! {
                // detect when UI signals new song should be added to the queue
                signal = recv.select_next_some() => {
                    match signal {
                        MediaSignal::AddSong(song) => {
                            let file = std::fs::File::open(song.path).expect("test");

                            let playing_clone = playing.clone();
                            sink.append(rodio::source::EmptyCallback::new(
                                Box::new(move || {
                                    playing_clone.fetch_add(1, Ordering::Relaxed);
                                })
                            ));

                            sink.append(rodio::Decoder::try_from(file).expect("test"));
                        },
                        
                        MediaSignal::PlaySong(song) => {
                            let file = std::fs::File::open(song.path).expect("test");

                            let playing_clone = playing.clone();
                            sink.append(rodio::source::EmptyCallback::new(
                                Box::new(move || {
                                    playing_clone.fetch_add(1, Ordering::Relaxed);
                                })
                            ));

                            sink.append(rodio::Decoder::try_from(file).expect("test"));
                        }
                        _ => todo!()
                    }
                },

                signal = async { media_event.read().unwrap().clone() }.fuse() => 'a: {
                    if signal.is_none() {
                        break 'a;
                    }

                    *media_event.write().unwrap() = None;

                    match signal.unwrap() {
                        MediaControlEvent::Pause => sink.pause(),
                        MediaControlEvent::Play => sink.play(),
                        MediaControlEvent::Toggle => {
                            if sink.is_paused() {
                                sink.play();
                            }
                            else {
                                sink.pause();
                            }
                        },
                        _ => {}
                    };
                },

                // detect when new song starts playing
                new = async { playing.load(Ordering::Relaxed) }.fuse() => {
                    if prev != new {
                        output.send(MediaEvent::Playing(new)).await;
                        prev = new;
                    }
                }
            }
        }
    })
}
