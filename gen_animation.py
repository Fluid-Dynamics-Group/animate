import matplotlib.pyplot as plt
import numpy as np

NUM_PLOTS = 500

x = np.linspace(0, 2*np.pi, 1000)

for i in range(1,NUM_PLOTS+1):
    phase_shift = i / NUM_PLOTS * (np.pi)

    plt.cla()
    plt.clf()

    y = np.sin(x + phase_shift)

    plt.plot(x,y)
    plt.savefig(f"./static/plot_anim_{i:05}.png", dpi = 120, bbox_inches="tight")
