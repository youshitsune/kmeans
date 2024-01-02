COMP=rustc

all: kmeans

kmeans:
	$(COMP) main.rs -o kmeans
