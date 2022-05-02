// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::{audio::AudioSink, prelude::*};

pub struct AudioTrack {
    handle: Handle<AudioSource>,
    sink: Option<Handle<AudioSink>>,
}

impl AudioTrack {
    pub fn new(handle: Handle<AudioSource>) -> Self {
        return Self { handle, sink: None };
    }

    pub fn play(&mut self, audio: &Res<Audio>, audio_sinks: &Res<Assets<AudioSink>>) {
        if let Some(sink) = self.sink.clone() {
            if let Some(sink) = audio_sinks.get(sink) {
                if sink.is_paused() {
                    sink.stop();
                    sink.play();
                } else {
                    return;
                }
            }
        }

        let sink = audio.play_with_settings(
            self.handle.clone(),
            PlaybackSettings {
                repeat: true,
                ..default()
            },
        );

        self.sink = Some(audio_sinks.get_handle(sink));
    }

    pub fn stop(&self, audio_sinks: &Res<Assets<AudioSink>>) {
        if let Some(sink) = self.sink.clone() {
            if let Some(sink) = audio_sinks.get(sink) {
                if !sink.is_paused() {
                    sink.pause();
                }
            }
        }
    }
}
