FROM debian:trixie-slim
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends dosbox-x xvfb xdotool imagemagick x11-utils && rm -rf /var/lib/apt/lists/*
ENV DISPLAY=:1 SDL_AUDIODRIVER=dummy
CMD ["sh","-c","Xvfb :1 -screen 0 640x480x24 & sleep 1; sleep infinity"]
