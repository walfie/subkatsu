FROM clux/muslrust:1.32.0-stable as builder

COPY . /workspace
RUN set -x \
  && curl -Lfo /tmp/NotoSansCJKjp-Medium.otf \
     "https://github.com/googlei18n/noto-cjk/blob/master/NotoSansCJKjp-Medium.otf?raw=true" \
  && cd /workspace \
  && cargo build --release \
  && mv /workspace/target/*/release /out

FROM jrottenberg/ffmpeg:4.1-alpine

# Copy over Noto Sans so we have a good fallback font for CJK
COPY --from=builder /tmp/NotoSansCJKjp-Medium.otf /usr/share/fonts/
COPY --from=builder /out/subkatsu /app

ENTRYPOINT ["/app"]

