default:
  #!/usr/bin/env bash
  just run | just tee-vcd ./logs/encoder.vcd

run:
  #!/usr/bin/env bash
  cargo run --release

tee-vcd OUT:
  #!/usr/bin/env bash
  tee >(awk '/--- VCD FILE START ---/ {p=1; next} p' - > {{OUT}})

stream:
  #!/usr/bin/env bash
  # remove fifo
  rm -f /tmp/gtkwave-fifo
  # create fifo
  mkfifo /tmp/gtkwave-fifo
  just run 2>/dev/null | just tee-vcd /tmp/gtkwave-fifo

gtkwave-stream FILE="/tmp/gtkwave-fifo":
  #!/usr/bin/env bash
  shmidcat {{FILE}} | gtkwave -v -I

gtkwave FILE:
  #!/usr/bin/env bash
  gtkwave --dark --rcvar 'fontname_signals Monospace 17' --rcvar 'fontname_waves Monospace 16' {{FILE}}
