/*!
A binding for SDL2_mixer.
 */

#![crate_type = "lib"]
#![desc = "SDL2_mixer bindings and wrappers"]
#![comment = "SDL2_mixer bindings and wrappers"]
#![license = "MIT"]

#![feature(globs, macro_rules)]

extern crate libc;
extern crate sdl2;

use std::default;
use std::ptr;
use std::mem;
use std::raw;
use std::c_str::CString;
use libc::{c_int, uint16_t, c_double};
use sdl2::get_error;
use sdl2::rwops::RWops;
use sdl2::version::Version;
use sdl2::SdlResult;

// Setup linking for all targets.
#[cfg(target_os="macos")]
mod mac {
    #[cfg(mac_framework)]
    #[link(kind="framework", name="SDL2_mixer")]
    extern {}

    #[cfg(not(mac_framework))]
    #[link(name="SDL2_mixer")]
    extern {}
}

#[cfg(any(
    target_os="win32",
    target_os="linux",
    target_os="freebsd"
))]
mod others {
    #[link(name="SDL2_mixer")]
    extern {}
}

#[allow(non_camel_case_types, dead_code)]
mod ffi;

// This comes from SDL_audio.h
#[allow(non_camel_case_types)]
mod ll {
    use libc::uint16_t;

    pub const AUDIO_U8     : uint16_t =     0x0008;
    pub const AUDIO_S8     : uint16_t =     0x8008;
    pub const AUDIO_U16LSB : uint16_t =     0x0010;
    pub const AUDIO_S16LSB : uint16_t =     0x8010;
    pub const AUDIO_U16MSB : uint16_t =     0x1010;
    pub const AUDIO_S16MSB : uint16_t =     0x9010;
    pub const AUDIO_U16    : uint16_t =     AUDIO_U16LSB;
    pub const AUDIO_S16    : uint16_t =     AUDIO_S16LSB;
    pub const AUDIO_S32LSB : uint16_t =     0x8020;
    pub const AUDIO_S32MSB : uint16_t =     0x9020;
    pub const AUDIO_S32    : uint16_t =     AUDIO_S32LSB;
    pub const AUDIO_F32LSB : uint16_t =     0x8120;
    pub const AUDIO_F32MSB : uint16_t =     0x9120;
    pub const AUDIO_F32    : uint16_t =     AUDIO_F32LSB;
    pub const AUDIO_U16SYS : uint16_t =     AUDIO_U16LSB;
    pub const AUDIO_S16SYS : uint16_t =     AUDIO_S16LSB;
    pub const AUDIO_S32SYS : uint16_t =     AUDIO_S32LSB;
    pub const AUDIO_F32SYS : uint16_t =     AUDIO_F32LSB;
}

pub type AudioFormat = uint16_t;

pub const AUDIO_U8     : AudioFormat = ll::AUDIO_U8;
pub const AUDIO_S8     : AudioFormat = ll::AUDIO_S8;
pub const AUDIO_U16LSB : AudioFormat = ll::AUDIO_U16LSB;
pub const AUDIO_S16LSB : AudioFormat = ll::AUDIO_S16LSB;
pub const AUDIO_U16MSB : AudioFormat = ll::AUDIO_U16MSB;
pub const AUDIO_S16MSB : AudioFormat = ll::AUDIO_S16MSB;
pub const AUDIO_U16    : AudioFormat = ll::AUDIO_U16;
pub const AUDIO_S16    : AudioFormat = ll::AUDIO_S16;
pub const AUDIO_S32LSB : AudioFormat = ll::AUDIO_S32LSB;
pub const AUDIO_S32MSB : AudioFormat = ll::AUDIO_S32MSB;
pub const AUDIO_S32    : AudioFormat = ll::AUDIO_S32;
pub const AUDIO_F32LSB : AudioFormat = ll::AUDIO_F32LSB;
pub const AUDIO_F32MSB : AudioFormat = ll::AUDIO_F32MSB;
pub const AUDIO_F32    : AudioFormat = ll::AUDIO_F32;
pub const AUDIO_U16SYS : AudioFormat = ll::AUDIO_U16SYS;
pub const AUDIO_S16SYS : AudioFormat = ll::AUDIO_S16SYS;
pub const AUDIO_S32SYS : AudioFormat = ll::AUDIO_S32SYS;
pub const AUDIO_F32SYS : AudioFormat = ll::AUDIO_F32SYS;

/// The suggested default is signed 16bit samples in host byte order.
pub const DEFAULT_FORMAT: AudioFormat = ll::AUDIO_S16SYS;
/// Defualt channels: Stereo.
pub const DEFAULT_CHANNELS : int = 2;
/// Good default sample rate in Hz (samples per second) for PC sound cards.
pub const DEFAULT_FREQUENCY : int = 22050;
/// Maximum value for any volume setting.
pub const MAX_VOLUME : int = 128;

/// Returns the version of the dynamically linked SDL_mixer library
pub fn get_linked_version() -> Version {

    unsafe {
        Version::from_ll(ffi::Mix_Linked_Version())
    }
}

bitflags!(flags InitFlag : u32 {
    const INIT_FLAC         = ffi::MIX_INIT_FLAC as u32,
    const INIT_MOD          = ffi::MIX_INIT_MOD as u32,
    const INIT_MODPLUG      = ffi::MIX_INIT_MODPLUG as u32,
    const INIT_MP3          = ffi::MIX_INIT_MP3 as u32,
    const INIT_OGG          = ffi::MIX_INIT_OGG as u32,
    const INIT_FLUID_SYNTH  = ffi::MIX_INIT_FLUIDSYNTH as u32
})

/// Loads dynamic libraries and prepares them for use.  Flags should be
/// one or more flags from InitFlag.
pub fn init(flags: InitFlag) -> InitFlag {
    unsafe {
        let ret = ffi::Mix_Init(flags.bits() as c_int);
        InitFlag::from_bits_truncate(ret as u32)
    }
}

/// Cleans up all dynamically loaded library handles, freeing memory.
pub fn quit() {
    unsafe { ffi::Mix_Quit(); }
}

/// Open the mixer with a certain audio format.
pub fn open_audio(frequency: int, format: AudioFormat, channels: int, chunksize: int) -> SdlResult<()> {
    let ret = unsafe { ffi::Mix_OpenAudio(frequency as c_int, format, channels as c_int, chunksize as c_int) };
    if ret == 0 { Ok(()) }
    else { Err(get_error()) }
}

/// Shutdown and cleanup the mixer API.
pub fn close_audio() {
    unsafe { ffi::Mix_CloseAudio() }
}

/// Get the actual audio format in use by the opened audio device.
pub fn query_spec() -> SdlResult<(int, AudioFormat, int)> {
    let frequency : c_int = 0;
    let format : uint16_t = 0;
    let channels : c_int  = 0;
    let ret = unsafe { ffi::Mix_QuerySpec(&frequency, &format, &channels) };
    if ret == 0 {
        Err(get_error())
    } else {
        Ok((frequency as int, format as AudioFormat, channels as int))
    }
}

// 4.2 Samples

/// Get the number of sample chunk decoders available from the Mix_GetChunkDecoder function.
pub fn get_chunk_decoders_number() -> int {
    unsafe { ffi::Mix_GetNumChunkDecoders() as int }
}

/// Get the name of the indexed sample chunk decoder.
pub fn get_chunk_decoder(index: int) -> String {
     unsafe {
        let name = ffi::Mix_GetChunkDecoder(index as c_int);
        CString::new(name, false).as_str().unwrap().into_string()
    }
}

/// The internal format for an audio chunk.
#[deriving(PartialEq)] #[allow(raw_pointer_deriving)]
pub struct Chunk {
    pub raw: *const ffi::Mix_Chunk,
    pub owned: bool
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
    pub fn from_file(path: &Path) -> SdlResult<Chunk> {
        let raw = unsafe {
            ffi::Mix_LoadWAV_RW(try!(RWops::from_file(path, "rb")).raw(), 0)
        };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Chunk{ raw: raw, owned: true })
        }
    }

    /// Set chunk->volume to volume.
    pub fn set_volume(&mut self, volume: int) -> int {
        unsafe {
            ffi::Mix_VolumeChunk(self.raw, volume as c_int) as int
        }
    }

    /// current volume for the chunk.
    pub fn get_volume(&self) -> int {
        unsafe {
            ffi::Mix_VolumeChunk(self.raw, -1) as int
        }
    }
}

/// Loader trait for RWops
pub trait LoaderRWops {
    /// Load src for use as a sample.
    fn load_wav(&self) -> SdlResult<Chunk>;
}

impl LoaderRWops for RWops {
    /// Load src for use as a sample.
    fn load_wav(&self) -> SdlResult<Chunk> {
        let raw = unsafe {
            ffi::Mix_LoadWAV_RW(self.raw(), 0)
        };
        if raw == ptr::null() {
            Err(get_error())
        } else {
            Ok(Chunk{ raw: raw, owned: true })
        }
    }
}


// 4.3 Channels

/// Fader effect type enumerations
#[repr(C)]
#[deriving(Clone, PartialEq, Hash, Show, FromPrimitive)]
pub enum Fading {
    NoFading  = ffi::MIX_NO_FADING as int,
    FadingOut = ffi::MIX_FADING_OUT as int,
    FadingIn  = ffi::MIX_FADING_IN as int
}

/// Sound effect channel.
#[deriving(PartialEq, Show)]
pub struct Channel(int);

/// Set the number of channels being mixed.
pub fn allocate_channels(numchans: int) -> int {
    unsafe {
        ffi::Mix_AllocateChannels(numchans as c_int) as int
    }
}

static mut channel_finished_callback: Option<raw::Closure> = None;

extern "C" fn c_channel_finished_callback(ch: c_int) {
    unsafe {
        match channel_finished_callback {
            None => (),
            Some(cb) => {
                let cb = mem::transmute::<_, |Channel|>(cb);
                cb(Channel(ch as int))
            }
        }
    }
}

/// When channel playback is halted, then the specified channel_finished function is called.
pub fn set_channel_finished(f: |Channel|:'static) {
    unsafe {
        channel_finished_callback = Some(mem::transmute::<_, raw::Closure>(f));
        ffi::Mix_ChannelFinished(Some(c_channel_finished_callback));
    }
}

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
    pub fn set_volume(self, volume: int) -> int {
        let Channel(ch) = self;
        unsafe {
            ffi::Mix_Volume(ch as c_int, volume as c_int) as int
        }
    }

    pub fn get_volume(self) -> int {
        let Channel(ch) = self;
        unsafe {
            ffi::Mix_Volume(ch as c_int, -1) as int
        }
    }

    /// Play chunk on channel, or if channel is -1, pick the first free unreserved channel.
    pub fn play(self, chunk: &Chunk, loops: int) -> SdlResult<Channel> {
        self.play_timed(chunk, loops, -1)
    }

    pub fn play_timed(self, chunk: &Chunk, loops: int, ticks: int) -> SdlResult<Channel> {
        let Channel(ch) = self;
        let ret = unsafe {
            ffi::Mix_PlayChannelTimed(ch as c_int, chunk.raw, loops as c_int, ticks as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(Channel(ret as int))
        }
    }

    /// Play chunk on channel, or if channel is -1, pick the first free unreserved channel.
    pub fn fade_in(self, chunk: &Chunk, loops: int, ms: int) -> SdlResult<Channel> {
        self.fade_in_timed(chunk, loops, ms, -1)
    }

    pub fn fade_in_timed(self, chunk: &Chunk, loops: int, ms: int, ticks: int) -> SdlResult<Channel> {
        let Channel(ch) = self;
        let ret = unsafe {
            ffi::Mix_FadeInChannelTimed(ch as c_int, chunk.raw, loops as c_int, ms as c_int, ticks as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(Channel(ret as int))
        }
    }

    /// Pause channel, or all playing channels if -1 is passed in.
    pub fn pause(self) {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Pause(ch as c_int); }
    }

    /// Unpause channel, or all playing and paused channels if -1 is passed in.
    pub fn resume(self) {
        let Channel(ch) = self;
        unsafe { ffi::Mix_Resume(ch as c_int); }
    }

    /// Halt channel playback
    pub fn halt(self) {
        let Channel(ch) = self;
        unsafe { ffi::Mix_HaltChannel(ch as c_int); }
    }

    /// Halt channel playback, after ticks milliseconds.
    pub fn expire(self, ticks: int) -> int {
        let Channel(ch) = self;
        unsafe { ffi::Mix_ExpireChannel(ch as c_int, ticks as c_int) as int }
    }

    /// Gradually fade out which channel over ms milliseconds starting from now.
    pub fn fade_out(self, ms: int) -> int {
        let Channel(ch) = self;
        unsafe { ffi::Mix_FadeOutChannel(ch as c_int, ms as c_int) as int }
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
        let ret = unsafe { ffi::Mix_FadingChannel(ch as c_int) as int };
        FromPrimitive::from_int(ret).unwrap()
    }

    /// Get the most recent sample chunk pointer played on channel.
    pub fn get_chunk(self) -> Option<Chunk> {
        let Channel(ch) = self;
        let raw = unsafe { ffi::Mix_GetChunk(ch as c_int) };
        if raw.is_null() {
            None
        } else {
            Some( Chunk { raw: raw, owned: false } )
        }
    }

    /// This removes all effects registered to channel.
    pub fn unregister_all_effects(self) -> SdlResult<()> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_UnregisterAllEffects(ch as c_int) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    pub fn set_panning(self, left: u8, right: u8) -> SdlResult<()> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPanning(ch as c_int, left, right) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// This effect simulates a simple attenuation of volume due to distance.
    pub fn set_distance(self, distance: u8) -> SdlResult<()> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetDistance(ch as c_int, distance) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// This effect emulates a simple 3D audio effect.
    pub fn set_position(self, angle: i16, distance: u8) -> SdlResult<()> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetPosition(ch as c_int, angle, distance) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Simple reverse stereo, swaps left and right channel sound.
    pub fn set_reverse_stereo(self, flip: bool) -> SdlResult<()> {
        let Channel(ch) = self;
        let ret = unsafe { ffi::Mix_SetReverseStereo(ch as c_int, flip as c_int) };
        if ret == 0 {
            Err(get_error())
        } else {
            Ok(())
        }
    }
}

pub fn get_playing_channels_number() -> int {
    unsafe { ffi::Mix_Playing(-1) as int }
}

pub fn get_paused_channels_number() -> int {
    unsafe { ffi::Mix_Paused(-1) as int }
}

// 4.4 Groups

/// Reserve num channels from being used when playing samples when
/// passing in -1 as a channel number to playback functions.
pub fn reserve_channels(num: int) -> int {
    unsafe { ffi::Mix_ReserveChannels(num as c_int) as int }
}

/// Sound effect channel grouping.
pub struct Group(int);

impl default::Default for Group {
    fn default() -> Group {
        Group(-1)
    }
}

impl Group {
    /// Add channels starting at from up through to to group tag,
    /// or reset it's group to the default group tag (-1).
    pub fn add_channels_range(self, from: int, to: int) -> int {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupChannels(from as c_int, to as c_int, g as c_int) as int }
    }

    /// Add which channel to group tag, or reset it's group to the default group tag
    pub fn add_channel(self, Channel(ch): Channel) -> bool {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupChannel(ch as c_int, g as c_int) == 1 }
    }

    /// Count the number of channels in group
    pub fn count(self) -> int {
        let Group(g) = self;
        unsafe { ffi::Mix_GroupCount(g as c_int) as int }
    }

    /// Find the first available (not playing) channel in group
    pub fn find_available(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupAvailable(g as c_int) as int };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Find the oldest actively playing channel in group
    pub fn find_oldest(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupOldest(g as c_int) as int };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Find the newest, most recently started, actively playing channel in group.
    pub fn find_newest(self) -> Option<Channel> {
        let Group(g) = self;
        let ret = unsafe { ffi::Mix_GroupNewer(g as c_int) as int };
        if ret == -1 {
            None
        } else {
            Some(Channel(ret))
        }
    }

    /// Gradually fade out channels in group over some milliseconds starting from now.
    /// Returns the number of channels set to fade out.
    pub fn fade_out(self, ms: int) -> int {
        let Group(g) = self;
        unsafe { ffi::Mix_FadeOutGroup(g as c_int, ms as c_int) as int }
    }

    /// Halt playback on all channels in group.
    pub fn halt(self) {
        let Group(g) = self;
        unsafe { ffi::Mix_HaltGroup(g as c_int); }
    }
}

// 4.5 Music

/// Get the number of music decoders available.
pub fn get_music_decoders_number() -> int {
    unsafe { ffi::Mix_GetNumMusicDecoders() as int }
}

/// Get the name of the indexed music decoder.
pub fn get_music_decoder(index: int) -> String {
    unsafe {
        let name = ffi::Mix_GetMusicDecoder(index as c_int);
        CString::new(name, false).as_str().unwrap().into_string()
    }
}

/// Music type enumerations
#[repr(C)]
#[deriving(Clone, PartialEq, Hash, Show, FromPrimitive)]
pub enum MusicType {
    MusicNone    = ffi::MUS_NONE as int,
    MusicCmd     = ffi::MUS_CMD as int,
    MusicWav     = ffi::MUS_WAV as int,
    MusicMod     = ffi::MUS_MOD as int,
    MusicMid     = ffi::MUS_MID as int,
    MusicOgg     = ffi::MUS_OGG as int,
    MusicMp3     = ffi::MUS_MP3 as int,
    MusicMp3Mad  = ffi::MUS_MP3_MAD as int,
    MusicFlac    = ffi::MUS_FLAC as int,
    MusicModPlug = ffi::MUS_MODPLUG as int
}

// hooks
static mut music_finished_hook: Option<raw::Closure> = None;

extern "C" fn c_music_finished_hook() {
    unsafe { match music_finished_hook {
        None => (),
        Some(f) => {
            let f = mem::transmute::<_, ||>(f);
            f()
        }
    } }
}

/// This is an opaque data type used for Music data.
#[deriving(PartialEq)] #[allow(raw_pointer_deriving)]
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

impl Music {
    /// Load music file to use.
    pub fn from_file(path: &Path) -> SdlResult<Music> {
        let raw = unsafe {
            ffi::Mix_LoadMUS(path.to_c_str().unwrap())
        };
        if raw.is_null() {
            Err(get_error())
        } else {
            Ok(Music{ raw: raw, owned: true })
        }
    }

    /// The file format encoding of the music.
    pub fn get_type(&self) -> MusicType {
        let ret = unsafe { ffi::Mix_GetMusicType(self.raw) as int };
        FromPrimitive::from_int(ret).unwrap()
    }

    /// Play the loaded music loop times through from start to finish.
    pub fn play(&self, loops: int) -> SdlResult<()> {
        let ret = unsafe {
            ffi::Mix_PlayMusic(self.raw, loops as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Fade in over ms milliseconds of time, the loaded music,
    /// playing it loop times through from start to finish.
    pub fn fade_in(&self, loops: int, ms: int) -> SdlResult<()> {
        let ret = unsafe {
            ffi::Mix_FadeInMusic(self.raw, loops as c_int, ms as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Fade in over ms milliseconds of time, from position.
    pub fn fade_in_from_pos(&self, loops: int, ms: int, position: f64) -> SdlResult<()> {
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
    pub fn get_volume() -> int {
        unsafe { ffi::Mix_VolumeMusic(-1) as int }
    }

    /// Set the volume.
    pub fn set_volume(volume: int) -> int {
        unsafe { ffi::Mix_VolumeMusic(volume as c_int) as int }
    }

    /// Pause the music playback.
    pub fn pause() {
        unsafe { ffi::Mix_PauseMusic(); }
    }

    /// Unpause the music.
    pub fn resume() {
        unsafe { ffi::Mix_ResumeMusic(); }
    }

    /// Rewind the music to the start.
    pub fn rewind() {
        unsafe { ffi::Mix_RewindMusic(); }
    }

    /// Set the position of the currently playing music.
    pub fn set_pos(position: f64) -> SdlResult<()> {
        let ret = unsafe {
            ffi::Mix_SetMusicPosition(position as c_double)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Setup a command line music player to use to play music.
    pub fn set_command(command: &str) -> SdlResult<()> {
        let ret = unsafe {
            ffi::Mix_SetMusicCMD(command.to_c_str().unwrap())
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    /// Halt playback of music.
    pub fn halt() {
        unsafe { ffi::Mix_HaltMusic(); }
    }

    /// Gradually fade out the music over ms milliseconds starting from now.
    pub fn fade_out(ms: int) -> SdlResult<()> {
        let ret = unsafe {
            ffi::Mix_FadeOutMusic(ms as c_int)
        };
        if ret == -1 {
            Err(get_error())
        } else {
            Ok(())
        }
    }

    // TODO: Mix_HookMusic
    // TODO: Mix_GetMusicHookData

    pub fn hook_finished(f: ||) {
        unsafe {
            music_finished_hook = Some(mem::transmute(f));
            ffi::Mix_HookMusicFinished(Some(c_music_finished_hook));
        }
    }

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
        let ret = unsafe { ffi::Mix_FadingMusic() as int };
        FromPrimitive::from_int(ret).unwrap()
    }
}

// 4.6 Effects

// TODO: Mix_RegisterEffect
// TODO: Mix_UnregisterEffect
// TODO: Mix_SetPostMix
