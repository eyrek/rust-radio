#[macro_use] extern crate structopt;
extern crate rtlsdr;
extern crate num;
extern crate liquid_dsp;

mod rtl_control;

use std::io::{self, Write};
use structopt::StructOpt;
use rtl_control::*;


#[derive(StructOpt, Debug)]
#[structopt(name = "rust-radio")]
struct Opt {
    /// Number of seconds to run. Default = indefinite
    #[structopt(short = "d", long  = "duration", default_value = "-1")]
    duration: i32,

    /// Desired sample rate. Note- RTL does not support all sample rates.
    //From rtlsdr source: (valid sample rates are <
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
    #[structopt(short = "f", long  = "freqency", default_value = "96900000")]
    frequency: u32,

    /// Disable demodulation
    #[structopt(short = "n", long  = "no_demod")]
    disable_demod: bool,
}

fn main() {

    let opt = Opt::from_args();
    let count = rtlsdr::get_device_count();
    if count < 1 { 
        eprintln!("No Available RTL tuner!");
        return;
    } else {
        let mut dev = RtlFm::new(0).unwrap(); //Use first available RTL
        match dev.tune(opt.frequency, opt.sample_rate) {
            Ok(()) => (),
            Err(err) => {
                eprintln!("{:?}", err);
                return;
            }
        }
        let stdout = io::stdout();
        //Buffer size is in bytes, and 2 bytes per sample
        let num_samples_buffer = dev.get_buffer_size() as u32 / 2;
        let mut num_iterations = 0;
        //Avoid overflow
        if opt.duration > 0 {
            num_iterations = opt.sample_rate * (opt.duration as u32) / num_samples_buffer;
            eprintln!("Num iterations = {}", num_iterations);
        }
        let mut iteration = 0;
        while iteration < num_iterations || opt.duration < 0 { 
            if opt.disable_demod {
                let iq_data = dev.get_cx_f32_iq().unwrap();
                let raw = iq_data.as_ptr() as *const u8;
                //64 bits per complex sample
                let slice = unsafe { std::slice::from_raw_parts(raw, iq_data.len() * 8) };
                let mut handle = stdout.lock();
                let _written = handle.write_all(slice).unwrap();
                handle.flush().unwrap();
            } else {
                let fm_data = dev.get_pcm().unwrap();
                let raw = fm_data.as_ptr() as *const u8;
                //4 * 8 =  32, need to match
                let slice = unsafe { std::slice::from_raw_parts(raw, fm_data.len() * 4) };
                let mut handle = stdout.lock();
                let _written = handle.write_all(slice).unwrap();
                handle.flush().unwrap();
            }        
            iteration += 1;
        }
        std::process::exit(0x0100);
    }
}