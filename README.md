# esp32c3-current-monitor

```shell
nix-shell
cargo run --release
# wait for flashing to complete...
# serial output, time, current shunt voltage, bus voltage.
# current uA = current shunt mV * 100 
# 8mV == 800uA == 0.8mA
Init!
2669876 us, 0 mV, 0 mV
2802594 us, 0 mV, 0 mV
2935287 us, 8 mV, 0 mV
3069009 us, 0 mV, 0 mV
3201762 us, 0 mV, 0 mV
3334489 us, 8 mV, 0 mV
3468201 us, 0 mV, 0 mV
3600975 us, 0 mV, 0 mV
3733712 us, 8 mV, 0 mV
3867438 us, 0 mV, 0 mV
4000235 us, 0 mV, 0 mV
4132990 us, 8 mV, 0 mV
4266799 us, 0 mV, 0 mV
4399605 us, 0 mV, 0 mV
```
