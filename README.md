## rust-radio

Simple FM Radio App for RTL-SDR that functions as both a command-line program as well as library. 

It is primarily a wrapper around rtlsdr-rs (https://github.com/adamgreig/rtlsdr-rs) and uses rust binding for liquid_dsp ("https://github.com/cubehub/rust-liquid-dsp.git") for the demodulation.

### Dependencies

Requires RTL-SDR dongle as well as librtlsdr driver to be install (available as package librtlsdr-dev on Ubuntu or follow steps on http://osmocom.org/projects/sdr/wiki/rtl-sdr)

```
rtlsdr = "0.1.4"

num = "0.1"

structopt = "0.2.8"

[dependencies.liquid_dsp]

git = "https://github.com/cubehub/rust-liquid-dsp.git"
```

### Usage

To play fm audio from the command line, you can pipe the output into sox or alsa

Example using Sox

```
cargo run -- -f 96900000 -s 250000 | play -t raw -r 250k -e floating-point -b 32 -c 1 -
```

This tells the RTL radio to tune to 96.9 MHz and sox to expect f32 pcm input at 250k samples/second

It is also possible to write the raw I/Q without demodulation to a file. For example:

```
cargo run -- -d 5 --no_demod -f 103500000 -s 250000 > raw.dat
```

To then view the spectral data, you can use the script in plot_iq_psd

```
python plot_iq_spectrum.py 103500000 250000 raw.dat
```
