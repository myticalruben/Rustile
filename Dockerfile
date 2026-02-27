FROM rust:latest

# Instalamos dependencias de X11 y herramientas de prueba
RUN apt-get update && apt-get install -y \
    libx11-dev \
    libxcb1-dev \
    libxcb-keysyms1-dev \
    xserver-xephyr \
    xterm \
    x11-apps \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY .. .

# Comando por defecto: mantiene el contenedor vivo
CMD ["sleep", "infinity"]