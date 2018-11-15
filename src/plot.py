import matplotlib
matplotlib.use("agg")
import matplotlib.pyplot as plt

x = []
for l in open("out.csv").readlines():
    x.append(float(l))

plt.hist(x)
plt.savefig("out.pdf", output="pdf")
