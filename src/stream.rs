use crate::BassError;

#[cfg(target_family = "unix")]
use std::ffi::CString;

use std::ffi::c_void;

use bass_sys::*;

#[cfg(target_os = "windows")]
use widestring::U16CString;

pub struct Stream {
    handle: HSTREAM,
}

impl Drop for Stream {
    fn drop(&mut self) {
        BASS_StreamFree(self.handle);
    }
}

impl Stream {
    pub fn create_from_file(file_name: String) -> Result<Stream, BassError> {
        Self::create_from_file_with_flags(file_name, BASS_UNICODE)
    }

    pub fn create_from_file_with_flags(
        file_name: String,
        flags: DWORD,
    ) -> Result<Stream, BassError> {
        let handle;

        #[cfg(target_family = "windows")]
        {
            let file_name_raw = U16CString::from_str(file_name).unwrap();
            let file_name_raw = file_name_raw.into_raw() as *const c_void;

            handle = BASS_StreamCreateFile(0, file_name_raw, 0, 0, flags);
        }

        #[cfg(target_family = "unix")]
        {
            let file_name_raw = CString::new(file_name).unwrap();
            let file_name_raw = file_name_raw.as_ptr() as *const c_void;

            handle = BASS_StreamCreateFile(0, file_name_raw, 0, 0, 0);
        }

        if handle == 0 {
            let error_code = BASS_ErrorGetCode();

            match error_code {
                BASS_ERROR_FILEOPEN => return Err(BassError::FileCouldNotBeOpened),
                BASS_ERROR_FILEFORM => return Err(BassError::InvalidFileFormat),
                BASS_ERROR_NOTAUDIO => return Err(BassError::InvalidFileContent),
                BASS_ERROR_CODEC => return Err(BassError::InvalidCodec),
                BASS_ERROR_FORMAT => return Err(BassError::InvalidSampleFormat),
                BASS_ERROR_MEM => return Err(BassError::InsufficientMemory),
                _ => panic!(
                    "Failed to create the stream, error code: {}",
                    BASS_ErrorGetCode()
                ),
            }
        }

        Ok(Stream { handle })
    }

    pub fn create_from_url(url: String) -> Result<Stream, BassError> {
        let handle;

        #[cfg(target_family = "windows")]
        {
            let url_raw = U16CString::from_str(url).unwrap();
            let url_raw = url_raw.into_raw() as *const c_void;

            handle = BASS_StreamCreateFile(0, url_raw, 0, 0, BASS_UNICODE);

            unsafe { U16CString::from_raw(url_raw as *mut u16) };
        }

        #[cfg(target_family = "unix")]
        {
            let url_raw = CString::new(url).unwrap();
            let url_raw = url_raw.as_ptr() as *const c_void;

            handle = BASS_StreamCreateFile(0, url_raw, 0, 0, 0);

            unsafe { CString::from_raw(url_raw as *mut i8) };
        }

        if handle == 0 {
            let error_code = BASS_ErrorGetCode();

            match error_code {
                BASS_ERROR_NONET => return Err(BassError::NoInternetConnection),
                BASS_ERROR_PROTOCOL => return Err(BassError::InvalidProtocol),
                BASS_ERROR_SSL => return Err(BassError::SslSupportNotAvailable),
                BASS_ERROR_TIMEOUT => return Err(BassError::TimeOut),
                BASS_ERROR_FILEOPEN => return Err(BassError::FileCouldNotBeOpened),
                BASS_ERROR_FILEFORM => return Err(BassError::InvalidFileFormat),
                BASS_ERROR_UNSTREAMABLE => return Err(BassError::UnstreamableFile),
                BASS_ERROR_NOTAUDIO => return Err(BassError::InvalidFileContent),
                BASS_ERROR_CODEC => return Err(BassError::InvalidCodec),
                BASS_ERROR_FORMAT => return Err(BassError::InvalidSampleFormat),
                BASS_ERROR_MEM => return Err(BassError::InsufficientMemory),
                _ => panic!(
                    "Failed to create the stream, error code: {}",
                    BASS_ErrorGetCode()
                ),
            }
        }

        Ok(Stream { handle })
    }

    pub fn play(&self) -> Result<(), BassError> {
        if BASS_ChannelPlay(self.handle, 0) == 0 {
            let error_code = BASS_ErrorGetCode();

            match error_code {
                BASS_ERROR_START => return Err(BassError::OutputIsPausedOrStopped),
                _ => panic!("Failed to play the stream, error code: {}", error_code),
            }
        }

        Ok(())
    }

    pub fn pause(&self) -> Result<(), BassError> {
        if BASS_ChannelPause(self.handle) == 0 {
            let error_code = BASS_ErrorGetCode();

            match error_code {
                BASS_ERROR_NOPLAY => return Err(BassError::StreamIsNotPlaying),
                _ => panic!("Failed to pause the stream, error code: {}", error_code),
            }
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<(), BassError> {
        if BASS_ChannelStop(self.handle) == 0 {
            panic!(
                "Failed to stop the stream, error code: {}",
                BASS_ErrorGetCode()
            );
        }

        Ok(())
    }

    pub fn lock(&self) {
        if BASS_ChannelLock(self.handle, 1) == 0 {
            panic!(
                "Failed to lock the stream, error code: {}",
                BASS_ErrorGetCode()
            );
        }
    }

    pub fn unlock(&self) {
        if BASS_ChannelLock(self.handle, 0) == 0 {
            panic!(
                "Failed to unlock the stream, error code: {}",
                BASS_ErrorGetCode()
            );
        }
    }

    pub fn get_bit_rate(&self) -> f32 {
        let mut bit_rate = 0.0f32;

        BASS_ChannelGetAttribute(self.handle, BASS_ATTRIB_BITRATE, &mut bit_rate as *mut f32);

        bit_rate
    }

    pub fn get_buffering_length(&self) -> f32 {
        let mut buffering_length = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_BUFFER,
            &mut buffering_length as *mut f32,
        );

        buffering_length
    }

    pub fn get_sample_rate(&self) -> f32 {
        let mut sample_rate = 0.0f32;

        BASS_ChannelGetAttribute(self.handle, BASS_ATTRIB_FREQ, &mut sample_rate as *mut f32);

        sample_rate
    }

    pub fn get_processing_granularity(&self) -> f32 {
        let mut processing_granularity = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_GRANULE,
            &mut processing_granularity as *mut f32,
        );

        processing_granularity
    }

    pub fn get_buffer_level_required_to_resume_stalled_playback(&self) -> f32 {
        let mut buffer_level_required_to_resume_stalled_playback = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_NET_RESUME,
            &mut buffer_level_required_to_resume_stalled_playback as *mut f32,
        );

        buffer_level_required_to_resume_stalled_playback
    }

    pub fn get_playback_buffering_switch(&self) -> f32 {
        let mut playback_buffering_switch = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_NOBUFFER,
            &mut playback_buffering_switch as *mut f32,
        );

        playback_buffering_switch
    }

    pub fn get_playback_ramping_switch(&self) -> f32 {
        let mut playback_ramping_switch = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_NORAMP,
            &mut playback_ramping_switch as *mut f32,
        );

        playback_ramping_switch
    }

    pub fn get_panning_position(&self) -> f32 {
        let mut panning_position = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_PAN,
            &mut panning_position as *mut f32,
        );

        panning_position
    }

    pub fn get_sample_rate_conversion_quality(&self) -> f32 {
        let mut sample_rate_conversion_quality = 0.0f32;

        BASS_ChannelGetAttribute(
            self.handle,
            BASS_ATTRIB_SRC,
            &mut sample_rate_conversion_quality as *mut f32,
        );

        sample_rate_conversion_quality
    }

    pub fn get_volume(&self) -> f32 {
        let mut volume = 0.0f32;

        BASS_ChannelGetAttribute(self.handle, BASS_ATTRIB_VOL, &mut volume as *mut f32);

        volume
    }

    pub fn set_buffering_length(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_BUFFER, value);
    }

    pub fn set_sample_rate(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_FREQ, value);
    }

    pub fn set_processing_granularity(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_GRANULE, value);
    }

    pub fn set_buffer_level_required_to_resume_stalled_playback(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NET_RESUME, value);
    }

    pub fn set_playback_buffering_switch(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NOBUFFER, value);
    }

    pub fn set_playback_ramping_switch(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NORAMP, value);
    }

    pub fn set_panning_position(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_PAN, value);
    }

    pub fn set_volume(&self, value: f32) {
        BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_VOL, value);
    }

    pub fn get_position(&self) -> u64 {
        BASS_ChannelGetPosition(self.handle, 0)
    }

    pub fn get_time(&self) -> f64 {
        BASS_ChannelBytes2Seconds(self.handle, self.get_position())
    }

    pub fn get_raw_handle(&self) -> &HSTREAM {
        &self.handle
    }
}
