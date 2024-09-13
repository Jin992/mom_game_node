FROM rust:1.81.0
LABEL authors="Yevhen Arteshchuk"

COPY ./ ./

RUN apt-get update && apt-get install -y librust-alsa-sys-dev libudev-dev
RUN cargo build --release

CMD ["./target/release/mmo_game_node"]
