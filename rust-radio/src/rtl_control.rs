use rtlsdr;
use std;
use liquid_dsp;
use num::complex::Complex;

pub struct RtlFm {
    rtl: rtlsdr::RTLSDRDevice,
    recv_buff_size: usize,
    fm_demod: liquid_dsp::freqdem::Freqdem
}

impl RtlFm {

    pub fn new(device_index: i32) -> Result<RtlFm, rtlsdr::RTLSDRError> {
        let dev = rtlsdr::open(device_index);
        let modulation_index = 1.0;
        let demod = liquid_dsp::freqdem::Freqdem::new(modulation_index);
        let buff_size = 16384;
        match dev {
            Ok(mut dev) => {
                match Self::initialize(&mut dev) {
                    Ok(()) => Ok(RtlFm{rtl: dev, recv_buff_size: buff_size, fm_demod: demod}),
                    Err(err) => Err(err)
                }
            }
            Err(err) => Err(err)
        }
    }

    fn initialize(dev: &mut rtlsdr::RTLSDRDevice) -> Result<(), rtlsdr::RTLSDRError> {
        //TODO- figure a better way out to compose this... seems ugly
        match dev.get_xtal_freq() {
            Ok((rtl_freq, tuner_freq)) => {
                match dev.set_xtal_freq(rtl_freq, tuner_freq) {
                    Ok(()) => {
                        match dev.set_freq_correction(1) {
                            Ok(()) => Ok(()),
                            Err(err) => Err(err)
                        }
                    },
                    Err(err) => Err(err)
                }
            },
            Err(err) => Err(err)
        }
    }

    pub fn tune(self: &mut RtlFm, frequency: u32, sample_rate: u32) -> Result<(), rtlsdr::RTLSDRError> {
        match self.rtl.set_center_freq(frequency) {
            Ok(()) => {
                match self.rtl.set_sample_rate(sample_rate) {
                    Ok(()) => {
                        match self.rtl.reset_buffer() {
                            Ok(()) => Ok(()),
                            Err(err) => Err(err)
                        }
                    }
                    Err(err) => Err(err)
                }
                    
            }
            Err(err) => Err(err)
        }
    }

    pub fn get_pcm(self: &mut RtlFm) -> Result<std::vec::Vec<f32>, rtlsdr::RTLSDRError> {
        match self.rtl.read_sync(self.recv_buff_size) {
            Ok(data) => {
                let mut converted_data = rtl_to_complexf32(&data, data.len());
                Ok(self.fm_demod.demodulate_block(&converted_data))
            },
            Err(err) => Err(err)
        }
    }

    pub fn get_cx_f32_iq(self: &mut RtlFm) -> Result<std::vec::Vec<Complex<f32>>, rtlsdr::RTLSDRError> {
        match self.rtl.read_sync(self.recv_buff_size) {
            Ok(data) => {
                Ok(rtl_to_complexf32(&data, data.len()))
            },
            Err(err) => Err(err)
        }
    }

    #[allow(dead_code)] // Not actually dead code, since it is part of the library...
    pub fn get_u8_iq(self: &mut RtlFm) -> Result<std::vec::Vec<u8>, rtlsdr::RTLSDRError> {
       self.rtl.read_sync(self.recv_buff_size)
    }

    pub fn get_buffer_size(self: &mut RtlFm) -> usize {
        self.recv_buff_size
    }
}

fn rtl_to_complexf32(rtl_data: &[u8], num_elements: usize) -> Vec<Complex<f32>> {
    //There are 2 components to each complex value
    let mut vec = Vec::with_capacity(num_elements);
    let mut index = 0;
    let scale: f32 = 1.0/128.0;
    while index < num_elements {
        let mut current_i = rtl_data[index] as f32;
        let mut current_q = rtl_data[index+1] as f32;
        // 127.4 instead of 128 because of DC offset
        // - see https://www.reddit.com/r/RTLSDR/comments/2qrfvn/help_with_iq_samples/cn9aylp
        current_i = (current_i - 127.4) * scale; //Scale between -1 and 1
        current_q = (current_q - 127.4) * scale;
        let curr_cx = Complex{re: current_i, im: current_q};
        vec.push(curr_cx); 
        index = index + 2;
    }
    vec
}