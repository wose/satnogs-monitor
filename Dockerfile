FROM rust:latest


RUN apt update
RUN DEBIAN_FRONTEND=noninteractive apt install -y vim mc git cmake build-essential

RUN     git clone https://github.com/cubehub/libgpredict.git
WORKDIR /libgpredict
RUN     mkdir build
WORKDIR /libgpredict/build
RUN     cmake ../
RUN     make
RUN     make install
RUN     ldconfig

RUN     rustup install stable

WORKDIR /
RUN     git clone https://github.com/wose/satnogs-monitor.git
WORKDIR /satnogs-monitor/monitor
RUN     cargo build --release

CMD     cargo run --release -- -s ${STATION_ID}
