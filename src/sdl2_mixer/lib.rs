//! A binding for SDL2_mixer.
//!

#![crate_name = "sdl2_mixer"]
#![crate_type = "lib"]

#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate sdl2;

use std::default;
use std::ptr;
use std::fmt;
use std::ffi::{CString, CStr};
use std::str::from_utf8;
use std::borrow::ToOwned;
use std::path::Path;
use libc::{c_int, uint16_t, c_double, c_uint};
use sdl2::get_error;
use sdl2::rwops::RWops;
use sdl2::version::Version;

// Setup linking for all targets.
#[cfg(target_os="macos")]
mod mac {
    #[cfg(mac_framework)]
    #[link(kind="framework", name="SDL2_mixer")]
    extern "C" {
    }

    #[cfg(not(mac_framework))]
    #[link(name="SDL2_mixer")]
    extern "C" {
    }
}

#[cfg(any(target_os="windows", target_os="linux", target_os="freebsd"))]
mod others {
    #[link(name="SDL2_mixer")]
    extern "C" {
    }
}

#[allow(non_camel_case_types, dead_code)]
mod ffi;

// This comes from SDL_audio.h
#[allow(non_camel_case_types)]
mod ll {
    use libc::uint16_t;

    pub const AUDIO_U8: uint16_t = 0x0008;
    pub const AUDIO_S8: uint16_t = 0x8008;
    pub const AUDIO_U16LSB: uint16_t = 0x0010;
    pub const AUDIO_S16LSB: uint16_t = 0x8010;
    pub const AUDIO_U16MSB: uint16_t = 0x1010;
    pub const AUDIO_S16MSB: uint16_t = 0x9010;
    pub const AUDIO_U16: uint16_t = AUDIO_U16LSB;
    pub const AUDIO_S16: uint16_t = AUDIO_S16LSB;
    pub const AUDIO_S32LSB: uint16_t = 0x8020;
    pub const AUDIO_S32MSB: uint16_t = 0x9020;
    pub const AUDIO_S32: uint16_t = AUDIO_S32LSB;
    pub const AUDIO_F32LSB: uint16_t = 0x8120;
    pub const AUDIO_F32MSB: uint16_t = 0x9120;
    pub const AUDIO_F32: uint16_t = AUDIO_F32LSB;
    pub const AUDIO_U16SYS: uint16_t = AUDIO_U16LSB;
    pub const AUDIO_S16SYS: uint16_t = AUDIO_S16LSB;
    pub const AUDIO_S32SYS: uint16_t = AUDIO_S32LSB;
    pub const AUDIO_F32SYS: uint16_t = AUDIO_F32LSB;
}

pub type AudioFormat = uint16_t;

pub const AUDIO_U8: AudioFormat = ll::AUDIO_U8;
pub const AUDIO_S8: AudioFormat = ll::AUDIO_S8;
pub const AUDIO_U16LSB: AudioFormat = ll::AUDIO_U16LSB;
pub const AUDIO_S16LSB: AudioFormat = ll::AUDIO_S16LSB;
pub const AUDIO_U16MSB: AudioFormat = ll::AUDIO_U16MSB;
pub const AUDIO_S16MSB: AudioFormat = ll::AUDIO_S16MSB;
pub const AUDIO_U16: AudioFormat = ll::AUDIO_U16;
pub const AUDIO_S16: AudioFormat = ll::AUDIO_S16;
pub const AUDIO_S32LSB: AudioFormat = ll::AUDIO_S32LSB;
pub const AUDIO_S32MSB: AudioFormat = ll::AUDIO_S32MSB;
pub const AUDIO_S32: AudioFormat = ll::AUDIO_S32;
pub const AUDIO_F32LSB: AudioFormat = ll::AUDIO_F32LSB;
pub const AUDIO_F32MSB: AudioFormat = ll::AUDIO_F32MSB;
pub const AUDIO_F32: AudioFormat = ll::AUDIO_F32;
pub const AUDIO_U16SYS: AudioFormat = ll::AUDIO_U16SYS;
pub const AUDIO_S16SYS: AudioFormat = ll::AUDIO_S16SYS;
pub const AUDIO_S32SYS: AudioFormat = ll::AUDIO_S32SYS;
pub const AUDIO_F32SYS: AudioFormat = ll::AUDIO_F32SYS;

/// The suggested default is signed 16bit samples in host byte order.
pub const DEFAULT_FORMAT: AudioFormat = ll::AUDIO_S16SYS;
/// Defualt channels: Stereo.
pub const DEFAULT_CHANNELS: isize = 2;
/// Good default sample rate in Hz (samples per second) for PC sound cards.
pub const DEFAULT_FREQUENCY: isize = 22050;
/// Maximum value for any volume setting.
pub const MAX_VOLUME: isize = 128;

/// Returns the version of the dynamically linked SDL_mixer library
pub fn get_linked_version() -> Version {

    unsafe { Version::from_ll(*ffi::Mix_Linked_Version()) }
}

bitflags!(flags InitFlag : u32 {
    const INIT_FLAC       = ffi::MIX_INIT_FLAC as u32,
    const INIT_MOD        = ffi::MIX_INIT_MOD as u32,
    const INIT_MODPLUG    = ffi::MIX_INIT_MODPLUG as u32,
    const INIT_MP3        = ffi::MIX_INIT_MP3 as u32,
    const INIT_OGG        = ffi::MIX_INIT_OGG as u32,
    const INIT_FLUIDSYNTH = ffi::MIX_INIT_FLUIDSYNTH as u32
});

impl ToString for InitFlag {
    fn to_string(&self) -> String {
        let mut string = "".to_string();
        if self.contains(INIT_FLAC) {
            string = string + &"INIT_FLAC ".to_string();
        }
        if self.contains(INIT_MOD) {
            string = string + &"INIT_MOD ".to_string();
        }
        if self.contains(INIT_MODPLUG) {
            string = string + &"INIT_MODPLUG ".to_string();
        }
        if self.contains(INIT_MP3) {
            string = string + &"INIT_MP3 ".to_string();
        }
        if self.contains(INIT_OGG) {
            string = string + &"INIT_OGG ".to_string();
        }
        if self.contains(INIT_FLUIDSYNTH) {
            string = string + &"INIT_FLUIDSYNTH ".to_string();
        }
        string
    }
}

/// Context manager for sdl2_mixer to manage init and quit
pub struct Sdl2MixerContext;

/// Cleans up all dynamically loaded library handles, freeing memory.
impl Drop for Sdl2MixerContext {
    fn drop(&mut self) {
        unsafe {
            ffi::Mix_Quit();
        }
    }
}

/// Loads dynamic libraries and prepares them for use.  Flags should be
/// one or more flags from InitFlag.
pub fn init(flags: InitFlag) -> Result<Sdl2MixerContext, String> {
    let return_flags = unsafe {
        let ret = ffi::Mix_Init(flags.bits() as c_int);
        InitFlag::from_bits_truncate(ret as u32)
    };
    // Check if all init flags were set
    if !flags.intersects(return_flags) {
        // Flags not matching won't always set the error message text
        // according to sdl docs
        if get_error() == "".to_string() {
            let un_init_flags = return_flags ^ flags;
            let error_str = &("Could not init: ".to_string() + &un_init_flags.to_string());
            sdl2::set_error(error_str).unwrap();
        }
        Err(get_error())
    } else {
        Ok(Sdl2MixerContext)
    }
}


/// Open the mixer with a certain audio format.
pub fn open_audio(frequency: isize,
                  format: AudioFormat,
                  channels: isize,
                  chunksize: isize)
                  -> Result<(), String> {
    let ret = unsafe {
        ffi::Mix_OpenAudio(frequency as c_int,
                           format,
                           channels as c_int,
                           chunksize as c_int)
    };
    if ret == 0 {
        Ok(())
    } else {
        Err(get_error())
    }
}

/// Shutdown and cleanup the mixer API.
pub fn close_audio() {
    unsafe { ffi::Mix_CloseAudio() }
}

/// Get the actual audio format in use by the opened audio device.
pub fn query_spec() -> Result<(isize, AudioFormat, isize), String> {
    let frequency: c_int = 0;
    let format: uint16_t = 0;
    let channels: c_int = 0;
    let ret = unsafe { ffi::Mix_QuerySpec(&frequency, &format, &channels) };
    if ret == 0 {
        Err(get_error())
    } else {
        Ok((frequency as isize, format as AudioFormat, channels as isize))
    }
}

// 4.2 Samples

/// Get the number of sample chunk decoders available from the Mix_GetChunkDecoder function.
pub fn get_chunk_decoders_number() -> isize {
    unsafe { ffi::Mix_GetNumChunkDecoders() as isize }
}

/// Get the name of the indexed sample chunk decoder.
pub fn get_chunk_decoder(index: isize) -> String {
    unsafe {
        let name = ffi::Mix_GetChunkDecoder(index as c_int);
        from_utf8(CStr::from_ptr(name).to_bytes()).unwrap().to_owned()
    }
}

/// The internal format for an audio chunk.
#[derive(PartialEq)]
pub struct Chunk {
    pub raw: *const ffi::Mix_Chunk,
    pub owned: bool,
}

impl Drop for Chunk {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi::Mix_FreeChunk(self.raw) }
        }
    }
}

impl Chunk {
    /// Load file for use as a sample.
    pub fn from_file(path: &Path) -> Result<Chunk, String> {
        let raw = unsafe { ffi::Mix_LoadWAV_RW(try!(RWops::from_file(path, "rb")).raw(), 0) };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Chunk {
                raw: raw,
                owned: true,
            })
        }
    }

    /// Set chunk->volume to volume.
    pub fn set_volume(&mut self, volume: isize) -> isize {
        unsafe { ffi::Mix_VolumeChunk(self.raw, volume as c_int) as isize }
    }

    /// current volume for the chunk.
    pub fn get_volume(&self) -> isize {
        unsafe { ffi::Mix_VolumeChunk(self.raw, -1) as isize }
    }
}

/// Loader trait for RWops
pub trait LoaderRWops {
    /// Load src for use as a sample.
    fn load_wav(&self) -> Result<Chunk, String>;
}

impl<'a> LoaderRWops for RWops<'a> {
    /// Load src for use as a sample.
    fn load_wav(&self) -> Result<Chunk, String> {
        let raw = unsafe { ffi::Mix_LoadWAV_RW(self.raw(), 0) };
        if raw == ptr::null() {
            Err(get_error())
        } else {
            Ok(Chunk {
                raw: raw,
                owned: true,
            })
        }
    }
}


// 4.3 Channels

/// Fader effect type enumerations
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub enum Fading {
    NoFading = ffi::MIX_NO_FADING as isize,
    FadingOut = ffi::MIX_FADING_OUT as isize,
    FadingIn = ffi::MIX_FADING_IN as isize,
}

/// Sound effect channel.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Channel(isize);

/// Return a channel object.
pub fn channel(chan: isize) -> Channel {
    Channel(chan)
}

/// Set the number of channels being mixed.
pub fn allocate_channels(numchans: isize) -> isize {
    unsafe { ffi::Mix_AllocateChannels(numchans as c_int) as isize }
}

static mut channel_finished_callback: Option<fn(Channel)> = None;

extern "C" fn c_channel_finished_callback(ch: c_int) {
    unsafe {
        match channel_finished_callback {
            None => (),
            Some(cb) => cb(Channel(ch as isize)),
        }
    }
}

/// When channel playback is halted, then the specified channel_finished function is called.
pub fn set_channel_finished(f: fn(Channel)) {
    unsafe {
        channel_finished_callback = Some(f);
        ffi::Mix_ChannelFinished(Some(c_channel_finished_callback as extern "C" fn(ch: c_int)));
    }
}

/// Unhooks the specified function set before, so no function is called when channel playback is
/// halted.
pub fn unset_channel_finished() {
    unsafe {
        ffi::Mix_ChannelFinished(None);
        channel_finished_callback = None;
    }
}

impl Channel {
    /// Represent for all channels (-1)
    pub fn all() -> Channel {
        Channel(-1)
    }

    /// This is the MIX_CHANNEL_POST (-2)
    pub fn post() -> Channel {
        Channel(-2)
    }

    /// Set the volume for any allocated channel.
    pub fn set_volume(self, volume: isize) -> isize {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Volume(ch as c_int, volume as c_int) as isize }
    }

    /// Returns the channels volume on scale of 0 to 128.
    pub fn get_volume(self) -> isize {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Volume(ch as c_int, -1) as isize }
    }

    /// Play chunk on channel, or if channel is -1, pick the first free unreserved channel.
    pub fn play(self, chunk: &Chunk, loops: isize) -> Result<Channel, String> {
        self.play_timed(chunk, loops, -1)
    }

    pub fn play_timed(self, chunk: &Chunk, loops: isize, ticks: isize) -> Result<Channel, String> {
        let Channel(ch) = self;
        let ret = unsafe {
            ffi::Mix_PlayChannelTimed(ch as c_int, chunk.raw, loops as c_int, ticks as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(Channel(ret as isize))
        }
    }

    /// Play chunk on channel, or if channel is -1, pick the first free unreserved channel.
    pub fn fade_in(self, chunk: &Chunk, loops: isize, ms: isize) -> Result<Channel, String> {
        self.fade_in_timed(chunk, loops, ms, -1)
    }

    pub fn fade_in_timed(self,
                         chunk: &Chunk,
                         loops: isize,
                         ms: isize,
                         ticks: isize)
                         -> Result<Channel, String> {
        let Channel(ch) = self;
        let ret = unsafe {
            ffi::Mix_FadeInChannelTimed(ch as c_int,
                                        chunk.raw,
                                        loops as c_int,
                                        ms as c_int,
                                        ticks as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(Channel(ret as isize))
        }
    }

    /// Pause channel, or all playing channels if -1 is passed in.
    pub fn pause(self) {
        let Channel(ch) = self;
        unsafe {
            ffi::Mix_Pause(ch as c_int);
        }
    }

    /// Unpause channel, or all playing and paused channels if -1 is passed in.
    pub fn resume(self) {
        let Channel(ch) = self;
        unsafe {
            ffi::Mix_Resume(ch as c_int);
        }
    }

    /// Halt channel playback
    pub fn halt(self) {
        let Channel(ch) = self;
        unsafe {
            ffi::Mix_HaltChannel(ch as c_int);
        }
    }

    /// Halt channel playback, after ticks milliseconds.
    pub fn expire(self, ticks: isize) -> isize {
        let Channel(ch) = self;
        unsafe { ffi::Mix_ExpireChannel(ch as c_int, ticks as c_int) as isize }
    }

    /// Gradually fade out which channel over ms milliseconds starting from now.
    pub fn fade_out(self, ms: isize) -> isize {
        let Channel(ch) = self;
        unsafe { ffi::Mix_FadeOutChannel(ch as c_int, ms as c_int) as isize }
    }

    /// if channel is playing, or not.
    pub fn is_playing(self) -> bool {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Playing(ch as c_int) != 0 }
    }

    ///  if channel is paused, or not.
    pub fn is_paused(self) -> bool {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Paused(ch as c_int) != 0 }
    }

    /// if channel is fading in, out, or not
    pub fn get_fading(self) -> Fading {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_FadingChannel(ch as c_int) as c_uint };
        match ret {
            ffi::MIX_NO_FADING => Fading::NoFading,
            ffi::MIX_FADING_OUT => Fading::FadingOut,
            ffi::MIX_FADING_IN => Fading::FadingIn,
            _ => Fading::NoFading,
        }
    }

    /// Get the most recent sample chunk pointer played on channel.
    pub fn get_chunk(self) -> Option<Chunk> {
        let Channel(ch) = self;
        let raw = unsafe { ffi::Mix_GetChunk(ch as c_int) };
        if raw.is_null() {
            None
        } else {
            Some(Chunk {
                raw: raw,
                owned: false,
            })
        }
    }

    /// This removes all effects registered to channel.
    pub fn unregister_all_effects(self) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_UnregisterAllEffects(ch as c_int) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Sets a panning effect, where left and right is the volume of the left and right channels.
    /// They range from 0 (silence) to 255 (loud).
    pub fn set_panning(self, left: u8, right: u8) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPanning(ch as c_int, left, right) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Unregisters panning effect.
    pub fn unset_panning(self) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPanning(ch as c_int, 255, 255) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// This effect simulates a simple attenuation of volume due to distance.
    /// distance ranges from 0 (close/loud) to 255 (far/quiet).
    pub fn set_distance(self, distance: u8) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetDistance(ch as c_int, distance) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Unregisters distance effect.
    pub fn unset_distance(self) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetDistance(ch as c_int, 0) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// This effect emulates a simple 3D audio effect.
    /// angle ranges from 0 to 360 degrees going clockwise, where 0 is directly in front.
    /// distance ranges from 0 (close/loud) to 255 (far/quiet).
    pub fn set_position(self, angle: i16, distance: u8) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPosition(ch as c_int, angle, distance) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Unregisters position effect.
    pub fn unset_position(self) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPosition(ch as c_int, 0, 0) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Simple reverse stereo, swaps left and right channel sound.
    /// true for reverse, false to unregister effect.
    pub fn set_reverse_stereo(self, flip: bool) -> Result<(), String> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetReverseStereo(ch as c_int, flip as c_int) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }
}

/// Returns how many channels are currently playing.
pub fn get_playing_channels_number() -> isize {
    unsafe { ffi::Mix_Playing(-1) as isize }
}

/// Returns how many channels are currently paused.
pub fn get_paused_channels_number() -> isize {
    unsafe { ffi::Mix_Paused(-1) as isize }
}

// 4.4 Groups

/// Reserve num channels from being used when playing samples when
/// passing in -1 as a channel number to playback functions.
pub fn reserve_channels(num: isize) -> isize {
    unsafe { ffi::Mix_ReserveChannels(num as c_int) as isize }
}

/// Sound effect channel grouping.
#[derive(Copy, Clone)]
pub struct Group(isize);

impl default::Default for Group {
    fn default() -> Group {
        Group(-1)
    }
}

impl Group {
    /// Add channels starting at from up through to to group tag,
    /// or reset it's group to the default group tag (-1).
    pub fn add_channels_range(self, from: isize, to: isize) -> isize {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupChannels(from as c_int, to as c_int, g as c_int) as isize }
    }

    /// Add which channel to group tag, or reset it's group to the default group tag
    pub fn add_channel(self, Channel(ch): Channel) -> bool {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupChannel(ch as c_int, g as c_int) == 1 }
    }

    /// Count the number of channels in group
    pub fn count(self) -> isize {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupCount(g as c_int) as isize }
    }

    /// Find the first available (not playing) channel in group
    pub fn find_available(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupAvailable(g as c_int) as isize };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Find the oldest actively playing channel in group
    pub fn find_oldest(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupOldest(g as c_int) as isize };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Find the newest, most recently started, actively playing channel in group.
    pub fn find_newest(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupNewer(g as c_int) as isize };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Gradually fade out channels in group over some milliseconds starting from now.
    /// Returns the number of channels set to fade out.
    pub fn fade_out(self, ms: isize) -> isize {
        let Group(g) = self;
        unsafe { ffi::Mix_FadeOutGroup(g as c_int, ms as c_int) as isize }
    }

    /// Halt playback on all channels in group.
    pub fn halt(self) {
        let Group(g) = self;
        unsafe {
            ffi::Mix_HaltGroup(g as c_int);
        }
    }
}

// 4.5 Music

/// Get the number of music decoders available.
pub fn get_music_decoders_number() -> isize {
    unsafe { ffi::Mix_GetNumMusicDecoders() as isize }
}

/// Get the name of the indexed music decoder.
pub fn get_music_decoder(index: isize) -> String {
    unsafe {
        let name = ffi::Mix_GetMusicDecoder(index as c_int);
        from_utf8(CStr::from_ptr(name).to_bytes()).unwrap().to_owned()
    }
}

/// Music type enumerations
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum MusicType {
    MusicNone = ffi::MUS_NONE as isize,
    MusicCmd = ffi::MUS_CMD as isize,
    MusicWav = ffi::MUS_WAV as isize,
    MusicMod = ffi::MUS_MOD as isize,
    MusicMid = ffi::MUS_MID as isize,
    MusicOgg = ffi::MUS_OGG as isize,
    MusicMp3 = ffi::MUS_MP3 as isize,
    MusicMp3Mad = ffi::MUS_MP3_MAD as isize,
    MusicFlac = ffi::MUS_FLAC as isize,
    MusicModPlug = ffi::MUS_MODPLUG as isize,
}

// hooks
static mut music_finished_hook: Option<fn()> = None;

extern "C" fn c_music_finished_hook() {
    unsafe {
        match music_finished_hook {
            None => (),
            Some(f) => f(),
        }
    }
}

/// This is an opaque data type used for Music data.
#[derive(PartialEq)]
pub struct Music {
    pub raw: *const ffi::Mix_Music,
    pub owned: bool,
}

impl Drop for Music {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi::Mix_FreeMusic(self.raw) };
        }
    }
}

impl fmt::Debug for Music {
    /// Shows the original regular expression.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Music>")
    }
}

impl Music {
    /// Load music file to use.
    pub fn from_file(path: &Path) -> Result<Music, String> {
        let raw = unsafe {
            ffi::Mix_LoadMUS(CString::new(path.to_str().unwrap()).unwrap().as_ptr())
        };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Music {
                raw: raw,
                owned: true,
            })
        }
    }

    /// The file format encoding of the music.
    pub fn get_type(&self) -> MusicType {
        let ret = unsafe { ffi::Mix_GetMusicType(self.raw) as isize } as c_uint;
        match ret {
            ffi::MUS_NONE => MusicType::MusicNone,
            ffi::MUS_CMD => MusicType::MusicCmd,
            ffi::MUS_WAV => MusicType::MusicWav,
            ffi::MUS_MOD => MusicType::MusicMod,
            ffi::MUS_MID => MusicType::MusicMid,
            ffi::MUS_OGG => MusicType::MusicOgg,
            ffi::MUS_MP3 => MusicType::MusicMp3,
            ffi::MUS_MP3_MAD => MusicType::MusicMp3Mad,
            ffi::MUS_FLAC => MusicType::MusicFlac,
            ffi::MUS_MODPLUG => MusicType::MusicModPlug,
            _ => MusicType::MusicNone,
        }
    }

    /// Play the loaded music loop times through from start to finish.
    pub fn play(&self, loops: isize) -> Result<(), String> {
        let ret = unsafe { ffi::Mix_PlayMusic(self.raw, loops as c_int) };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Fade in over ms milliseconds of time, the loaded music,
    /// playing it loop times through from start to finish.
    pub fn fade_in(&self, loops: isize, ms: isize) -> Result<(), String> {
        let ret = unsafe { ffi::Mix_FadeInMusic(self.raw, loops as c_int, ms as c_int) };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Fade in over ms milliseconds of time, from position.
    pub fn fade_in_from_pos(&self, loops: isize, ms: isize, position: f64) -> Result<(), String> {
        let ret = unsafe {
            ffi::Mix_FadeInMusicPos(self.raw, loops as c_int, ms as c_int, position as c_double)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    // FIXME: make these class method?
    /// Returns current volume
    pub fn get_volume() -> isize {
        unsafe { ffi::Mix_VolumeMusic(-1) as isize }
    }

    /// Set the volume on a scale of 0 to 128.
    /// Values greater than 128 will use 128.
    pub fn set_volume(volume: isize) {
        // This shouldn't return anything. Use get_volume instead
        let _ = unsafe { ffi::Mix_VolumeMusic(volume as c_int) as isize };
    }

    /// Pause the music playback.
    pub fn pause() {
        unsafe {
            ffi::Mix_PauseMusic();
        }
    }

    /// Unpause the music.
    pub fn resume() {
        unsafe {
            ffi::Mix_ResumeMusic();
        }
    }

    /// Rewind the music to the start.
    pub fn rewind() {
        unsafe {
            ffi::Mix_RewindMusic();
        }
    }

    /// Set the position of the currently playing music.
    pub fn set_pos(position: f64) -> Result<(), String> {
        let ret = unsafe { ffi::Mix_SetMusicPosition(position as c_double) };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Setup a command line music player to use to play music.
    pub fn set_command(command: &str) -> Result<(), String> {
        let ret = unsafe { ffi::Mix_SetMusicCMD(CString::new(command).unwrap().as_ptr()) };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Halt playback of music.
    pub fn halt() {
        unsafe {
            ffi::Mix_HaltMusic();
        }
    }

    /// Gradually fade out the music over ms milliseconds starting from now.
    pub fn fade_out(ms: isize) -> Result<(), String> {
        let ret = unsafe { ffi::Mix_FadeOutMusic(ms as c_int) };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    // TODO: Mix_HookMusic
    // TODO: Mix_GetMusicHookData

    /// Sets up a function to be called when music playback is halted.
    ///
    /// # Examples
    ///
    /// ```
    /// fn after_music() {
    ///     println!("Music has ended");
    /// }
    ///
    /// sdl2_mixer::Music::hook_finished(after_music);
    /// ```
    pub fn hook_finished(f: fn()) {
        unsafe {
            music_finished_hook = Some(f);
            ffi::Mix_HookMusicFinished(Some(c_music_finished_hook as extern "C" fn()));
        }
    }

    /// A previously set up function would no longer be called when music playback is halted.
    pub fn unhook_finished() {
        unsafe {
            ffi::Mix_HookMusicFinished(None);
            // unset from c, then rust, to avoid race condiction
            music_finished_hook = None;
        }
    }

    /// If music is actively playing, or not.
    pub fn is_playing() -> bool {
        unsafe { ffi::Mix_PlayingMusic() == 1 }
    }

    /// If music is paused, or not.
    pub fn is_paused() -> bool {
        unsafe { ffi::Mix_PausedMusic() == 1 }
    }

    /// If music is fading, or not.
    pub fn get_fading() -> Fading {
        let ret = unsafe { ffi::Mix_FadingMusic() as isize } as c_uint;
        match ret {
            ffi::MIX_NO_FADING => Fading::NoFading,
            ffi::MIX_FADING_OUT => Fading::FadingOut,
            ffi::MIX_FADING_IN => Fading::FadingIn,
            _ => Fading::NoFading,
        }
    }
}

// 4.6 Effects

// TODO: Mix_RegisterEffect
// TODO: Mix_UnregisterEffect
// TODO: Mix_SetPostMix
