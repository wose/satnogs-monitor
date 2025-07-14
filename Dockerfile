FROM rust:latest


RUN apt update
RUN DEBIAN_FRONTEND=noninteractive apt install -y vim mc git cmake build-essential libglib2.0-dev

RUN     git clone https://github.com/cubehub/libgpredict.git
WORKDIR /libgpredict
RUN     mkdir build
WORKDIR /libgpredict/build
RUN     cmake -DBUILD_SHARED_LIBS=ON ../
RUN     make
RUN     make install
RUN     ldconfig

RUN     rustup install stable

COPY    . /satnogs-monitor/
WORKDIR /satnogs-monitor/monitor
RUN     cargo build --release

CMD     cargo run --release -- -s ${STATION_ID}
