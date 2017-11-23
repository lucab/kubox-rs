FROM ekidd/rust-musl-builder:1.21.0
COPY . /home/rust/src/
RUN cargo build --locked --release --target x86_64-unknown-linux-musl 
RUN strip /home/rust/src/target/x86_64-unknown-linux-musl/release/kubox

FROM scratch
COPY --from=0 /home/rust/src/target/x86_64-unknown-linux-musl/release/kubox /sbin/
USER 0
CMD ["/sbin/kubox"] 
