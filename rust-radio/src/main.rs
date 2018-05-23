extern crate rtlsdr;
extern crate liquid_dsp;
extern crate num;
#[macro_use] extern crate structopt;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::io::{self, Write};
use num::complex::Complex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rust-radio")]
struct Opt {
    /// Number of seconds to run. Default = indefinite
    #[structopt(short = "d", long  = "duration", default_value = "-1")]
    duration: i32,

    /// Desired sample rate. Note- RTL does not support all sample rates.
    //From librtlsdr source: (valid sample rates are <
    /* check if the rate is supported by the resampler 
    if ((samp_rate <= 225000) || (samp_rate > 3200000) ||
       ((samp_rate > 300000) && (samp_rate <= 900000))) {
        fprintf(stderr, "Invalid sample rate: %u Hz\n", samp_rate);
        return -EINVAL;
    }
    */
    #[structopt(short = "s", long  = "sample_rate", default_value = "250000")]
    sample_rate: u32,

    /// Desired radio frequency (RF).
    #[structopt(short = "f", long  = "freqency", default_value = "103500000")]
    frequency: u32,

    /// Disable demodulation
    #[structopt(short = "n", long  = "no_demod")]
    disable_demod: bool,
}

//RTL outputs 8 bit I/ 8 bit Q complex unsigned int
//Need to convert it to complex float 32 for liquid-dsp
fn rtl_to_complexf32(rtl_data: &[u8], num_elements: usize) -> std::vec::Vec<Complex<f32>> {
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
        let curr_cx = num::Complex{re: current_i, im: current_q};
        vec.push(curr_cx); 
        index = index + 2;
    }
    vec
}

fn main() {

    let opt = Opt::from_args();
    //println!("{:?}", opt);

    let count = rtlsdr::get_device_count();
    //println!("Found {} device(s)", count);

    for index in 0..count {
        //println!("Index {}:", index);

        let name = rtlsdr::get_device_name(index);
        //println!("  Name: {}", name);

        let strs = rtlsdr::get_device_usb_strings(index).unwrap();
        //println!("  Manufacturer: {}", strs.manufacturer);
        //println!("  Product:      {}", strs.product);
        //println!("  Serial:       {}", strs.serial);
        //println!("");

        let idx2 = rtlsdr::get_index_by_serial(strs.serial).unwrap();
        //println!("  Index looked up by serial: {}", idx2);

        //println!("  Opening device...");
        let mut dev = rtlsdr::open(index).unwrap();
        //println!("");

        //println!("  Getting USB strings from opened device...");
        let strs2 = dev.get_usb_strings().unwrap();
        //println!("  Manufacturer: {}", strs2.manufacturer);
        //println!("  Product:      {}", strs2.product);
        //println!("  Serial:       {}", strs2.serial);
        //println!("");

        let (rtl_freq, tuner_freq) = dev.get_xtal_freq().unwrap();
        //println!("  RTL clock freq: {}Hz", rtl_freq);
        //println!("  Tuner clock freq: {}Hz", tuner_freq);

        //println!("  Setting RTL and tuner clock frequencies to same...");
        dev.set_xtal_freq(rtl_freq, tuner_freq).unwrap();

        //println!("  Setting centre frequency to {}Hz...", opt.frequency);
        dev.set_center_freq(opt.frequency).unwrap();

        let cfreq = dev.get_center_freq().unwrap();
        //println!("  Read current centre frequency: {}Hz", cfreq);

        let ppm = dev.get_freq_correction();
        //println!("  Current freq correction: {}ppm", ppm);

        //println!("  Setting freq correction to 1ppm...");
        dev.set_freq_correction(1).unwrap();

        let (t_id, t_name) = dev.get_tuner_type();
        //println!("  Tuner is a {} (id {})", &t_name, &t_id);

        let gains = dev.get_tuner_gains().unwrap();
        //println!("  Available gains: {:?}", &gains);

        //println!("  Setting gain to second option {}dB",
                 //(gains[1] as f64)/10.0f64);
        //dev.set_tuner_gain(gains[1]).unwrap();

        //let gain = dev.get_tuner_gain();
        //println!("  Current gain: {}dB", (gain as f64)/10.0f64);

        //println!("  Setting sample rate to {}kHz...", opt.sample_rate/1000u32);
        dev.set_sample_rate(opt.sample_rate).unwrap();

        let rate = dev.get_sample_rate().unwrap();
        //println!("  Current sample rate: {}Hz", rate);

        let m = dev.get_direct_sampling().unwrap();
        //println!("  Direct sampling mode: {:?}", m);

        dev.reset_buffer().unwrap();

        /*
        For these the maximum deviation is ±75 kHz and the sample rate is 250 kHz. 
        This means that the deviation ratio is 75 / 250 = 0.3.
        */
        let modulation_index: f32 = 1.0;
        //#define DEFAULT_BUF_LENGTH        (16 * 16384) rtl.c
        let hardcoded_buffer_size = 131072;
        let fdemod = liquid_dsp::freqdem::Freqdem::new(modulation_index);
        //let mut file = File::create(&Path::new("data.fm")).unwrap();
        let stdout = io::stdout();
        //println!("  Created file: data.fm");
        let mut samps_collected = 0;
        // Multiply by 2 because complex
        let samps_to_collect = (opt.sample_rate as i32) * opt.duration * 2;
        let resampler = liquid_dsp::msresamp::MsresampCrcf::new(0.48, 45.0);
        while samps_collected < samps_to_collect { 
            //println!("Before receiving data, {} samples", samps_collected);
            let data = dev.read_sync(hardcoded_buffer_size).unwrap();
            //println!("Received data");
            let mut converted_data = rtl_to_complexf32(&data, data.len());
            let resampled_data = resampler.resample(&mut converted_data);

            if opt.disable_demod {
                let raw = converted_data.as_ptr() as *const u8;
                //4 * 8 =  32, need to match
                let slice = unsafe { std::slice::from_raw_parts(raw, converted_data.len() * 4) };
                let mut handle = stdout.lock();
                let _written = handle.write(slice).unwrap();
            }
            else{
                let fm_data = fdemod.demodulate_block(&resampled_data);
                let raw = fm_data.as_ptr() as *const u8;
                //4 * 8 =  32, need to match
                let slice = unsafe { std::slice::from_raw_parts(raw, fm_data.len() * 4) };
                let mut handle = stdout.lock();
                let _written = handle.write(slice).unwrap();
            }        
            //println!("Wrote to file");
            samps_collected = samps_collected + (hardcoded_buffer_size as i32);
        }
        
        //println!("  Closing device...");
        dev.close().unwrap();
        //println!("  Farewell!");
        //RTL library isnt cleaning up after itself.. need to investigate further
        std::process::exit(0x0100);
    }

    /*
    /// Synchronous iterator for accessing IQ data
    struct StreamingIQ {
        dev: rtlsdr::RTLSDRDevice
    }

    impl StreamingIQ {
        fn new(dev: &rtlsdr::RTLSDRDevice) -> StreamingIQ {
            StreamingIQ { 
                dev: dev
            }
        }
    }

    impl Iterator for StreamingIQ {
        type Item = std::vec::Vec<u8>;
        fn next(&mut self) -> Option<std::vec::Vec<u8>> {
            let data = self.dev.read_sync(131072);
            match data {
                Ok(v) => Some(v),
                Err(e) => None,
            }
        }
    }*/

}