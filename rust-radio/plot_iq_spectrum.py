import matplotlib.pyplot as plt
import numpy as np


def plot_iq(iq, np_data_type, frequency, sample_rate):
    """Plots the power spectrum of the given I/Q data.

    Args:
        iq: String containing alternating I/Q pairs (e.g. IQIQIQIQIQ..etc)
        dtype: numpy dtype to interpret the data as. Rather than dealing
            with a complex type, just treat each I and Q as same type 
            (e.g 32 bit complex float is just 32 bit float)
    """

    #Convert to Numpy array
    iq_array = np.fromstring(iq, dtype = np_data_type)
    #Get Power (Power = I^2 + Q^2)
    all_i = iq_array[::2] ** 2
    all_q = iq_array[1::2] ** 2
    pwr_array = np.add(all_i, all_q)
    #Take FFT
    fft_len = 4096
    fft_array = np.fft.fft(pwr_array, fft_len)
    #Shift FFT
    fft_array = np.fft.fftshift(fft_array)
    db_array = 20*np.log10(abs(fft_array))
    x_vals = np.arange( frequency - sample_rate/2, frequency + sample_rate /2, sample_rate/fft_len)
    #Plot DB values
    plt.plot(x_vals, db_array)
    #plt.xlim(frequency - (sample_rate/2), frequency + (sample_rate /2))
    plt.ylim(min(db_array), max(db_array))

    plt.show()

if __name__ == "__main__":
    with open('f32_resampled.dat', 'r') as data_file:
        iq_data = data_file.read()
        plot_iq(iq_data, np.float32, 103500000.0, 250000.0)